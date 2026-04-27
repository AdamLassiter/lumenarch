use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::ship::{ModuleKind, ShipDefinition, ShipModule};

use super::super::{
    components::{
        AngularVelocity, CurrentStation, EngineModule, HeldInteraction, Integrity, Interactable,
        InternalPosition, LinearVelocity, MissionState, ModuleFieldEmitter, ModuleRuntimeState,
        NearbyInteraction, PlayerFieldState, PlayerShip, PlayerShipAssignment,
        PowerConsumer, PowerProducer, RuntimeShipModule, ShipControlMode, ShipControlState,
        ShipInteriorMap, ShipInteriorNode, ShipPowerState, ShipRoot,
        ShipWeaponState, ShipboardControlState, ShipboardMarker, ShipboardPlayer, SimPosition,
        SimRotation, WeaponModule,
    },
    helpers::{
        count_modules, module_integrity, module_local_position, module_local_translation,
        ship_movement_model, ship_power_model, sprite_path_for_kind, FixedVec2, Fx,
    },
    RUNTIME_SHIP_ORIGIN,
};
use super::super::super::state::PlayingCleanup;

pub(super) fn spawn_runtime_ship(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
) {
    let engine_count = count_modules(ship, ModuleKind::Engine);
    let reactor_count = count_modules(ship, ModuleKind::Reactor);
    let battery_count = count_modules(ship, ModuleKind::Battery);
    let turret_count = count_modules(ship, ModuleKind::Turret);
    let movement_model = ship_movement_model(ship.modules.len(), engine_count);
    let power_model = ship_power_model(
        ship.modules.len(),
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
    );

    let root_entity = commands
        .spawn((
            Transform::from_translation(RUNTIME_SHIP_ORIGIN),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            PlayerShip,
            ShipRoot,
            PlayingCleanup,
        ))
        .insert((
            LinearVelocity {
                value: FixedVec2::zero(),
            },
            AngularVelocity {
                radians_per_second: Fx::from_num(0),
            },
            SimPosition {
                value: FixedVec2::from_vec2(RUNTIME_SHIP_ORIGIN.truncate()),
            },
            SimRotation {
                radians: Fx::from_num(0),
            },
            movement_model,
            ShipPowerState {
                stored_energy: power_model.battery_capacity,
                generation: power_model.reactor_output,
                draw: power_model.passive_draw,
                surplus: power_model.reactor_output - power_model.passive_draw,
                engine_power_ratio: if engine_count > 0 {
                    Fx::from_num(1)
                } else {
                    Fx::from_num(0)
                },
                weapons_powered: turret_count > 0,
                engines_powered: engine_count > 0,
            },
            power_model,
            ShipControlState::default(),
            ShipboardControlState {
                mode: ShipControlMode::ShipFlight,
            },
            ShipWeaponState {
                turret_count,
                cooldown_remaining: Fx::from_num(0),
                cooldown_duration: if turret_count > 0 {
                    Fx::from_num(0.3)
                } else {
                    Fx::from_num(0)
                },
            },
            MissionState {
                failed: false,
                failure_reason: None,
                encounter_cleared: false,
                completed: false,
                completion_reason: None,
                salvage_collected: false,
                salvage_scrap_awarded: 0,
                return_delay_remaining: None,
            },
        ))
        .id();

    let (min_x, max_x, min_y, max_y) = ship.bounds().unwrap_or((0, 0, 0, 0));
    let center_x = (min_x + max_x) as f32 * 0.5;
    let center_y = (min_y + max_y) as f32 * 0.5;
    let center_x_fixed = Fx::from_num(center_x);
    let center_y_fixed = Fx::from_num(center_y);

    let interior_nodes = build_interior_nodes(ship, center_x_fixed, center_y_fixed);
    let start_node = interior_nodes
        .iter()
        .position(|node| node.kind == ModuleKind::Cockpit)
        .unwrap_or(0);
    let start_station = interior_nodes
        .get(start_node)
        .cloned()
        .unwrap_or(ShipInteriorNode {
            module_id: 0,
            kind: ModuleKind::Cockpit,
            grid_x: 0,
            grid_y: 0,
            local_position: FixedVec2::zero(),
        });

    let child_entities: Vec<_> = ship
        .modules
        .iter()
        .map(|module| {
            spawn_runtime_module(
                commands,
                asset_server,
                module,
                center_x,
                center_y,
                center_x_fixed,
                center_y_fixed,
            )
        })
        .collect();

    let shipboard_marker = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.82, 0.96, 0.62), Vec2::splat(12.0)),
            Transform::from_xyz(
                start_station.local_position.x.to_num::<f32>(),
                start_station.local_position.y.to_num::<f32>(),
                6.0,
            ),
            ShipboardPlayer,
            ShipboardMarker,
            PlayerShipAssignment {
                _ship_entity: root_entity,
            },
            InternalPosition {
                node_index: start_node,
                grid_x: start_station.grid_x,
                grid_y: start_station.grid_y,
                local_position: start_station.local_position,
            },
            CurrentStation {
                module_id: start_station.module_id,
                kind: start_station.kind,
            },
            NearbyInteraction::default(),
            HeldInteraction::default(),
            PlayerFieldState {
                local_heat: Fx::from_num(0),
                local_electrical: Fx::from_num(0),
                heat_danger: false,
                electrical_danger: false,
            },
            PlayingCleanup,
        ))
        .id();

    commands
        .entity(root_entity)
        .insert(ShipInteriorMap {
            walkable_nodes: interior_nodes,
        })
        .add_children(&child_entities)
        .add_child(shipboard_marker);
}

fn build_interior_nodes(
    ship: &ShipDefinition,
    center_x_fixed: Fx,
    center_y_fixed: Fx,
) -> Vec<ShipInteriorNode> {
    ship.modules
        .iter()
        .map(|module| ShipInteriorNode {
            module_id: module.id,
            kind: module.kind,
            grid_x: module.grid_x,
            grid_y: module.grid_y,
            local_position: module_local_position(module, center_x_fixed, center_y_fixed),
        })
        .collect()
}

fn spawn_runtime_module(
    commands: &mut Commands,
    asset_server: &AssetServer,
    module: &ShipModule,
    center_x: f32,
    center_y: f32,
    center_x_fixed: Fx,
    center_y_fixed: Fx,
) -> Entity {
    let local_position = module_local_position(module, center_x_fixed, center_y_fixed);
    let mut entity = commands.spawn((
        Sprite::from_image(asset_server.load(sprite_path_for_kind(&module.kind))),
        Transform {
            translation: module_local_translation(module, center_x, center_y),
            rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
            ..default()
        },
        RuntimeShipModule {
            module_id: module.id,
            kind: module.kind,
            grid_x: module.grid_x,
            grid_y: module.grid_y,
            local_position,
        },
        Integrity {
            current: module_integrity(module.kind),
            max: module_integrity(module.kind),
        },
        ModuleRuntimeState {
            current_heat: Fx::from_num(0),
            electrical_instability: Fx::from_num(0),
            sampled_heat: Fx::from_num(0),
            sampled_electrical: Fx::from_num(0),
            is_disabled: false,
            needs_attention: false,
            last_interaction_age: Fx::from_num(0),
        },
        module_field_emitter(module.kind),
        Interactable,
        PlayingCleanup,
    ));

    match module.kind {
        ModuleKind::Reactor => {
            entity.insert(PowerProducer { output: 10 });
        }
        ModuleKind::Battery => {
            entity.insert(PowerProducer { output: 4 });
        }
        ModuleKind::Engine => {
            entity.insert((PowerConsumer { draw: 3 }, EngineModule));
        }
        ModuleKind::Turret => {
            entity.insert((PowerConsumer { draw: 2 }, WeaponModule));
        }
        _ => {}
    }

    entity.id()
}

fn module_field_emitter(kind: ModuleKind) -> ModuleFieldEmitter {
    match kind {
        ModuleKind::Reactor => ModuleFieldEmitter {
            heat_output: Fx::from_num(5),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(2),
        },
        ModuleKind::Engine => ModuleFieldEmitter {
            heat_output: Fx::from_num(3),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(1),
        },
        ModuleKind::Turret => ModuleFieldEmitter {
            heat_output: Fx::from_num(2),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(1),
        },
        ModuleKind::Battery => ModuleFieldEmitter {
            heat_output: Fx::from_num(1),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(2),
        },
        ModuleKind::Hull | ModuleKind::HullCorner | ModuleKind::Airlock => ModuleFieldEmitter {
            heat_output: Fx::from_num(0),
            cooling_output: Fx::from_num(2),
            electrical_output: Fx::from_num(0),
        },
        _ => ModuleFieldEmitter {
            heat_output: Fx::from_num(0),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(0),
        },
    }
}

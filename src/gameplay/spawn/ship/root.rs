use bevy::prelude::*;
use ggrs::PlayerHandle;

use super::{
    interior::{build_atmosphere_tiles, build_interior_nodes},
    modules::spawn_runtime_module,
};
use crate::{
    balance::BalanceConfig,
    gameplay::{
        RUNTIME_SHIP_ORIGIN,
        components::{
            AngularVelocity,
            CarriedResource,
            CurrentStation,
            HeldInteraction,
            HostileShip,
            HostileShipAi,
            InternalPosition,
            LinearVelocity,
            MissionState,
            NearbyInteraction,
            ObservedLocalPlayerMarker,
            PlayerFieldState,
            PlayerHandleComponent,
            PlayerMotionState,
            PlayerReferenceFrame,
            PlayerShip,
            PlayerShipAssignment,
            ShipArchCommandState,
            ShipAtmosphereState,
            ShipAutomationMode,
            ShipAutomationState,
            ShipControlMode,
            ShipControlState,
            ShipInertiaField,
            ShipInteriorMap,
            ShipInteriorNode,
            ShipPowerState,
            ShipRoot,
            ShipWeaponState,
            ShipboardControlState,
            ShipboardMarker,
            ShipboardPlayer,
            SimPosition,
            SimRotation,
            StationFocusMode,
        },
        helpers::{
            FixedVec2,
            Fx,
            module_local_position,
            ship_movement_model_with_effective,
            ship_power_model_with_effective,
        },
    },
    netcode,
    ship::{ModuleKind, ModuleSpec, ShipDefinition},
    state::PlayingCleanup,
};

#[derive(Default)]
struct ShipVariantTotals {
    engine_count: u32,
    reactor_count: u32,
    battery_count: u32,
    turret_count: u32,
    shield_count: u32,
    computer_count: u32,
    effective_engines: Fx,
    effective_reactors: Fx,
    effective_batteries: Fx,
    effective_charge_rate: Fx,
    effective_discharge_rate: Fx,
    effective_turrets: Fx,
    effective_helm: Fx,
    inertia_multiplier: Fx,
}

fn accumulate_ship_variant_totals(ship: &ShipDefinition) -> ShipVariantTotals {
    let mut totals = ShipVariantTotals {
        effective_helm: Fx::from_num(1),
        inertia_multiplier: Fx::from_num(1),
        ..Default::default()
    };

    for module in &ship.modules {
        let spec = ModuleSpec::for_module(module.kind, module.variant);
        match module.kind {
            ModuleKind::Engine => {
                totals.engine_count += 1;
                totals.effective_engines += Fx::from_num(spec.engine_multiplier.max(0.0));
            }
            ModuleKind::Reactor => {
                totals.reactor_count += 1;
                totals.effective_reactors += Fx::from_num(spec.reactor_output_multiplier.max(0.0));
            }
            ModuleKind::Battery => {
                totals.battery_count += 1;
                totals.effective_batteries +=
                    Fx::from_num(spec.battery_capacity_multiplier.max(0.0));
                totals.effective_charge_rate += Fx::from_num(spec.battery_flow_multiplier.max(0.0));
                totals.effective_discharge_rate +=
                    Fx::from_num(spec.battery_flow_multiplier.max(0.0));
            }
            ModuleKind::Turret => {
                totals.turret_count += 1;
                totals.effective_turrets += Fx::from_num(1);
            }
            ModuleKind::Shield => {
                totals.shield_count += 1;
            }
            ModuleKind::Computer => {
                totals.computer_count += 1;
            }
            ModuleKind::Cockpit => {
                totals.effective_helm = totals
                    .effective_helm
                    .max(Fx::from_num(spec.helm_multiplier.max(1.0)));
            }
            ModuleKind::Core => {
                totals.inertia_multiplier = totals
                    .inertia_multiplier
                    .max(Fx::from_num(spec.inertia_multiplier.max(1.0)));
            }
            _ => {}
        }
    }

    totals
}

pub(crate) fn spawn_runtime_ship(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    player_handles: &[PlayerHandle],
    local_handle: Option<PlayerHandle>,
    player_handle_map: &mut netcode::PlayerHandleMap,
    observed_local_player: &mut netcode::ObservedLocalPlayer,
    balance: &BalanceConfig,
    node_id: u32,
    node_name: &str,
    node_kind_name: &str,
    reward_multiplier: u32,
    ambient_heat_pressure: i32,
    ambient_electrical_pressure: i32,
    wear_penalty: u32,
) {
    let totals = accumulate_ship_variant_totals(ship);
    let movement_model = ship_movement_model_with_variant_totals(ship, &totals, balance);
    let power_model = ship_power_model_with_effective(
        ship.modules.len(),
        totals.reactor_count,
        totals.battery_count,
        totals.engine_count,
        totals.turret_count,
        totals
            .effective_reactors
            .max(Fx::from_num(totals.reactor_count.max(1))),
        totals
            .effective_batteries
            .max(Fx::from_num(totals.battery_count)),
        totals
            .effective_charge_rate
            .max(Fx::from_num(totals.battery_count.max(1))),
        totals
            .effective_discharge_rate
            .max(Fx::from_num(totals.battery_count.max(1))),
        totals
            .effective_engines
            .max(Fx::from_num(totals.engine_count.max(1))),
        totals
            .effective_turrets
            .max(Fx::from_num(totals.turret_count.max(1))),
        balance,
    );
    let (min_x, max_x, min_y, max_y) = ship.bounds().unwrap_or((0, 0, 0, 0));
    let center_x = (min_x + max_x) as f32 * 0.5;
    let center_y = (min_y + max_y) as f32 * 0.5;
    let center_x_fixed = Fx::from_num(center_x);
    let center_y_fixed = Fx::from_num(center_y);
    let inertia_radius = ship
        .modules
        .iter()
        .map(|module| module_local_position(module, center_x_fixed, center_y_fixed).length())
        .fold(Fx::from_num(96), |acc, value| {
            acc.max(value + Fx::from_num(56))
        })
        * totals.inertia_multiplier.max(Fx::from_num(1));

    let root_entity = commands
        .spawn((
            Transform::from_translation(RUNTIME_SHIP_ORIGIN),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            PlayerShip,
            ShipRoot,
            ShipInertiaField {
                radius: inertia_radius,
            },
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
                engine_power_ratio: if totals.engine_count > 0 {
                    Fx::from_num(1)
                } else {
                    Fx::from_num(0)
                },
                weapons_powered: totals.turret_count > 0,
                engines_powered: totals.engine_count > 0,
            },
            power_model,
            ShipControlState::default(),
            ShipWeaponState {
                turret_count: totals.turret_count,
                cooldown_remaining: Fx::from_num(0),
                cooldown_duration: if totals.turret_count > 0 {
                    Fx::from_num(balance.combat.player_weapon_cooldown)
                } else {
                    Fx::from_num(0)
                },
                shield_count: totals.shield_count,
            },
            ShipAutomationState {
                mode: if totals.computer_count > 0 {
                    ShipAutomationMode::BalancedOps
                } else {
                    ShipAutomationMode::Off
                },
                active: false,
                output_scale: Fx::from_num(1),
                trigger_count: 0,
                invalid_executions: 0,
                last_primary_program: None,
                last_secondary_program: None,
                recent_writes: Vec::new(),
            },
            ShipArchCommandState::default(),
            MissionState {
                node_id,
                node_name: node_name.to_string(),
                node_kind_name: node_kind_name.to_string(),
                reward_multiplier,
                ambient_heat_pressure: Fx::from_num(ambient_heat_pressure),
                ambient_electrical_pressure: Fx::from_num(ambient_electrical_pressure),
                failed: false,
                failure_reason: None,
                encounter_cleared: false,
                completed: false,
                completion_reason: None,
                salvage_collected: false,
                salvage_scrap_awarded: 0,
                return_delay_remaining: None,
                repairs_performed: 0,
                stabilizations_performed: 0,
                automation_used: false,
                automation_trigger_count: 0,
                highest_heat: Fx::from_num(0),
                hottest_module_kind: None,
                first_disabled_module_kind: None,
                recent_action: None,
                recent_action_timer: Fx::from_num(0),
                recovered_raw_salvage: 0,
                processed_repair_charge: 0,
                consumed_repair_charge: 0,
                transfer_count: 0,
                processor_cycles: 0,
                logistics_bottleneck: None,
                logistics_automation_used: false,
                lowest_player_oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
                hostile_decompression_events: 0,
                player_ship_breached: false,
                airlocks_cycled: 0,
            },
        ))
        .id();

    let interior_nodes = build_interior_nodes(ship, center_x_fixed, center_y_fixed);
    let atmosphere_tiles = build_atmosphere_tiles(ship, center_x_fixed, center_y_fixed, balance);
    let start_station = interior_nodes
        .iter()
        .find(|node| node.kind == ModuleKind::Cockpit)
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
                balance,
                center_x,
                center_y,
                center_x_fixed,
                center_y_fixed,
                wear_penalty,
                false,
            )
        })
        .collect();

    let mut spawn_nodes = interior_nodes.clone();
    spawn_nodes.sort_by_key(|node| {
        (node.local_position - start_station.local_position)
            .length_sq()
            .to_num::<i128>()
    });
    let palette = [
        Color::srgb(0.82, 0.96, 0.62),
        Color::srgb(0.62, 0.90, 0.96),
        Color::srgb(0.96, 0.86, 0.62),
        Color::srgb(0.94, 0.70, 0.82),
    ];
    let shipboard_markers: Vec<_> = player_handles
        .iter()
        .enumerate()
        .map(|(index, handle)| {
            let spawn_node = spawn_nodes.get(index).unwrap_or(&start_station);
            let mut entity_commands = commands.spawn((
                Sprite::from_color(palette[index % palette.len()], Vec2::splat(12.0)),
                Transform::from_xyz(
                    spawn_node.local_position.x.to_num::<f32>(),
                    spawn_node.local_position.y.to_num::<f32>(),
                    6.0,
                ),
                ShipboardPlayer,
                ShipboardMarker,
                PlayerHandleComponent { handle: *handle },
                PlayerShipAssignment {
                    _ship_entity: root_entity,
                },
                InternalPosition {
                    grid_x: spawn_node.grid_x,
                    grid_y: spawn_node.grid_y,
                    local_position: spawn_node.local_position,
                },
                PlayerMotionState {
                    frame: PlayerReferenceFrame::Ship(root_entity),
                    world_position: FixedVec2::from_vec2(
                        RUNTIME_SHIP_ORIGIN.truncate() + spawn_node.local_position.to_vec2(),
                    ),
                    world_velocity: FixedVec2::zero(),
                    local_position: spawn_node.local_position,
                    local_velocity: FixedVec2::zero(),
                },
                CarriedResource::default(),
                CurrentStation {
                    module_id: spawn_node.module_id,
                    kind: spawn_node.kind,
                },
                ShipboardControlState {
                    mode: ShipControlMode::Interior,
                    focus_mode: StationFocusMode::Internal,
                    focused_entity: None,
                    focused_module_id: None,
                    focused_kind: None,
                    focused_family: None,
                },
                NearbyInteraction::default(),
                HeldInteraction::default(),
                PlayerFieldState {
                    local_heat: Fx::from_num(0),
                    local_electrical: Fx::from_num(0),
                    local_oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
                    heat_danger: false,
                    electrical_danger: false,
                    oxygen_warning: false,
                    oxygen_critical: false,
                },
                PlayingCleanup,
            ));
            if Some(*handle) == local_handle {
                entity_commands.insert(ObservedLocalPlayerMarker);
            }
            let entity = entity_commands.id();
            player_handle_map.entities.insert(*handle, entity);
            if Some(*handle) == local_handle {
                observed_local_player.handle = Some(*handle);
                observed_local_player.entity = Some(entity);
            }
            entity
        })
        .collect();

    commands.entity(root_entity).insert((
        ShipInteriorMap {
            walkable_nodes: interior_nodes,
        },
        ShipAtmosphereState {
            tiles: atmosphere_tiles,
            average_oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
            minimum_oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
            venting_tiles: 0,
            decompression_reported: false,
        },
    ));
    commands.entity(root_entity).add_children(&child_entities);
    commands
        .entity(root_entity)
        .add_children(&shipboard_markers);
}

pub(crate) fn spawn_hostile_ship(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    balance: &BalanceConfig,
    spawn_position: FixedVec2,
    preferred_range: Fx,
    aggression: Fx,
    salvage_reward: u32,
) {
    let totals = accumulate_ship_variant_totals(ship);
    let movement_model = ship_movement_model_with_variant_totals(ship, &totals, balance);
    let power_model = ship_power_model_with_effective(
        ship.modules.len(),
        totals.reactor_count,
        totals.battery_count,
        totals.engine_count,
        totals.turret_count,
        totals
            .effective_reactors
            .max(Fx::from_num(totals.reactor_count.max(1))),
        totals
            .effective_batteries
            .max(Fx::from_num(totals.battery_count)),
        totals
            .effective_charge_rate
            .max(Fx::from_num(totals.battery_count.max(1))),
        totals
            .effective_discharge_rate
            .max(Fx::from_num(totals.battery_count.max(1))),
        totals
            .effective_engines
            .max(Fx::from_num(totals.engine_count.max(1))),
        totals
            .effective_turrets
            .max(Fx::from_num(totals.turret_count.max(1))),
        balance,
    );
    let (min_x, max_x, min_y, max_y) = ship.bounds().unwrap_or((0, 0, 0, 0));
    let center_x = (min_x + max_x) as f32 * 0.5;
    let center_y = (min_y + max_y) as f32 * 0.5;
    let center_x_fixed = Fx::from_num(center_x);
    let center_y_fixed = Fx::from_num(center_y);
    let inertia_radius = ship
        .modules
        .iter()
        .map(|module| module_local_position(module, center_x_fixed, center_y_fixed).length())
        .fold(Fx::from_num(96), |acc, value| {
            acc.max(value + Fx::from_num(56))
        })
        * totals.inertia_multiplier.max(Fx::from_num(1));
    let interior_nodes = build_interior_nodes(ship, center_x_fixed, center_y_fixed);
    let atmosphere_tiles = build_atmosphere_tiles(ship, center_x_fixed, center_y_fixed, balance);

    let root_entity = commands
        .spawn((
            Transform::from_xyz(
                spawn_position.x.to_num::<f32>(),
                spawn_position.y.to_num::<f32>(),
                RUNTIME_SHIP_ORIGIN.z,
            ),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::VISIBLE,
            ViewVisibility::default(),
            HostileShip,
            ShipRoot,
            ShipInertiaField {
                radius: inertia_radius,
            },
            crate::gameplay::components::HostileTarget,
            HostileShipAi {
                preferred_range,
                aggression,
                salvage_reward,
            },
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
                value: spawn_position,
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
                engine_power_ratio: if totals.engine_count > 0 {
                    Fx::from_num(1)
                } else {
                    Fx::from_num(0)
                },
                weapons_powered: totals.turret_count > 0,
                engines_powered: totals.engine_count > 0,
            },
            power_model,
            ShipControlState::default(),
            ShipWeaponState {
                turret_count: totals.turret_count,
                cooldown_remaining: Fx::from_num(balance.combat.hostile_fire_cooldown * 0.22),
                cooldown_duration: if totals.turret_count > 0 {
                    Fx::from_num(balance.combat.hostile_fire_cooldown * 0.31)
                } else {
                    Fx::from_num(0)
                },
                shield_count: totals.shield_count,
            },
            ShipArchCommandState::default(),
        ))
        .id();

    let child_entities: Vec<_> = ship
        .modules
        .iter()
        .map(|module| {
            spawn_runtime_module(
                commands,
                asset_server,
                module,
                balance,
                center_x,
                center_y,
                center_x_fixed,
                center_y_fixed,
                0,
                true,
            )
        })
        .collect();

    commands
        .entity(root_entity)
        .insert((
            ShipInteriorMap {
                walkable_nodes: interior_nodes,
            },
            ShipAtmosphereState {
                tiles: atmosphere_tiles,
                average_oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
                minimum_oxygen: Fx::from_num(balance.atmosphere.initial_tile_oxygen),
                venting_tiles: 0,
                decompression_reported: false,
            },
        ))
        .add_children(&child_entities);
}

fn ship_movement_model_with_variant_totals(
    ship: &ShipDefinition,
    totals: &ShipVariantTotals,
    balance: &BalanceConfig,
) -> crate::gameplay::components::ShipMovementModel {
    ship_movement_model_with_effective(
        ship.modules.len(),
        totals.engine_count,
        totals
            .effective_engines
            .max(Fx::from_num(totals.engine_count.max(1))),
        totals.effective_helm,
        balance,
    )
}

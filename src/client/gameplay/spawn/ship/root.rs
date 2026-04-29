use bevy::prelude::*;

use super::{interior::build_interior_nodes, modules::spawn_runtime_module};
use crate::{
    client::{
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
                PlayerFieldState,
                PlayerMotionState,
                PlayerReferenceFrame,
                PlayerShip,
                PlayerShipAssignment,
                ShipArchCommandState,
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
                count_modules,
                module_local_position,
                ship_movement_model,
                ship_power_model,
            },
        },
        state::PlayingCleanup,
    },
    ship::{ModuleKind, ShipDefinition},
};

pub(crate) fn spawn_runtime_ship(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ship: &ShipDefinition,
    balance: &BalanceConfig,
    node_id: u32,
    node_name: &str,
    node_kind_name: &str,
    reward_multiplier: u32,
    ambient_heat_pressure: i32,
    ambient_electrical_pressure: i32,
    wear_penalty: u32,
) {
    let engine_count = count_modules(ship, ModuleKind::Engine);
    let reactor_count = count_modules(ship, ModuleKind::Reactor);
    let battery_count = count_modules(ship, ModuleKind::Battery);
    let turret_count = count_modules(ship, ModuleKind::Turret);
    let computer_count = count_modules(ship, ModuleKind::Computer);
    let movement_model = ship_movement_model(ship.modules.len(), engine_count, balance);
    let power_model = ship_power_model(
        ship.modules.len(),
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
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
        });

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
                mode: ShipControlMode::Interior,
                focus_mode: StationFocusMode::Internal,
                focused_entity: None,
                focused_module_id: None,
                focused_kind: None,
                focused_family: None,
            },
            ShipWeaponState {
                turret_count,
                cooldown_remaining: Fx::from_num(0),
                cooldown_duration: if turret_count > 0 {
                    Fx::from_num(balance.combat.player_weapon_cooldown)
                } else {
                    Fx::from_num(0)
                },
            },
            ShipAutomationState {
                mode: if computer_count > 0 {
                    ShipAutomationMode::BalancedOps
                } else {
                    ShipAutomationMode::Off
                },
                active: false,
                output_scale: Fx::from_num(1),
                trigger_count: 0,
                invalid_executions: 0,
                last_primary_program: None,
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
            },
        ))
        .id();

    let interior_nodes = build_interior_nodes(ship, center_x_fixed, center_y_fixed);
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

    let shipboard_marker = commands
        .spawn((
            Sprite::from_color(Color::srgb(0.82, 0.96, 0.62), Vec2::splat(12.0)),
            Transform::from_xyz(
                RUNTIME_SHIP_ORIGIN.x + start_station.local_position.x.to_num::<f32>(),
                RUNTIME_SHIP_ORIGIN.y + start_station.local_position.y.to_num::<f32>(),
                6.0,
            ),
            ShipboardPlayer,
            ShipboardMarker,
            PlayerShipAssignment {
                _ship_entity: root_entity,
            },
            InternalPosition {
                grid_x: start_station.grid_x,
                grid_y: start_station.grid_y,
                local_position: start_station.local_position,
            },
            PlayerMotionState {
                frame: PlayerReferenceFrame::Ship(root_entity),
                world_position: FixedVec2::from_vec2(
                    RUNTIME_SHIP_ORIGIN.truncate() + start_station.local_position.to_vec2(),
                ),
                world_velocity: FixedVec2::zero(),
                local_position: start_station.local_position,
                local_velocity: FixedVec2::zero(),
            },
            CarriedResource::default(),
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

    commands.entity(root_entity).insert(ShipInteriorMap {
        walkable_nodes: interior_nodes,
    });
    commands.entity(root_entity).add_children(&child_entities);

    let _ = shipboard_marker;
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
    let engine_count = count_modules(ship, ModuleKind::Engine);
    let reactor_count = count_modules(ship, ModuleKind::Reactor);
    let battery_count = count_modules(ship, ModuleKind::Battery);
    let turret_count = count_modules(ship, ModuleKind::Turret);
    let movement_model = ship_movement_model(ship.modules.len(), engine_count, balance);
    let power_model = ship_power_model(
        ship.modules.len(),
        reactor_count,
        battery_count,
        engine_count,
        turret_count,
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
        });
    let interior_nodes = build_interior_nodes(ship, center_x_fixed, center_y_fixed);

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
            crate::client::gameplay::components::HostileTarget,
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
            ShipWeaponState {
                turret_count,
                cooldown_remaining: Fx::from_num(balance.combat.hostile_fire_cooldown * 0.22),
                cooldown_duration: if turret_count > 0 {
                    Fx::from_num(balance.combat.hostile_fire_cooldown * 0.31)
                } else {
                    Fx::from_num(0)
                },
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
        .insert(ShipInteriorMap {
            walkable_nodes: interior_nodes,
        })
        .add_children(&child_entities);
}

use bevy::prelude::*;

use super::visuals::module_visual_z;
use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    gameplay::{
        components::{
            AirlockCommandState,
            ArchComputerModule,
            ArchExecutionResult,
            DetectorCommandState,
            DetectorKind,
            DetectorModule,
            DroneStationCommandState,
            DroneStationModule,
            DroneTask,
            EngineFlameOverlay,
            EngineModule,
            HostileShipModule,
            Integrity,
            Interactable,
            JunctionCommandState,
            LumenExecutionResult,
            ManipulatorCommandState,
            ManipulatorModule,
            ModuleFieldEmitter,
            ModuleRuntimeState,
            ModuleWorkEffect,
            ModuleWorkProgressFill,
            ModuleWorkProgressRoot,
            PowerConsumer,
            PowerProducer,
            ProcessorCommandState,
            ProcessorModule,
            ProcessorRecipe,
            ReactorCommandState,
            ReactorGlowOverlay,
            ResourceInventory,
            ResourceKind,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShieldCommandState,
            StorageCommandState,
            StorageModule,
            TurretCommandState,
            TurretTopSprite,
            ValveCommandState,
            WeaponModule,
        },
        effects::{EngineFlameMaterial, ReactorGlowMaterial},
        helpers::{FixedVec2, Fx, module_integrity, module_local_position, sprite_path_for_kind},
    },
    ship::{
        ModuleKind,
        ModuleSpec,
        ModuleVariant,
        ShipModule,
        StoredProcessorRecipe,
        StoredResourceKind,
        arch::{ArchProgram, ArchProgramTemplate},
        lumen::{LumenProgram, LumenProgramTemplate},
    },
    state::PlayingCleanup,
};

/// Spawns one runtime module entity with its visuals, state, and role-specific components for encounter play.
pub(crate) fn spawn_runtime_module(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    reactor_materials: &mut Assets<ReactorGlowMaterial>,
    engine_materials: &mut Assets<EngineFlameMaterial>,
    shaders_enabled: bool,
    module: &ShipModule,
    balance: &BalanceConfig,
    center_x: f32,
    center_y: f32,
    center_x_fixed: Fx,
    center_y_fixed: Fx,
    wear_penalty: u32,
    hostile: bool,
) -> Entity {
    let local_position = module_local_position(module, center_x_fixed, center_y_fixed);
    let spec = ModuleSpec::for_module(module.kind, module.variant);
    let max_integrity = module_integrity(module.kind, module.variant);
    let applied_wear = i32::min(wear_penalty as i32, max_integrity.saturating_sub(1)).max(0);
    let mut entity = commands.spawn((
        Sprite::from_image(asset_server.load(sprite_path_for_kind(&module.kind, module.variant))),
        Transform {
            translation: Vec3::new(
                (module.grid_x as f32 - center_x) * TILE_SIZE,
                -((module.grid_y as f32) - center_y) * TILE_SIZE,
                module_visual_z(module.kind),
            ),
            rotation: Quat::from_rotation_z(
                -(module.rotation_quadrants as f32) * std::f32::consts::FRAC_PI_2,
            ),
            ..default()
        },
        RuntimeShipModule {
            module_id: module.id,
            kind: module.kind,
            variant: module.variant,
            channel: module.effective_channel(),
            grid_x: module.grid_x,
            grid_y: module.grid_y,
            rotation_quadrants: module.rotation_quadrants,
            local_position,
        },
        Integrity {
            current: max_integrity - applied_wear,
            max: max_integrity,
        },
        ModuleRuntimeState {
            current_heat: Fx::from_num(0),
            electrical_instability: Fx::from_num(0),
            sampled_heat: Fx::from_num(0),
            sampled_electrical: Fx::from_num(0),
            is_disabled: false,
            was_disabled_last_frame: false,
            needs_attention: false,
            extracted: false,
            last_interaction_age: Fx::from_num(0),
        },
        module_field_emitter(module.kind, balance),
        Interactable,
        PlayingCleanup,
    ));

    if hostile {
        entity.insert(HostileShipModule);
    }

    match module.kind {
        ModuleKind::Reactor => {
            entity.insert((
                PowerProducer { output: 10 },
                ReactorCommandState {
                    variant: module.variant,
                    reaction_rate: Fx::from_num(module.defaults.reaction_rate_milli)
                        / Fx::from_num(1000),
                    turbine_load: Fx::from_num(module.defaults.turbine_load_milli)
                        / Fx::from_num(1000),
                    confinement: Fx::from_num(0.5),
                    power_output: Fx::from_num(balance.reactor.starting_power_output),
                    fuel_remaining: Fx::from_num(balance.reactor.starting_fuel * 0.35),
                },
            ));
            entity.with_children(|parent| {
                if shaders_enabled {
                    parent.spawn((
                        Mesh2d(meshes.add(Rectangle::new(38.0, 38.0))),
                        MeshMaterial2d(reactor_materials.add(ReactorGlowMaterial::default())),
                        Transform::from_xyz(0.0, 0.0, 0.16),
                        Visibility::Hidden,
                        ReactorGlowOverlay,
                    ));
                }
                spawn_work_effect_children(parent);
            });
        }
        ModuleKind::Battery => {
            entity.insert(PowerProducer { output: 4 });
        }
        ModuleKind::Cargo => {
            entity.insert((
                StorageModule {
                    capacity: spec.storage_capacity,
                    inventory: {
                        let mut inventory = ResourceInventory::default();
                        match module.variant {
                            ModuleVariant::FuelTank => inventory.fuel = spec.storage_capacity / 2,
                            ModuleVariant::AmmoRack => {
                                inventory.ammunition = spec.storage_capacity * 2
                            }
                            ModuleVariant::O2Canister => inventory.oxygen = spec.storage_capacity,
                            _ => {}
                        }
                        inventory
                    },
                    damaged_components: Vec::new(),
                    artifacts: Vec::new(),
                    accepts_fuel: module.variant == ModuleVariant::FuelTank,
                    accepts_ammunition: module.variant == ModuleVariant::AmmoRack,
                    accepts_general: matches!(
                        module.variant,
                        ModuleVariant::GeneralCargo
                            | ModuleVariant::RawSalvageCrate
                            | ModuleVariant::RepairChargeRack
                    ),
                    accepts_oxygen: module.variant == ModuleVariant::O2Canister,
                },
                StorageCommandState {
                    allow_intake: module.defaults.storage_allow_intake,
                },
            ));
        }
        ModuleKind::Airlock => {
            let bundle = (
                AirlockCommandState {
                    open: module.defaults.airlock_open,
                },
                StorageModule {
                    capacity: spec.storage_capacity,
                    inventory: ResourceInventory::default(),
                    damaged_components: Vec::new(),
                    artifacts: Vec::new(),
                    accepts_fuel: false,
                    accepts_ammunition: false,
                    accepts_general: true,
                    accepts_oxygen: false,
                },
                StorageCommandState {
                    allow_intake: module.defaults.storage_allow_intake,
                },
                ManipulatorModule {
                    transfer_progress: Fx::from_num(0),
                    transfer_duration: Fx::from_num(
                        balance.logistics.manipulator_transfer_duration,
                    ),
                    active: false,
                    source_module_id: None,
                    target_module_id: None,
                    resource_kind: None,
                    blocked_reason: None,
                },
                ManipulatorCommandState {
                    manual_mode: module.defaults.manipulator_manual_mode,
                    transfer_enabled: module.defaults.manipulator_transfer_enabled,
                    source_module_id: Some(module.id),
                    target_module_id: None,
                    resource_kind: match module.defaults.manipulator_resource_kind {
                        StoredResourceKind::RawSalvage => ResourceKind::RawSalvage,
                        StoredResourceKind::RepairCharge => ResourceKind::RepairCharge,
                        StoredResourceKind::Fuel => ResourceKind::Fuel,
                        StoredResourceKind::Ammunition => ResourceKind::Ammunition,
                        StoredResourceKind::Oxygen => ResourceKind::Oxygen,
                    },
                },
            );
            entity.insert(bundle);
            if module.variant == ModuleVariant::DroneBay {
                entity.insert((
                    DroneStationCommandState {
                        selected_task: DroneTask::Logistics,
                    },
                    DroneStationModule {
                        max_drones: 2,
                        operational_range: Fx::from_num(
                            TILE_SIZE * balance.logistics.manipulator_range_tiles * 3.6,
                        ),
                        active_drones: 0,
                        active_tasks: 0,
                        queued_tasks: 0,
                        idle_drones: 2,
                        power_draw: Fx::from_num(0),
                        last_status: "Ready".to_string(),
                    },
                ));
            }
            entity.with_children(|parent| {
                spawn_work_effect_children(parent);
            });
        }
        ModuleKind::Engine => {
            entity.insert((
                PowerConsumer { draw: 3 },
                EngineModule {
                    thrust_multiplier: Fx::from_num(spec.engine_multiplier.max(1.0)),
                },
            ));
            entity.with_children(|parent| {
                if shaders_enabled {
                    parent.spawn((
                        Mesh2d(meshes.add(Rectangle::new(20.0, 34.0))),
                        MeshMaterial2d(engine_materials.add(EngineFlameMaterial::default())),
                        Transform::from_xyz(0.0, 40.0, -0.14),
                        Visibility::Hidden,
                        EngineFlameOverlay,
                    ));
                }
                spawn_work_effect_children(parent);
            });
        }
        ModuleKind::Computer => {
            entity.insert((
                ArchComputerModule,
                RuntimeArchComputer {
                    enabled: module.defaults.computer_enabled,
                    instruction_budget: 24,
                    program: module.arch_program.clone().unwrap_or_else(|| {
                        ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                    }),
                    last_result: ArchExecutionResult::default(),
                    lumen_program: module.lumen_program.clone().unwrap_or_else(|| {
                        LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision)
                    }),
                    last_lumen_result: LumenExecutionResult::default(),
                },
            ));
        }
        ModuleKind::Detector => {
            let (kind, tier, range, draw) = detector_profile(module.variant);
            entity.insert((
                PowerConsumer { draw },
                DetectorCommandState {
                    show_targets: true,
                    show_heat: true,
                    show_electrical: true,
                },
                DetectorModule {
                    kind,
                    tier,
                    range,
                    detected: false,
                    secondary_detected: false,
                    direction: FixedVec2::zero(),
                    distance: Fx::from_num(0),
                    magnitude: Fx::from_num(0),
                    critical: false,
                },
            ));
        }
        ModuleKind::Processor => {
            entity.insert((
                PowerConsumer { draw: 2 },
                ProcessorModule {
                    progress: Fx::from_num(0),
                    duration: Fx::from_num(
                        balance.logistics.processor_duration / spec.processor_speed_multiplier,
                    ),
                    active: false,
                    blocked_reason: None,
                    inventory: ResourceInventory::default(),
                    input_required: 2,
                    output_amount: 1,
                },
                ProcessorCommandState {
                    selected_recipe: match module.defaults.processor_recipe {
                        StoredProcessorRecipe::RepairCharge => ProcessorRecipe::RepairCharge,
                        StoredProcessorRecipe::Ammunition => ProcessorRecipe::Ammunition,
                        StoredProcessorRecipe::Fuel => ProcessorRecipe::Fuel,
                    },
                    enabled: module.defaults.processor_enabled,
                },
            ));
            entity.with_children(|parent| {
                spawn_work_effect_children(parent);
            });
        }
        ModuleKind::Turret => {
            entity
                .insert((
                    PowerConsumer { draw: 2 },
                    WeaponModule {
                        damage: spec.projectile_damage,
                        requires_ammo: spec.ammo_per_shot > 0,
                        ammo_per_shot: spec.ammo_per_shot,
                        projectile_speed_multiplier: Fx::from_num(
                            spec.projectile_speed_multiplier.max(0.1),
                        ),
                        cooldown_multiplier: Fx::from_num(spec.weapon_cooldown_multiplier),
                        automation_difficulty: Fx::from_num(
                            if module.variant == ModuleVariant::BallisticTurret {
                                1.35
                            } else {
                                1.0
                            },
                        ),
                    },
                    TurretCommandState {
                        desired_angle: Fx::from_num(0),
                        actual_angle: Fx::from_num(0),
                        fire_intent: module.defaults.turret_fire_intent,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Sprite::from_image(asset_server.load("tiles/hull/turret.png")),
                        Transform::from_xyz(0.0, 0.0, 0.2),
                        TurretTopSprite,
                    ));
                });
        }
        ModuleKind::Shield => {
            entity.insert((
                PowerConsumer { draw: 2 },
                ShieldCommandState {
                    desired_angle: Fx::from_num(0),
                    width: Fx::from_num(spec.shield_arc_degrees),
                    strength: Fx::from_num(spec.shield_capacity),
                    max_strength: Fx::from_num(spec.shield_capacity),
                    regen_rate: Fx::from_num(spec.shield_regen),
                    directional: module.variant == ModuleVariant::DirectionalShield,
                },
            ));
        }
        ModuleKind::O2Generator => {
            entity.insert((
                PowerConsumer { draw: 1 },
                StorageModule {
                    capacity: spec.storage_capacity.max(4),
                    inventory: ResourceInventory {
                        oxygen: spec.storage_capacity.max(4),
                        ..default()
                    },
                    damaged_components: Vec::new(),
                    artifacts: Vec::new(),
                    accepts_fuel: false,
                    accepts_ammunition: false,
                    accepts_general: false,
                    accepts_oxygen: true,
                },
                StorageCommandState {
                    allow_intake: module.defaults.storage_allow_intake,
                },
            ));
        }
        ModuleKind::JunctionBox => {
            entity.insert((
                PowerConsumer { draw: 1 },
                JunctionCommandState { open: true },
            ));
        }
        ModuleKind::Valve => {
            entity.insert((PowerConsumer { draw: 1 }, ValveCommandState { open: true }));
        }
        _ => {
            entity.with_children(|parent| {
                spawn_work_effect_children(parent);
            });
        }
    }

    entity.id()
}

fn spawn_work_effect_children(parent: &mut ChildSpawnerCommands<'_>) {
    parent.spawn((
        Sprite::from_color(Color::srgba(1.0, 0.72, 0.28, 0.0), Vec2::new(18.0, 18.0)),
        Transform::from_xyz(0.0, 0.0, 0.18),
        Visibility::Hidden,
        ModuleWorkEffect,
    ));
    parent.spawn((
        Sprite::from_color(Color::srgba(0.08, 0.10, 0.14, 0.80), Vec2::new(22.0, 4.0)),
        Transform::from_xyz(0.0, 21.0, 0.2),
        Visibility::Hidden,
        ModuleWorkProgressRoot,
    ));
    parent.spawn((
        Sprite::from_color(Color::srgb(0.36, 0.92, 0.72), Vec2::new(20.0, 2.0)),
        Transform::from_xyz(-10.0, 21.0, 0.21),
        Visibility::Hidden,
        ModuleWorkProgressFill,
    ));
}

fn module_field_emitter(kind: ModuleKind, balance: &BalanceConfig) -> ModuleFieldEmitter {
    match kind {
        ModuleKind::Reactor => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_reactor_heat),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_reactor_electrical),
            grounding_output: Fx::from_num(balance.fields.emitter_reactor_grounding),
        },
        ModuleKind::Engine => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_engine_heat),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_engine_electrical),
            grounding_output: Fx::from_num(balance.fields.emitter_engine_grounding),
        },
        ModuleKind::Turret => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_turret_heat),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_turret_electrical),
            grounding_output: Fx::from_num(balance.fields.emitter_turret_grounding),
        },
        ModuleKind::Battery => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_battery_heat),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_battery_electrical),
            grounding_output: Fx::from_num(balance.fields.emitter_battery_grounding),
        },
        ModuleKind::Computer => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_computer_heat),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_computer_electrical),
            grounding_output: Fx::from_num(balance.fields.emitter_computer_grounding),
        },
        ModuleKind::Detector => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_computer_heat) * Fx::from_num(0.45),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_computer_electrical)
                * Fx::from_num(0.6),
            grounding_output: Fx::from_num(balance.fields.emitter_computer_grounding),
        },
        ModuleKind::Processor => ModuleFieldEmitter {
            heat_output: Fx::from_num(balance.fields.emitter_processor_heat),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(balance.fields.emitter_processor_electrical),
            grounding_output: Fx::from_num(balance.fields.emitter_processor_grounding),
        },
        ModuleKind::Hull
        | ModuleKind::HullInnerCorner
        | ModuleKind::HullOuterCorner
        | ModuleKind::Airlock => ModuleFieldEmitter {
            heat_output: Fx::from_num(0),
            cooling_output: Fx::from_num(balance.fields.emitter_hull_cooling),
            electrical_output: Fx::from_num(0),
            grounding_output: Fx::from_num(balance.fields.emitter_hull_grounding),
        },
        ModuleKind::Shield => ModuleFieldEmitter {
            heat_output: Fx::from_num(0.6),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(1.2),
            grounding_output: Fx::from_num(0.5),
        },
        ModuleKind::Core
        | ModuleKind::Cockpit
        | ModuleKind::Cargo
        | ModuleKind::Interior
        | ModuleKind::InteriorWall
        | ModuleKind::JunctionBox
        | ModuleKind::Valve
        | ModuleKind::O2Generator => ModuleFieldEmitter {
            heat_output: Fx::from_num(0),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(0),
            grounding_output: Fx::from_num(balance.fields.emitter_generic_grounding),
        },
    }
}

fn detector_profile(variant: ModuleVariant) -> (DetectorKind, u8, Fx, i32) {
    match variant {
        ModuleVariant::LifePulse => (DetectorKind::LifeSign, 1, Fx::from_num(170), 1),
        ModuleVariant::LifeSweep => (DetectorKind::LifeSign, 2, Fx::from_num(260), 2),
        ModuleVariant::LifeSurvey => (DetectorKind::LifeSign, 3, Fx::from_num(360), 2),
        ModuleVariant::ShipPing => (DetectorKind::Ship, 1, Fx::from_num(260), 1),
        ModuleVariant::ShipVector => (DetectorKind::Ship, 2, Fx::from_num(360), 2),
        ModuleVariant::ShipSurvey => (DetectorKind::Ship, 3, Fx::from_num(480), 2),
        ModuleVariant::DamageAlarm => (DetectorKind::Damage, 1, Fx::from_num(0), 1),
        ModuleVariant::DamageArray => (DetectorKind::Damage, 2, Fx::from_num(0), 2),
        ModuleVariant::StructuralSurveyor => (DetectorKind::Damage, 3, Fx::from_num(0), 2),
        ModuleVariant::PowerMonitor => (DetectorKind::Power, 1, Fx::from_num(0), 1),
        ModuleVariant::HeatMonitor => (DetectorKind::Heat, 1, Fx::from_num(0), 1),
        ModuleVariant::LogisticsBeacon => (DetectorKind::Logistics, 1, Fx::from_num(0), 1),
        _ => (DetectorKind::LifeSign, 1, Fx::from_num(170), 1),
    }
}

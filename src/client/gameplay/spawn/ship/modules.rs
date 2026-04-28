use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::client::gameplay::{
    components::{
        ArchComputerModule,
        ArchExecutionResult,
        EngineModule,
        Integrity,
        Interactable,
        ManipulatorCommandState,
        ManipulatorModule,
        ModuleFieldEmitter,
        ModuleRuntimeState,
        PowerConsumer,
        PowerProducer,
        ProcessorCommandState,
        ProcessorRecipe,
        ProcessorModule,
        ReactorCommandState,
        ResourceInventory,
        RuntimeArchComputer,
        RuntimeShipModule,
        StorageCommandState,
        TurretCommandState,
        StorageModule,
        TurretTopSprite,
        WeaponModule,
    },
    helpers::{
        module_integrity,
        module_local_position,
        module_local_translation,
        sprite_path_for_kind,
        Fx,
    },
};
use crate::ship::{ModuleKind, ShipModule};

pub(super) fn spawn_runtime_module(
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
            was_disabled_last_frame: false,
            needs_attention: false,
            last_interaction_age: Fx::from_num(0),
        },
        module_field_emitter(module.kind),
        Interactable,
        crate::client::state::PlayingCleanup,
    ));

    match module.kind {
        ModuleKind::Reactor => {
            entity.insert((
                PowerProducer { output: 10 },
                ReactorCommandState {
                    reaction_rate: Fx::from_num(0.5),
                    turbine_load: Fx::from_num(0.5),
                    power_output: Fx::from_num(4),
                    fuel_remaining: Fx::from_num(100),
                },
            ));
        }
        ModuleKind::Battery => {
            entity.insert(PowerProducer { output: 4 });
        }
        ModuleKind::Cargo => {
            entity.insert((
                StorageModule {
                    capacity: 8,
                    inventory: ResourceInventory::default(),
                },
                StorageCommandState { allow_intake: true },
            ));
        }
        ModuleKind::Airlock => {
            entity.insert((
                StorageModule {
                    capacity: 4,
                    inventory: ResourceInventory::default(),
                },
                StorageCommandState { allow_intake: true },
                ManipulatorModule {
                    transfer_progress: Fx::from_num(0),
                    transfer_duration: Fx::from_num(0.75),
                    active: false,
                    source_module_id: None,
                    target_module_id: None,
                    resource_kind: None,
                    blocked_reason: None,
                },
                ManipulatorCommandState {
                    manual_mode: false,
                    transfer_enabled: false,
                    source_module_id: Some(module.id),
                    target_module_id: None,
                    resource_kind: crate::client::gameplay::components::ResourceKind::RawSalvage,
                },
            ));
        }
        ModuleKind::Engine => {
            entity.insert((PowerConsumer { draw: 3 }, EngineModule));
        }
        ModuleKind::Computer => {
            entity.insert((
                ArchComputerModule,
                RuntimeArchComputer {
                    enabled: true,
                    instruction_budget: 24,
                    program: module
                        .arch_program
                        .clone()
                        .unwrap_or_else(|| {
                            crate::ship::arch::ArchProgram::from_template(
                                crate::ship::arch::ArchProgramTemplate::BalancedOps,
                            )
                        }),
                    last_result: ArchExecutionResult::default(),
                },
            ));
        }
        ModuleKind::Processor => {
            entity.insert((
                PowerConsumer { draw: 2 },
                ProcessorModule {
                    progress: Fx::from_num(0),
                    duration: Fx::from_num(2.2),
                    active: false,
                    blocked_reason: None,
                    inventory: ResourceInventory::default(),
                    input_required: 2,
                    output_amount: 1,
                },
                ProcessorCommandState {
                    selected_recipe: ProcessorRecipe::RepairCharge,
                    enabled: true,
                },
            ));
        }
        ModuleKind::Turret => {
            entity
                .insert((
                    PowerConsumer { draw: 2 },
                    WeaponModule,
                    TurretCommandState {
                        desired_angle: Fx::from_num(0),
                        actual_angle: Fx::from_num(0),
                        fire_intent: false,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Sprite::from_image(asset_server.load("tiles/turret.png")),
                        Transform::from_xyz(0.0, 0.0, 0.2),
                        TurretTopSprite,
                    ));
                });
        }
        _ => {}
    }

    entity.id()
}

fn module_field_emitter(kind: ModuleKind) -> ModuleFieldEmitter {
    match kind {
        ModuleKind::Reactor => ModuleFieldEmitter {
            heat_output: Fx::from_num(1),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(0.5),
            grounding_output: Fx::from_num(0.2),
        },
        ModuleKind::Engine => ModuleFieldEmitter {
            heat_output: Fx::from_num(1),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(0.5),
            grounding_output: Fx::from_num(0.2),
        },
        ModuleKind::Turret => ModuleFieldEmitter {
            heat_output: Fx::from_num(2),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(1),
            grounding_output: Fx::from_num(0.2),
        },
        ModuleKind::Battery => ModuleFieldEmitter {
            heat_output: Fx::from_num(0.5),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(2),
            grounding_output: Fx::from_num(1.4),
        },
        ModuleKind::Computer => ModuleFieldEmitter {
            heat_output: Fx::from_num(0.3),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(0.4),
            grounding_output: Fx::from_num(1.6),
        },
        ModuleKind::Processor => ModuleFieldEmitter {
            heat_output: Fx::from_num(0.8),
            cooling_output: Fx::from_num(0),
            electrical_output: Fx::from_num(0.4),
            grounding_output: Fx::from_num(0.8),
        },
        ModuleKind::Hull | ModuleKind::HullCorner | ModuleKind::Airlock => ModuleFieldEmitter {
            heat_output: Fx::from_num(0),
            cooling_output: Fx::from_num(2),
            electrical_output: Fx::from_num(0),
            grounding_output: Fx::from_num(2.8),
        },
        ModuleKind::Core | ModuleKind::Cockpit | ModuleKind::Cargo | ModuleKind::Interior => {
            ModuleFieldEmitter {
                heat_output: Fx::from_num(0),
                cooling_output: Fx::from_num(0),
                electrical_output: Fx::from_num(0),
                grounding_output: Fx::from_num(0.8),
            }
        }
    }
}

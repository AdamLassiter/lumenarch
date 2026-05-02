use bevy::prelude::*;
use cordic::{acos, asin, atan, atan2, cos, sin, sqrt, tan};

use crate::{
    gameplay::{
        components::{
            ArchComputerModule,
            ArchExecutionResult,
            ArchLogisticsPreference,
            DestroyedModule,
            HostileTarget,
            MissionState,
            ModuleRuntimeState,
            PlayerShip,
            ProcessorModule,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShipArchCommandState,
            ShipAutomationMode,
            ShipAutomationState,
            ShipRoot,
            StorageModule,
        },
        helpers::{Fx, fx_from_time_delta},
    },
    ship::{
        ModuleKind,
        arch::{ArchInstruction, ArchProgram, ArchProgramTemplate, ArchRegister, ArchValueRef},
    },
};

#[derive(Clone, Copy)]
struct ArchSnapshot {
    ship_power_reserve: Fx,
    ship_average_heat: Fx,
    mission_threat: Fx,
    reactor_heat: Fx,
    reactor_instability: Fx,
    storage_raw_salvage: Fx,
    storage_repair_charge: Fx,
    processor_raw_salvage: Fx,
    processor_repair_charge: Fx,
    turret_ready: Fx,
    turret_cooldown: Fx,
}

#[derive(Default)]
struct PendingArchCommands {
    reactor_bias: Fx,
    logistics_enabled: bool,
    logistics_preference: ArchLogisticsPreference,
    turret_assist: bool,
    turret_auto_fire: bool,
}

pub(crate) fn run_arch_automation(
    time: Res<Time>,
    ship_query: Single<
        (
            &Children,
            &super::super::super::components::ShipPowerState,
            &super::super::super::components::ShipWeaponState,
            &mut ShipArchCommandState,
            &mut ShipAutomationState,
            &mut MissionState,
        ),
        (With<PlayerShip>, With<ShipRoot>),
    >,
    hostile_query: Query<Entity, With<HostileTarget>>,
    mut module_query: Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);
    let (
        children,
        ship_power_state,
        ship_weapon_state,
        mut command_state,
        mut automation_state,
        mut mission_state,
    ) = ship_query.into_inner();

    *command_state = ShipArchCommandState::default();
    automation_state.active = false;
    automation_state.output_scale = Fx::from_num(1);
    automation_state.invalid_executions = 0;
    automation_state.last_primary_program = None;
    automation_state.recent_writes.clear();

    let snapshot = build_snapshot(
        children,
        ship_power_state,
        ship_weapon_state,
        !hostile_query.is_empty(),
        &module_query,
    );
    let mut aggregate = PendingArchCommands::default();
    let mut had_active_computer = false;
    let mut trigger_delta = 0u32;

    for child in children.iter() {
        let Ok((
            _,
            runtime_module,
            mut runtime_state,
            computer_marker,
            arch_runtime,
            _storage,
            _processor,
            destroyed,
        )) = module_query.get_mut(child)
        else {
            continue;
        };
        let Some(_) = computer_marker else {
            continue;
        };
        let Some(mut arch_runtime) = arch_runtime else {
            continue;
        };
        if destroyed.is_some() || runtime_state.is_disabled || !arch_runtime.enabled {
            arch_runtime.last_result = ArchExecutionResult {
                executed: 0,
                budget: arch_runtime.instruction_budget,
                recent_writes: Vec::new(),
                halted_reason: Some("offline".to_string()),
                program_name: arch_runtime.program.name.clone(),
            };
            continue;
        }

        had_active_computer = true;
        if automation_state.last_primary_program.is_none() {
            automation_state.last_primary_program = Some(arch_runtime.program.name.clone());
            automation_state.mode = template_to_mode(arch_runtime.program.template);
        } else if automation_state.mode != template_to_mode(arch_runtime.program.template) {
            automation_state.mode = ShipAutomationMode::Mixed;
        }

        let was_active = aggregate.reactor_bias > Fx::from_num(0)
            || aggregate.logistics_enabled
            || aggregate.turret_assist
            || aggregate.turret_auto_fire;
        let (result, outputs) = execute_program(
            &arch_runtime.program,
            arch_runtime.instruction_budget,
            snapshot,
        );
        let now_active = outputs.reactor_bias > Fx::from_num(0)
            || outputs.logistics_enabled
            || outputs.turret_assist
            || outputs.turret_auto_fire;
        if now_active && !was_active {
            trigger_delta += 1;
        }
        aggregate.reactor_bias = aggregate.reactor_bias.max(outputs.reactor_bias);
        aggregate.logistics_enabled |= outputs.logistics_enabled;
        if outputs.logistics_enabled {
            aggregate.logistics_preference = outputs.logistics_preference;
        }
        aggregate.turret_assist |= outputs.turret_assist;
        aggregate.turret_auto_fire |= outputs.turret_auto_fire;
        if result.halted_reason.is_some() {
            automation_state.invalid_executions += 1;
        }
        if automation_state.recent_writes.is_empty() && !result.recent_writes.is_empty() {
            automation_state.recent_writes = result.recent_writes.clone();
        }
        arch_runtime.last_result = result;

        if runtime_module.kind == ModuleKind::Computer && aggregate.reactor_bias > Fx::from_num(0) {
            runtime_state.current_heat =
                (runtime_state.current_heat - Fx::from_num(0.4) * dt).max(Fx::from_num(0));
        }
    }

    if !had_active_computer {
        automation_state.mode = ShipAutomationMode::Off;
        return;
    }

    command_state.reactor_cooling_bias = aggregate.reactor_bias;
    command_state.logistics_enabled = aggregate.logistics_enabled;
    command_state.logistics_preference = aggregate.logistics_preference;
    command_state.turret_assist_enabled = aggregate.turret_assist;
    command_state.turret_auto_fire = aggregate.turret_auto_fire;

    automation_state.active = aggregate.reactor_bias > Fx::from_num(0)
        || aggregate.logistics_enabled
        || aggregate.turret_assist
        || aggregate.turret_auto_fire;
    if aggregate.reactor_bias > Fx::from_num(0) {
        automation_state.output_scale = Fx::from_num(0.72);
    }
    if automation_state.active {
        automation_state.trigger_count += trigger_delta.max(1);
        mission_state.automation_used = true;
        mission_state.automation_trigger_count = automation_state.trigger_count;
        mission_state.logistics_automation_used |= aggregate.logistics_enabled;
        mission_state.recent_action = Some(format!(
            "ARCH active: {}",
            automation_state
                .last_primary_program
                .as_deref()
                .unwrap_or("program")
        ));
        mission_state.recent_action_timer = Fx::from_num(1.4);
    }

    if aggregate.reactor_bias > Fx::from_num(0) {
        for child in children.iter() {
            let Ok((_, runtime_module, mut runtime_state, _, _, _, _, destroyed)) =
                module_query.get_mut(child)
            else {
                continue;
            };
            if destroyed.is_some() || runtime_module.kind != ModuleKind::Reactor {
                continue;
            }
            runtime_state.current_heat = (runtime_state.current_heat
                - aggregate.reactor_bias * Fx::from_num(0.75) * dt)
                .max(Fx::from_num(0));
            runtime_state.electrical_instability = (runtime_state.electrical_instability
                - aggregate.reactor_bias * Fx::from_num(0.9) * dt)
                .max(Fx::from_num(0));
            if runtime_state.current_heat <= Fx::from_num(8)
                && runtime_state.electrical_instability <= Fx::from_num(6)
            {
                runtime_state.is_disabled = false;
                runtime_state.needs_attention = false;
            }
        }
    }
}

pub(crate) fn tick_recent_action_feedback(
    time: Res<Time>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    let mut mission_state = mission_query.into_inner();
    if mission_state.recent_action_timer <= Fx::from_num(0) {
        return;
    }
    mission_state.recent_action_timer =
        (mission_state.recent_action_timer - fx_from_time_delta(&time)).max(Fx::from_num(0));
    if mission_state.recent_action_timer == Fx::from_num(0) {
        mission_state.recent_action = None;
    }
}

fn build_snapshot(
    children: &Children,
    ship_power_state: &super::super::super::components::ShipPowerState,
    ship_weapon_state: &super::super::super::components::ShipWeaponState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&DestroyedModule>,
    )>,
) -> ArchSnapshot {
    let mut heat_sum = Fx::from_num(0);
    let mut live_modules = 0i32;
    let mut reactor_heat = Fx::from_num(0);
    let mut reactor_instability = Fx::from_num(0);
    let mut storage_raw = Fx::from_num(0);
    let mut storage_charge = Fx::from_num(0);
    let mut processor_raw = Fx::from_num(0);
    let mut processor_charge = Fx::from_num(0);

    for child in children.iter() {
        let Ok((_, runtime_module, runtime_state, _, _, storage, processor, destroyed)) =
            module_query.get(child)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        heat_sum += runtime_state.current_heat;
        live_modules += 1;
        match runtime_module.kind {
            ModuleKind::Reactor => {
                reactor_heat = reactor_heat.max(runtime_state.current_heat);
                reactor_instability = reactor_instability.max(runtime_state.electrical_instability);
            }
            ModuleKind::Cargo | ModuleKind::Airlock => {
                if let Some(storage) = storage {
                    storage_raw += Fx::from_num(storage.inventory.raw_salvage as i32);
                    storage_charge += Fx::from_num(storage.inventory.repair_charge as i32);
                }
            }
            ModuleKind::Processor => {
                if let Some(processor) = processor {
                    processor_raw += Fx::from_num(processor.inventory.raw_salvage as i32);
                    processor_charge += Fx::from_num(processor.inventory.repair_charge as i32);
                }
            }
            _ => {}
        }
    }

    let avg_heat = if live_modules > 0 {
        heat_sum / Fx::from_num(live_modules)
    } else {
        Fx::from_num(0)
    };

    // These values are later overwritten by system-specific queries in execute_program context.
    ArchSnapshot {
        ship_power_reserve: ship_power_state.stored_energy,
        ship_average_heat: avg_heat,
        mission_threat: if threat_present {
            Fx::from_num(1)
        } else {
            Fx::from_num(0)
        },
        reactor_heat,
        reactor_instability,
        storage_raw_salvage: storage_raw,
        storage_repair_charge: storage_charge,
        processor_raw_salvage: processor_raw,
        processor_repair_charge: processor_charge,
        turret_ready: if ship_weapon_state.turret_count > 0 {
            Fx::from_num(1)
        } else {
            Fx::from_num(0)
        },
        turret_cooldown: ship_weapon_state.cooldown_remaining.max(Fx::from_num(0)),
    }
}

fn execute_program(
    program: &ArchProgram,
    budget: u32,
    mut snapshot: ArchSnapshot,
) -> (ArchExecutionResult, PendingArchCommands) {
    let mut gp = [Fx::from_num(0); 4];
    let mut writes = Vec::new();
    let mut commands = PendingArchCommands::default();
    let mut pc = 0usize;
    let mut executed = 0u32;
    let mut halted_reason = None;

    while pc < program.instructions.len() && executed < budget {
        let instruction = &program.instructions[pc];
        let step = execute_instruction(
            instruction,
            &mut snapshot,
            &mut gp,
            &mut commands,
            &mut writes,
            pc,
        );
        executed += 1;
        match step {
            Ok(next_pc) => {
                pc = next_pc;
            }
            Err(reason) => {
                halted_reason = Some(reason);
                break;
            }
        }
    }

    if halted_reason.is_none() && executed >= budget && pc < program.instructions.len() {
        halted_reason = Some("budget exhausted".to_string());
    }

    (
        ArchExecutionResult {
            executed,
            budget,
            recent_writes: writes.into_iter().take(4).collect(),
            halted_reason,
            program_name: program.name.clone(),
        },
        commands,
    )
}

fn execute_instruction(
    instruction: &ArchInstruction,
    snapshot: &mut ArchSnapshot,
    gp: &mut [Fx; 4],
    commands: &mut PendingArchCommands,
    writes: &mut Vec<String>,
    pc: usize,
) -> Result<usize, String> {
    match instruction {
        ArchInstruction::Nop => Ok(pc + 1),
        ArchInstruction::Mov { dst, src } => {
            let value = resolve_value(src, snapshot, gp, commands);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Clp {
            dst,
            value,
            min,
            max,
        } => {
            let value = resolve_value(value, snapshot, gp, commands);
            let min = resolve_value(min, snapshot, gp, commands);
            let max = resolve_value(max, snapshot, gp, commands);
            let (min, max) = if min <= max { (min, max) } else { (max, min) };
            write_register(*dst, value.clamp(min, max), gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Abs { dst, src } => {
            let value = resolve_value(src, snapshot, gp, commands).abs();
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Neg { dst, src } => {
            let value = -resolve_value(src, snapshot, gp, commands);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Add { dst, lhs, rhs } => {
            let value = resolve_value(lhs, snapshot, gp, commands)
                + resolve_value(rhs, snapshot, gp, commands);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Sub { dst, lhs, rhs } => {
            let value = resolve_value(lhs, snapshot, gp, commands)
                - resolve_value(rhs, snapshot, gp, commands);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Mul { dst, lhs, rhs } => {
            let value = resolve_value(lhs, snapshot, gp, commands)
                * resolve_value(rhs, snapshot, gp, commands);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Div { dst, lhs, rhs } => {
            let rhs = resolve_value(rhs, snapshot, gp, commands);
            if rhs == Fx::from_num(0) {
                return Err("division by zero".to_string());
            }
            let value = resolve_value(lhs, snapshot, gp, commands) / rhs;
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Mod { dst, lhs, rhs } => {
            let rhs = resolve_value(rhs, snapshot, gp, commands);
            if rhs == Fx::from_num(0) {
                return Err("modulo by zero".to_string());
            }
            let value = resolve_value(lhs, snapshot, gp, commands) % rhs;
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Pow { dst, lhs, rhs } => {
            let lhs = resolve_value(lhs, snapshot, gp, commands);
            let rhs = resolve_value(rhs, snapshot, gp, commands);
            let value = pow_fx(lhs, rhs)?;
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Sqrt { dst, src } => {
            let src = resolve_value(src, snapshot, gp, commands);
            if src < Fx::from_num(0) {
                return Err("sqrt of negative value".to_string());
            }
            write_register(*dst, sqrt(src), gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Log { dst, src } => {
            let src = resolve_value(src, snapshot, gp, commands);
            let value = log_fx(src)?;
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Sin { dst, src } => {
            let value = sin(resolve_value(src, snapshot, gp, commands));
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Cos { dst, src } => {
            let value = cos(resolve_value(src, snapshot, gp, commands));
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Tan { dst, src } => {
            let src = resolve_value(src, snapshot, gp, commands);
            let value = tan(src);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Asin { dst, src } => {
            let src = resolve_value(src, snapshot, gp, commands);
            if !(-Fx::from_num(1)..=Fx::from_num(1)).contains(&src) {
                return Err("asin domain error".to_string());
            }
            write_register(*dst, asin(src), gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Acos { dst, src } => {
            let src = resolve_value(src, snapshot, gp, commands);
            if !(-Fx::from_num(1)..=Fx::from_num(1)).contains(&src) {
                return Err("acos domain error".to_string());
            }
            write_register(*dst, acos(src), gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Atan { dst, src } => {
            let value = atan(resolve_value(src, snapshot, gp, commands));
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Atn2 { dst, y, x } => {
            let value = atan2(
                resolve_value(y, snapshot, gp, commands),
                resolve_value(x, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Gt { dst, lhs, rhs } => {
            let value = bool_fx(
                resolve_value(lhs, snapshot, gp, commands)
                    > resolve_value(rhs, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Gte { dst, lhs, rhs } => {
            let value = bool_fx(
                resolve_value(lhs, snapshot, gp, commands)
                    >= resolve_value(rhs, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Lt { dst, lhs, rhs } => {
            let value = bool_fx(
                resolve_value(lhs, snapshot, gp, commands)
                    < resolve_value(rhs, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Lte { dst, lhs, rhs } => {
            let value = bool_fx(
                resolve_value(lhs, snapshot, gp, commands)
                    <= resolve_value(rhs, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Eq { dst, lhs, rhs } => {
            let value = bool_fx(
                resolve_value(lhs, snapshot, gp, commands)
                    == resolve_value(rhs, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Neq { dst, lhs, rhs } => {
            let value = bool_fx(
                resolve_value(lhs, snapshot, gp, commands)
                    != resolve_value(rhs, snapshot, gp, commands),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::And { dst, lhs, rhs } => {
            let value = bool_fx(
                is_truthy(resolve_value(lhs, snapshot, gp, commands))
                    && is_truthy(resolve_value(rhs, snapshot, gp, commands)),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Or { dst, lhs, rhs } => {
            let value = bool_fx(
                is_truthy(resolve_value(lhs, snapshot, gp, commands))
                    || is_truthy(resolve_value(rhs, snapshot, gp, commands)),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Xor { dst, lhs, rhs } => {
            let value = bool_fx(
                is_truthy(resolve_value(lhs, snapshot, gp, commands))
                    ^ is_truthy(resolve_value(rhs, snapshot, gp, commands)),
            );
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Not { dst, src } => {
            let value = bool_fx(!is_truthy(resolve_value(src, snapshot, gp, commands)));
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Jmp { target } => jump_target(*target, pc),
        ArchInstruction::Jeq { lhs, rhs, target } => jump_if(
            resolve_value(lhs, snapshot, gp, commands)
                == resolve_value(rhs, snapshot, gp, commands),
            *target,
            pc,
        ),
        ArchInstruction::Jne { lhs, rhs, target } => jump_if(
            resolve_value(lhs, snapshot, gp, commands)
                != resolve_value(rhs, snapshot, gp, commands),
            *target,
            pc,
        ),
        ArchInstruction::Jgt { lhs, rhs, target } => jump_if(
            resolve_value(lhs, snapshot, gp, commands) > resolve_value(rhs, snapshot, gp, commands),
            *target,
            pc,
        ),
        ArchInstruction::Jge { lhs, rhs, target } => jump_if(
            resolve_value(lhs, snapshot, gp, commands)
                >= resolve_value(rhs, snapshot, gp, commands),
            *target,
            pc,
        ),
        ArchInstruction::Jlt { lhs, rhs, target } => jump_if(
            resolve_value(lhs, snapshot, gp, commands) < resolve_value(rhs, snapshot, gp, commands),
            *target,
            pc,
        ),
        ArchInstruction::Jle { lhs, rhs, target } => jump_if(
            resolve_value(lhs, snapshot, gp, commands)
                <= resolve_value(rhs, snapshot, gp, commands),
            *target,
            pc,
        ),
        ArchInstruction::Jnz { cond, target } => jump_if(
            is_truthy(resolve_value(cond, snapshot, gp, commands)),
            *target,
            pc,
        ),
        ArchInstruction::Min { dst, lhs, rhs } => {
            let value = resolve_value(lhs, snapshot, gp, commands)
                .min(resolve_value(rhs, snapshot, gp, commands));
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Max { dst, lhs, rhs } => {
            let value = resolve_value(lhs, snapshot, gp, commands)
                .max(resolve_value(rhs, snapshot, gp, commands));
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Avg { dst, lhs, rhs } => {
            let value = (resolve_value(lhs, snapshot, gp, commands)
                + resolve_value(rhs, snapshot, gp, commands))
                / Fx::from_num(2);
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Sgn { dst, src } => {
            let value = match resolve_value(src, snapshot, gp, commands).cmp(&Fx::from_num(0)) {
                std::cmp::Ordering::Less => Fx::from_num(-1),
                std::cmp::Ordering::Equal => Fx::from_num(0),
                std::cmp::Ordering::Greater => Fx::from_num(1),
            };
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
        ArchInstruction::Lerp {
            dst,
            start,
            end,
            factor,
        } => {
            let start = resolve_value(start, snapshot, gp, commands);
            let end = resolve_value(end, snapshot, gp, commands);
            let factor = resolve_value(factor, snapshot, gp, commands);
            let value = start + (end - start) * factor;
            write_register(*dst, value, gp, commands, writes)?;
            Ok(pc + 1)
        }
    }
}

fn bool_fx(value: bool) -> Fx {
    if value {
        Fx::from_num(1)
    } else {
        Fx::from_num(0)
    }
}

fn is_truthy(value: Fx) -> bool {
    value != Fx::from_num(0)
}

fn jump_target(target: usize, pc: usize) -> Result<usize, String> {
    if target <= pc {
        return Err("backward jump rejected".to_string());
    }
    Ok(target)
}

fn jump_if(condition: bool, target: usize, pc: usize) -> Result<usize, String> {
    if condition {
        jump_target(target, pc)
    } else {
        Ok(pc + 1)
    }
}

fn pow_fx(base: Fx, exponent: Fx) -> Result<Fx, String> {
    let base_f64 = base.to_num::<f64>();
    let exponent_f64 = exponent.to_num::<f64>();
    let value = base_f64.powf(exponent_f64);
    if !value.is_finite() {
        return Err("pow produced non-finite value".to_string());
    }
    Ok(Fx::from_num(value))
}

fn log_fx(value: Fx) -> Result<Fx, String> {
    if value <= Fx::from_num(0) {
        return Err("log of non-positive value".to_string());
    }
    let result = value.to_num::<f64>().ln();
    if !result.is_finite() {
        return Err("log produced non-finite value".to_string());
    }
    Ok(Fx::from_num(result))
}

fn resolve_value(
    value: &ArchValueRef,
    snapshot: &ArchSnapshot,
    gp: &[Fx; 4],
    commands: &PendingArchCommands,
) -> Fx {
    match value {
        ArchValueRef::Immediate(value) => Fx::from_num(*value),
        ArchValueRef::Register(register) => read_register(*register, snapshot, gp, commands),
    }
}

fn read_register(
    register: ArchRegister,
    snapshot: &ArchSnapshot,
    gp: &[Fx; 4],
    commands: &PendingArchCommands,
) -> Fx {
    match register {
        ArchRegister::Gp0 => gp[0],
        ArchRegister::Gp1 => gp[1],
        ArchRegister::Gp2 => gp[2],
        ArchRegister::Gp3 => gp[3],
        ArchRegister::ShipPowerReserve => snapshot.ship_power_reserve,
        ArchRegister::ShipAverageHeat => snapshot.ship_average_heat,
        ArchRegister::MissionThreat => snapshot.mission_threat,
        ArchRegister::ReactorHeat => snapshot.reactor_heat,
        ArchRegister::ReactorInstability => snapshot.reactor_instability,
        ArchRegister::StorageRawSalvage => snapshot.storage_raw_salvage,
        ArchRegister::StorageRepairCharge => snapshot.storage_repair_charge,
        ArchRegister::ProcessorRawSalvage => snapshot.processor_raw_salvage,
        ArchRegister::ProcessorRepairCharge => snapshot.processor_repair_charge,
        ArchRegister::TurretReady => snapshot.turret_ready,
        ArchRegister::TurretCooldown => snapshot.turret_cooldown,
        ArchRegister::CmdReactorBias => commands.reactor_bias,
        ArchRegister::CmdLogisticsEnable => Fx::from_num(i32::from(commands.logistics_enabled)),
        ArchRegister::CmdLogisticsPreference => Fx::from_num(match commands.logistics_preference {
            ArchLogisticsPreference::FeedProcessor => 0,
            ArchLogisticsPreference::StoreCharges => 1,
        }),
        ArchRegister::CmdTurretAssist => Fx::from_num(i32::from(commands.turret_assist)),
        ArchRegister::CmdTurretAutoFire => Fx::from_num(i32::from(commands.turret_auto_fire)),
    }
}

fn write_register(
    register: ArchRegister,
    value: Fx,
    gp: &mut [Fx; 4],
    commands: &mut PendingArchCommands,
    writes: &mut Vec<String>,
) -> Result<(), String> {
    if !register.is_writable() {
        return Err(format!("register {} is read-only", register.as_str()));
    }

    match register {
        ArchRegister::Gp0 => gp[0] = value,
        ArchRegister::Gp1 => gp[1] = value,
        ArchRegister::Gp2 => gp[2] = value,
        ArchRegister::Gp3 => gp[3] = value,
        ArchRegister::CmdReactorBias => {
            commands.reactor_bias = value.clamp(Fx::from_num(0), Fx::from_num(3));
            writes.push(format!(
                "{} <- {:.1}",
                register.as_str(),
                value.to_num::<f32>()
            ));
        }
        ArchRegister::CmdLogisticsEnable => {
            commands.logistics_enabled = value > Fx::from_num(0);
            writes.push(format!(
                "{} <- {}",
                register.as_str(),
                if commands.logistics_enabled {
                    "on"
                } else {
                    "off"
                }
            ));
        }
        ArchRegister::CmdLogisticsPreference => {
            commands.logistics_preference = if value >= Fx::from_num(1) {
                ArchLogisticsPreference::StoreCharges
            } else {
                ArchLogisticsPreference::FeedProcessor
            };
            writes.push(format!(
                "{} <- {}",
                register.as_str(),
                match commands.logistics_preference {
                    ArchLogisticsPreference::FeedProcessor => "feed",
                    ArchLogisticsPreference::StoreCharges => "store",
                }
            ));
        }
        ArchRegister::CmdTurretAssist => {
            commands.turret_assist = value > Fx::from_num(0);
            writes.push(format!(
                "{} <- {}",
                register.as_str(),
                if commands.turret_assist { "on" } else { "off" }
            ));
        }
        ArchRegister::CmdTurretAutoFire => {
            commands.turret_auto_fire = value > Fx::from_num(0);
            writes.push(format!(
                "{} <- {}",
                register.as_str(),
                if commands.turret_auto_fire {
                    "on"
                } else {
                    "off"
                }
            ));
        }
        _ => {}
    }

    Ok(())
}

fn template_to_mode(template: ArchProgramTemplate) -> ShipAutomationMode {
    match template {
        ArchProgramTemplate::ReactorGuard => ShipAutomationMode::ReactorGuard,
        ArchProgramTemplate::LogisticsFeed => ShipAutomationMode::LogisticsFeed,
        ArchProgramTemplate::TurretAssist => ShipAutomationMode::TurretAssist,
        ArchProgramTemplate::BalancedOps => ShipAutomationMode::BalancedOps,
    }
}

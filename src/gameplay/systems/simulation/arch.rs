use bevy::prelude::*;
use cordic::{acos, asin, atan, atan2, cos, sin, sqrt, tan};

use crate::{
    gameplay::{
        components::{
            ArchComputerModule,
            ArchExecutionResult,
            ArchLogisticsPreference,
            DestroyedModule,
            DetectorModule,
            HostileTarget,
            LumenExecutionResult,
            MissionState,
            ModuleRuntimeState,
            PlayerShip,
            ProcessorModule,
            ReactorCommandState,
            RuntimeArchComputer,
            RuntimeShipModule,
            ShipArchCommandState,
            ShipAutomationMode,
            ShipAutomationState,
            ShipDamageSensorState,
            ShipPowerState,
            ShipRoot,
            ShipWeaponState,
            StorageModule,
        },
        helpers::{Fx, fx_from_time_delta},
        systems::simulation::snapshots,
    },
    ship::{
        ModuleKind,
        arch::{ArchInstruction, ArchProgram, ArchProgramTemplate, ArchRegister, ArchValueRef},
        lumen::{LumenAspect, LumenInstruction, LumenOp, LumenProgram, LumenTarget},
    },
};

#[derive(Clone, Copy)]
pub(crate) struct ArchSnapshot {
    pub(crate) ship_power_reserve: Fx,
    pub(crate) ship_average_heat: Fx,
    pub(crate) mission_threat: Fx,
    pub(crate) reactor_reaction_rate: Fx,
    pub(crate) reactor_turbine_load: Fx,
    pub(crate) reactor_heat: Fx,
    pub(crate) reactor_instability: Fx,
    pub(crate) reactor_power_output: Fx,
    pub(crate) storage_raw_salvage: Fx,
    pub(crate) storage_repair_charge: Fx,
    pub(crate) processor_raw_salvage: Fx,
    pub(crate) processor_repair_charge: Fx,
    pub(crate) turret_ready: Fx,
    pub(crate) turret_cooldown: Fx,
    pub(crate) life_friendly_present: Fx,
    pub(crate) life_hostile_present: Fx,
    pub(crate) life_dir_x: Fx,
    pub(crate) life_dir_y: Fx,
    pub(crate) life_distance: Fx,
    pub(crate) ship_nearby_present: Fx,
    pub(crate) ship_hostile_present: Fx,
    pub(crate) ship_dir_x: Fx,
    pub(crate) ship_dir_y: Fx,
    pub(crate) ship_distance: Fx,
    pub(crate) damage_incoming: Fx,
    pub(crate) damage_critical: Fx,
    pub(crate) damage_dir_x: Fx,
    pub(crate) damage_dir_y: Fx,
    pub(crate) damage_intensity: Fx,
    pub(crate) power_deficit: Fx,
    pub(crate) power_low_battery: Fx,
    pub(crate) heat_alert: Fx,
    pub(crate) heat_dir_x: Fx,
    pub(crate) heat_dir_y: Fx,
    pub(crate) heat_severity: Fx,
    pub(crate) logistics_demand: Fx,
    pub(crate) logistics_dir_x: Fx,
    pub(crate) logistics_dir_y: Fx,
    pub(crate) logistics_severity: Fx,
}

#[derive(Default)]
pub(crate) struct PendingArchCommands {
    pub(crate) reactor_bias: Fx,
    pub(crate) logistics_enabled: bool,
    pub(crate) logistics_preference: ArchLogisticsPreference,
    pub(crate) turret_assist: bool,
    pub(crate) turret_auto_fire: bool,
    pub(crate) turret_fire_hold: bool,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct LumenSnapshot {
    pub(crate) reactor_count: u32,
    pub(crate) turret_count: u32,
    pub(crate) cargo_count: u32,
    pub(crate) processor_count: u32,
    pub(crate) computer_count: u32,
    pub(crate) hot_module_count: u32,
    pub(crate) threat_present: bool,
    pub(crate) low_power: bool,
}

/// Executes ARCH and LUMEN automation programs so ship commands can react to live sensor state.
pub(crate) fn run_arch_automation(
    time: Res<Time>,
    ship_query: Single<
        (
            &Children,
            &ShipPowerState,
            &ShipWeaponState,
            &ShipDamageSensorState,
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
        Option<&ReactorCommandState>,
        Option<&DetectorModule>,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);
    let (
        children,
        ship_power_state,
        ship_weapon_state,
        damage_state,
        mut command_state,
        mut automation_state,
        mut mission_state,
    ) = ship_query.into_inner();

    *command_state = ShipArchCommandState::default();
    automation_state.active = false;
    automation_state.output_scale = Fx::from_num(1);
    automation_state.invalid_executions = 0;
    automation_state.last_primary_program = None;
    automation_state.last_secondary_program = None;
    automation_state.recent_writes.clear();

    let snapshot = build_snapshot(
        children,
        ship_power_state,
        ship_weapon_state,
        damage_state,
        !hostile_query.is_empty(),
        &module_query,
    );
    let lumen_snapshot = build_lumen_snapshot(
        children,
        ship_power_state,
        damage_state,
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
            _reactor,
            _detector,
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

        if arch_runtime.lumen_program.enabled {
            let (lumen_result, lumen_outputs) =
                execute_lumen_program(&arch_runtime.lumen_program, lumen_snapshot);
            automation_state.last_secondary_program = Some(arch_runtime.lumen_program.name.clone());
            aggregate.reactor_bias += lumen_outputs.reactor_bias;
            aggregate.logistics_enabled |= lumen_outputs.logistics_enabled;
            if lumen_outputs.logistics_enabled {
                aggregate.logistics_preference = lumen_outputs.logistics_preference;
            }
            aggregate.turret_assist |= lumen_outputs.turret_assist;
            aggregate.turret_auto_fire |= lumen_outputs.turret_auto_fire;
            aggregate.turret_fire_hold |= lumen_outputs.turret_fire_hold;
            if automation_state.recent_writes.len() < 4 && !lumen_result.recent_effects.is_empty() {
                automation_state
                    .recent_writes
                    .extend(lumen_result.recent_effects.iter().take(2).cloned());
            }
            if lumen_result.halted_reason.is_some() {
                automation_state.invalid_executions += 1;
            }
            arch_runtime.last_lumen_result = lumen_result;
        } else {
            arch_runtime.last_lumen_result = LumenExecutionResult {
                resolved_targets: 0,
                recent_effects: Vec::new(),
                halted_reason: Some("offline".to_string()),
                program_name: arch_runtime.lumen_program.name.clone(),
            };
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
    command_state.turret_fire_hold = aggregate.turret_fire_hold;

    automation_state.active = aggregate.reactor_bias > Fx::from_num(0)
        || aggregate.logistics_enabled
        || aggregate.turret_assist
        || aggregate.turret_auto_fire
        || aggregate.turret_fire_hold;
    if aggregate.reactor_bias > Fx::from_num(0) {
        automation_state.output_scale = Fx::from_num(0.72);
    }
    if automation_state.active {
        automation_state.trigger_count += trigger_delta.max(1);
        mission_state.automation_used = true;
        mission_state.automation_trigger_count = automation_state.trigger_count;
        mission_state.logistics_automation_used |= aggregate.logistics_enabled;
        mission_state.recent_action = Some(format!(
            "Automation active: {}{}",
            automation_state
                .last_primary_program
                .as_deref()
                .unwrap_or("program"),
            automation_state
                .last_secondary_program
                .as_deref()
                .map(|name| format!(" + {name}"))
                .unwrap_or_default()
        ));
        mission_state.recent_action_timer = Fx::from_num(1.4);
    }

    if aggregate.reactor_bias > Fx::from_num(0) {
        for child in children.iter() {
            let Ok((_, runtime_module, mut runtime_state, _, _, _, _, _, _, destroyed)) =
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

pub(crate) fn build_lumen_snapshot(
    children: &Children,
    ship_power_state: &ShipPowerState,
    _damage_state: &ShipDamageSensorState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&ReactorCommandState>,
        Option<&DetectorModule>,
        Option<&DestroyedModule>,
    )>,
) -> LumenSnapshot {
    snapshots::build_lumen_snapshot(
        children,
        ship_power_state,
        _damage_state,
        threat_present,
        module_query,
    )
}

/// Fades recent automation action feedback so command HUD messages feel timely without lingering forever.
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

pub(crate) fn build_snapshot(
    children: &Children,
    ship_power_state: &ShipPowerState,
    ship_weapon_state: &ShipWeaponState,
    damage_state: &ShipDamageSensorState,
    threat_present: bool,
    module_query: &Query<(
        Entity,
        &RuntimeShipModule,
        &mut ModuleRuntimeState,
        Option<&ArchComputerModule>,
        Option<&mut RuntimeArchComputer>,
        Option<&StorageModule>,
        Option<&ProcessorModule>,
        Option<&ReactorCommandState>,
        Option<&DetectorModule>,
        Option<&DestroyedModule>,
    )>,
) -> ArchSnapshot {
    snapshots::build_snapshot(
        children,
        ship_power_state,
        ship_weapon_state,
        damage_state,
        threat_present,
        module_query,
    )
}

pub(crate) fn bool_fx(value: bool) -> Fx {
    Fx::from_num(i32::from(value))
}

pub(crate) fn detector_axis(
    detector: Option<&DetectorModule>,
    axis: impl FnOnce(&DetectorModule) -> Fx,
) -> Fx {
    detector.map(axis).unwrap_or_else(|| Fx::from_num(0))
}

pub(crate) fn detector_metric(
    detector: Option<&DetectorModule>,
    metric: impl FnOnce(&DetectorModule) -> Fx,
) -> Fx {
    detector.map(metric).unwrap_or_else(|| Fx::from_num(0))
}

pub(crate) fn execute_program(
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

pub(crate) fn execute_instruction(
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
        ArchRegister::ReactorReactionRate => snapshot.reactor_reaction_rate,
        ArchRegister::ReactorTurbineLoad => snapshot.reactor_turbine_load,
        ArchRegister::ReactorHeat => snapshot.reactor_heat,
        ArchRegister::ReactorInstability => snapshot.reactor_instability,
        ArchRegister::ReactorPowerOutput => snapshot.reactor_power_output,
        ArchRegister::StorageRawSalvage => snapshot.storage_raw_salvage,
        ArchRegister::StorageRepairCharge => snapshot.storage_repair_charge,
        ArchRegister::ProcessorRawSalvage => snapshot.processor_raw_salvage,
        ArchRegister::ProcessorRepairCharge => snapshot.processor_repair_charge,
        ArchRegister::TurretReady => snapshot.turret_ready,
        ArchRegister::TurretCooldown => snapshot.turret_cooldown,
        ArchRegister::DetectLifeFriendly => snapshot.life_friendly_present,
        ArchRegister::DetectLifeHostile => snapshot.life_hostile_present,
        ArchRegister::DetectLifeDirX => snapshot.life_dir_x,
        ArchRegister::DetectLifeDirY => snapshot.life_dir_y,
        ArchRegister::DetectLifeDistance => snapshot.life_distance,
        ArchRegister::DetectShipNearby => snapshot.ship_nearby_present,
        ArchRegister::DetectShipHostile => snapshot.ship_hostile_present,
        ArchRegister::DetectShipDirX => snapshot.ship_dir_x,
        ArchRegister::DetectShipDirY => snapshot.ship_dir_y,
        ArchRegister::DetectShipDistance => snapshot.ship_distance,
        ArchRegister::DetectDamageIncoming => snapshot.damage_incoming,
        ArchRegister::DetectDamageCritical => snapshot.damage_critical,
        ArchRegister::DetectDamageDirX => snapshot.damage_dir_x,
        ArchRegister::DetectDamageDirY => snapshot.damage_dir_y,
        ArchRegister::DetectDamageIntensity => snapshot.damage_intensity,
        ArchRegister::DetectPowerDeficit => snapshot.power_deficit,
        ArchRegister::DetectPowerLowBattery => snapshot.power_low_battery,
        ArchRegister::DetectHeatAlert => snapshot.heat_alert,
        ArchRegister::DetectHeatDirX => snapshot.heat_dir_x,
        ArchRegister::DetectHeatDirY => snapshot.heat_dir_y,
        ArchRegister::DetectHeatSeverity => snapshot.heat_severity,
        ArchRegister::DetectLogisticsDemand => snapshot.logistics_demand,
        ArchRegister::DetectLogisticsDirX => snapshot.logistics_dir_x,
        ArchRegister::DetectLogisticsDirY => snapshot.logistics_dir_y,
        ArchRegister::DetectLogisticsSeverity => snapshot.logistics_severity,
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

fn execute_lumen_program(
    program: &LumenProgram,
    snapshot: LumenSnapshot,
) -> (LumenExecutionResult, PendingArchCommands) {
    let mut commands = PendingArchCommands::default();
    let mut resolved_targets = 0u32;
    let mut effects = Vec::new();

    for instruction in &program.instructions {
        let target_count = resolve_lumen_target_count(instruction.target, snapshot);
        if target_count == 0 {
            effects.push(format!(
                "{} {} -> no targets",
                instruction.op.as_str(),
                instruction.target.as_str()
            ));
            continue;
        }
        resolved_targets += target_count;
        apply_lumen_instruction(
            instruction,
            snapshot,
            target_count,
            &mut commands,
            &mut effects,
        );
    }

    (
        LumenExecutionResult {
            resolved_targets,
            recent_effects: effects.into_iter().take(4).collect(),
            halted_reason: None,
            program_name: program.name.clone(),
        },
        commands,
    )
}

fn resolve_lumen_target_count(target: LumenTarget, snapshot: LumenSnapshot) -> u32 {
    match target {
        LumenTarget::Reactors => snapshot.reactor_count,
        LumenTarget::Turrets => snapshot.turret_count,
        LumenTarget::Cargo => snapshot.cargo_count,
        LumenTarget::Processors => snapshot.processor_count,
        LumenTarget::Computers => snapshot.computer_count,
        LumenTarget::HotModules => snapshot.hot_module_count,
    }
}

fn apply_lumen_instruction(
    instruction: &LumenInstruction,
    snapshot: LumenSnapshot,
    target_count: u32,
    commands: &mut PendingArchCommands,
    effects: &mut Vec<String>,
) {
    let weight = Fx::from_num(instruction.weight as i32).clamp(Fx::from_num(0), Fx::from_num(3));
    match (instruction.op, instruction.aspect) {
        (LumenOp::Buff, LumenAspect::HeatCooling) | (LumenOp::Nerf, LumenAspect::Instability) => {
            commands.reactor_bias =
                (commands.reactor_bias + weight).clamp(Fx::from_num(0), Fx::from_num(3));
            effects.push(format!(
                "{} {} {} -> cooling +{} ({} targets)",
                instruction.op.as_str(),
                instruction.target.as_str(),
                instruction.aspect.as_str(),
                instruction.weight,
                target_count
            ));
        }
        (LumenOp::Buff, LumenAspect::Throughput) => {
            commands.logistics_enabled = true;
            commands.logistics_preference = ArchLogisticsPreference::FeedProcessor;
            effects.push(format!(
                "BUFF {} throughput -> feed processor ({} targets)",
                instruction.target.as_str(),
                target_count
            ));
        }
        (LumenOp::Nerf, LumenAspect::Throughput) => {
            commands.logistics_enabled = true;
            commands.logistics_preference = ArchLogisticsPreference::StoreCharges;
            effects.push(format!(
                "NERF {} throughput -> store charges ({} targets)",
                instruction.target.as_str(),
                target_count
            ));
        }
        (LumenOp::Buff, LumenAspect::FireControl) => {
            commands.turret_assist = true;
            if snapshot.threat_present || instruction.weight >= 2 {
                commands.turret_auto_fire = true;
            }
            effects.push(format!(
                "BUFF {} fire_control -> assist {}auto ({} targets)",
                instruction.target.as_str(),
                if commands.turret_auto_fire { "+ " } else { "" },
                target_count
            ));
        }
        (LumenOp::Nerf, LumenAspect::FireControl) | (LumenOp::Nerf, LumenAspect::PowerDraw) => {
            commands.turret_fire_hold = true;
            effects.push(format!(
                "{} {} {} -> hold fire ({} targets)",
                instruction.op.as_str(),
                instruction.target.as_str(),
                instruction.aspect.as_str(),
                target_count
            ));
        }
        (LumenOp::Buff, LumenAspect::PowerDraw) => {
            if snapshot.low_power {
                commands.reactor_bias = (commands.reactor_bias + Fx::from_num(1))
                    .clamp(Fx::from_num(0), Fx::from_num(3));
            }
            effects.push(format!(
                "BUFF {} power_draw -> reserve support ({} targets)",
                instruction.target.as_str(),
                target_count
            ));
        }
        _ => {
            effects.push(format!(
                "{} {} {} -> observed ({} targets)",
                instruction.op.as_str(),
                instruction.target.as_str(),
                instruction.aspect.as_str(),
                target_count
            ));
        }
    }
}

fn template_to_mode(template: ArchProgramTemplate) -> ShipAutomationMode {
    match template {
        ArchProgramTemplate::ReactorGuard => ShipAutomationMode::ReactorGuard,
        ArchProgramTemplate::LogisticsFeed => ShipAutomationMode::LogisticsFeed,
        ArchProgramTemplate::TurretAssist => ShipAutomationMode::TurretAssist,
        ArchProgramTemplate::BalancedOps => ShipAutomationMode::BalancedOps,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::*;
    use crate::{
        gameplay::{
            components::{
                DetectorKind,
                HostileShip,
                PlayerMotionState,
                PlayerReferenceFrame,
                SimPosition,
                SimRotation,
            },
            helpers::FixedVec2,
            update_detector_modules,
        },
        ship::ModuleVariant,
    };

    fn runtime_module(kind: ModuleKind, variant: ModuleVariant) -> RuntimeShipModule {
        RuntimeShipModule {
            module_id: 1,
            kind,
            variant,
            channel: 0,
            grid_x: 0,
            grid_y: 0,
            rotation_quadrants: 0,
            local_position: FixedVec2::zero(),
        }
    }

    fn power_state() -> ShipPowerState {
        ShipPowerState {
            stored_energy: Fx::from_num(6),
            generation: Fx::from_num(4),
            draw: Fx::from_num(3),
            surplus: Fx::from_num(1),
            engine_power_ratio: Fx::from_num(1),
            weapons_powered: true,
            engines_powered: true,
        }
    }

    #[test]
    fn life_pulse_reports_presence_without_vector_detail() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, update_detector_modules);

        app.world_mut().spawn((
            PlayerShip,
            ShipRoot,
            SimPosition {
                value: FixedVec2::zero(),
            },
            SimRotation {
                radians: Fx::from_num(0),
            },
            power_state(),
            ShipDamageSensorState::default(),
        ));
        app.world_mut().spawn((
            runtime_module(ModuleKind::Detector, ModuleVariant::LifePulse),
            DetectorModule {
                kind: DetectorKind::LifeSign,
                tier: 1,
                range: Fx::from_num(200),
                detected: false,
                secondary_detected: false,
                direction: FixedVec2::zero(),
                distance: Fx::from_num(0),
                magnitude: Fx::from_num(0),
                critical: false,
            },
        ));
        app.world_mut().spawn(PlayerMotionState {
            frame: PlayerReferenceFrame::World,
            world_position: FixedVec2::from_num(48, 12),
            world_velocity: FixedVec2::zero(),
            local_position: FixedVec2::zero(),
            local_velocity: FixedVec2::zero(),
            facing_radians: Fx::from_num(0),
        });

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<&DetectorModule>();
        let detector = query.single(world).expect("detector present");
        assert!(detector.detected);
        assert!(!detector.secondary_detected);
        assert_eq!(detector.direction.x, Fx::from_num(0));
        assert_eq!(detector.direction.y, Fx::from_num(0));
        assert_eq!(detector.distance, Fx::from_num(0));
    }

    #[test]
    fn ship_survey_reports_hostile_direction_and_distance() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, update_detector_modules);

        app.world_mut().spawn((
            PlayerShip,
            ShipRoot,
            SimPosition {
                value: FixedVec2::zero(),
            },
            SimRotation {
                radians: Fx::from_num(0),
            },
            power_state(),
            ShipDamageSensorState::default(),
        ));
        app.world_mut().spawn((
            HostileShip,
            ShipRoot,
            SimPosition {
                value: FixedVec2::from_num(120, -40),
            },
        ));
        app.world_mut().spawn((
            runtime_module(ModuleKind::Detector, ModuleVariant::ShipSurvey),
            DetectorModule {
                kind: DetectorKind::Ship,
                tier: 3,
                range: Fx::from_num(500),
                detected: false,
                secondary_detected: false,
                direction: FixedVec2::zero(),
                distance: Fx::from_num(0),
                magnitude: Fx::from_num(0),
                critical: false,
            },
        ));

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<&DetectorModule>();
        let detector = query.single(world).expect("detector present");
        assert!(detector.detected);
        assert!(detector.secondary_detected);
        assert!(detector.distance > Fx::from_num(0));
        assert!(detector.direction.x > Fx::from_num(0));
        assert!(detector.direction.y < Fx::from_num(0));
    }
}

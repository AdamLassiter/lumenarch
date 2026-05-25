use super::*;

pub(super) fn apply_blocker_arch_commands(
    junction_open: Option<bool>,
    valve_open: Option<bool>,
    blocker_query: &mut Query<(
        &RuntimeShipModule,
        Option<&mut JunctionCommandState>,
        Option<&mut ValveCommandState>,
        Option<&DestroyedModule>,
    )>,
) {
    if junction_open.is_none() && valve_open.is_none() {
        return;
    }
    for (runtime_module, junction, valve, destroyed) in blocker_query.iter_mut() {
        if destroyed.is_some() {
            continue;
        }
        if matches!(runtime_module.kind, ModuleKind::JunctionBox)
            && let (Some(open), Some(mut junction)) = (junction_open, junction)
        {
            junction.open = open;
        }
        if matches!(runtime_module.kind, ModuleKind::Valve)
            && let (Some(open), Some(mut valve)) = (valve_open, valve)
        {
            valve.open = open;
        }
    }
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

pub(super) fn is_truthy(value: Fx) -> bool {
    value != Fx::from_num(0)
}

pub(super) fn jump_target(target: usize, pc: usize) -> Result<usize, String> {
    if target <= pc {
        return Err("backward jump rejected".to_string());
    }
    Ok(target)
}

pub(super) fn jump_if(condition: bool, target: usize, pc: usize) -> Result<usize, String> {
    if condition {
        jump_target(target, pc)
    } else {
        Ok(pc + 1)
    }
}

pub(super) fn pow_fx(base: Fx, exponent: Fx) -> Result<Fx, String> {
    let base_f64 = base.to_num::<f64>();
    let exponent_f64 = exponent.to_num::<f64>();
    let value = base_f64.powf(exponent_f64);
    if !value.is_finite() {
        return Err("pow produced non-finite value".to_string());
    }
    Ok(Fx::from_num(value))
}

pub(super) fn log_fx(value: Fx) -> Result<Fx, String> {
    if value <= Fx::from_num(0) {
        return Err("log of non-positive value".to_string());
    }
    let result = value.to_num::<f64>().ln();
    if !result.is_finite() {
        return Err("log produced non-finite value".to_string());
    }
    Ok(Fx::from_num(result))
}

pub(super) fn resolve_value(
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

pub(super) fn read_register(
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
        ArchRegister::TurretDesiredAngle => snapshot.turret_desired_angle,
        ArchRegister::TurretActualAngle => snapshot.turret_actual_angle,
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
        ArchRegister::JunctionOpen => snapshot.junction_open,
        ArchRegister::JunctionPowered => snapshot.junction_powered,
        ArchRegister::JunctionSupply => snapshot.junction_supply,
        ArchRegister::JunctionDemand => snapshot.junction_demand,
        ArchRegister::ValveOpen => snapshot.valve_open,
        ArchRegister::ValvePowered => snapshot.valve_powered,
        ArchRegister::ValveSupply => snapshot.valve_supply,
        ArchRegister::ValveDemand => snapshot.valve_demand,
        ArchRegister::CmdReactorBias => commands.reactor_bias,
        ArchRegister::CmdLogisticsEnable => Fx::from_num(i32::from(commands.logistics_enabled)),
        ArchRegister::CmdLogisticsPreference => Fx::from_num(match commands.logistics_preference {
            ArchLogisticsPreference::FeedProcessor => 0,
            ArchLogisticsPreference::StoreCharges => 1,
        }),
        ArchRegister::CmdTurretAssist => Fx::from_num(i32::from(commands.turret_assist)),
        ArchRegister::CmdTurretAutoFire => Fx::from_num(i32::from(commands.turret_auto_fire)),
        ArchRegister::CmdJunctionOpen => {
            Fx::from_num(i32::from(commands.junction_open.unwrap_or(false)))
        }
        ArchRegister::CmdValveOpen => Fx::from_num(i32::from(commands.valve_open.unwrap_or(false))),
    }
}

pub(super) fn write_register(
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
        ArchRegister::CmdJunctionOpen => {
            commands.junction_open = Some(value > Fx::from_num(0));
            writes.push(format!(
                "{} <- {}",
                register.as_str(),
                if commands.junction_open == Some(true) {
                    "open"
                } else {
                    "closed"
                }
            ));
        }
        ArchRegister::CmdValveOpen => {
            commands.valve_open = Some(value > Fx::from_num(0));
            writes.push(format!(
                "{} <- {}",
                register.as_str(),
                if commands.valve_open == Some(true) {
                    "open"
                } else {
                    "closed"
                }
            ));
        }
        _ => {}
    }

    Ok(())
}

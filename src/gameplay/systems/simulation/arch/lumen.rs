use super::*;

pub(super) fn execute_lumen_program(
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

pub(super) fn resolve_lumen_target_count(target: LumenTarget, snapshot: LumenSnapshot) -> u32 {
    match target {
        LumenTarget::Reactors => snapshot.reactor_count,
        LumenTarget::Turrets => snapshot.turret_count,
        LumenTarget::Cargo => snapshot.cargo_count,
        LumenTarget::Processors => snapshot.processor_count,
        LumenTarget::Computers => snapshot.computer_count,
        LumenTarget::HotModules => snapshot.hot_module_count,
    }
}

pub(super) fn apply_lumen_instruction(
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

pub(super) fn template_to_mode(template: ArchProgramTemplate) -> ShipAutomationMode {
    match template {
        ArchProgramTemplate::ReactorGuard => ShipAutomationMode::ReactorGuard,
        ArchProgramTemplate::LogisticsFeed => ShipAutomationMode::LogisticsFeed,
        ArchProgramTemplate::TurretAssist => ShipAutomationMode::TurretAssist,
        ArchProgramTemplate::BalancedOps => ShipAutomationMode::BalancedOps,
    }
}

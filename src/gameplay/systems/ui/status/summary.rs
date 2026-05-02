use super::*;

pub(super) struct ArchSummary {
    pub(super) program_name: String,
    pub(super) exec_summary: String,
    pub(super) invalid_count: u32,
    pub(super) recent_writes: String,
}

#[derive(Clone, Copy, Default)]
pub(super) struct StationFlags {
    pub(super) storage: bool,
    pub(super) manipulator: bool,
    pub(super) processor: bool,
    pub(super) airlock: bool,
    pub(super) computer: bool,
    pub(super) reactor: bool,
    pub(super) turret: bool,
}

pub(super) fn build_top_banner(
    mission_state: &MissionState,
    weapon_state: &ShipWeaponState,
    hostile_count: usize,
    projectile_count: usize,
) -> String {
    let outcome = match mission_return_line(mission_state) {
        Some(return_line) => format!("{}  |  {}", mission_status_line(mission_state), return_line),
        None => mission_status_line(mission_state).to_string(),
    };
    format!(
        "{outcome}\nNode: {} [{}]  |  Threat x{}  |  Projectiles {}  |  Turret Cooldown {}",
        mission_state.node_name,
        mission_state.node_kind_name,
        hostile_count,
        projectile_count,
        format_fx2(weapon_state.cooldown_remaining.max(Fx::from_num(0)))
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_compact_status(
    player_motion: &PlayerMotionState,
    ship_position: &SimPosition,
    linear_velocity: &LinearVelocity,
    angular_velocity: &AngularVelocity,
    control_mode: &ShipboardControlState,
    current_station: &CurrentStation,
    frame_label: &str,
    focused_station_context: &str,
    current_integrity: i32,
    max_integrity: i32,
    active_modules: usize,
    degraded_modules: usize,
    disabled_modules: usize,
    atmosphere_state: &ShipAtmosphereState,
    carried_resource: &CarriedResource,
    mission_state: &MissionState,
    scrap_total: u32,
    arch_summary: &ArchSummary,
    rollback_frame: i32,
    checksum: u128,
) -> String {
    format!(
        "Mode: {}  |  Focus: {}\nFrame: {}\nContext: {}\nStation: {}\nPlayer: {}, {} @ {}\nShip: {}, {} @ {}\nTurn: {}\nModules: {} active  |  {} degraded  |  {} disabled\nIntegrity: {} / {}\nAtmosphere: {} avg / {} min  |  venting {}\nCargo: {}\nMission Ops: repairs {}  stabs {}  transfers {}  cycles {}\nARCH: {}  [{}]  writes {}  invalid {}\nRollback: frame {}  checksum {:016x}\nScrap: {}",
        control_mode.mode.as_str(),
        control_mode
            .focused_family
            .map(|family| family.as_str())
            .unwrap_or("None"),
        frame_label,
        focused_station_context,
        module_display_name(current_station.kind),
        format_fx0(player_motion.world_position.x),
        format_fx0(player_motion.world_position.y),
        format_fx1(player_motion.world_velocity.length()),
        format_fx0(ship_position.value.x),
        format_fx0(ship_position.value.y),
        format_fx1(linear_velocity.value.length()),
        format_fx2(angular_velocity.radians_per_second),
        active_modules,
        degraded_modules,
        disabled_modules,
        current_integrity,
        max_integrity,
        format_fx1(atmosphere_state.average_oxygen),
        format_fx1(atmosphere_state.minimum_oxygen),
        atmosphere_state.venting_tiles,
        carried_resource
            .kind
            .map(|kind| format!("{} {}", carried_resource.amount, resource_kind_label(kind)))
            .unwrap_or_else(|| "none".to_string()),
        mission_state.repairs_performed,
        mission_state.stabilizations_performed,
        mission_state.transfer_count,
        mission_state.processor_cycles,
        arch_summary.program_name,
        arch_summary.exec_summary,
        arch_summary.recent_writes,
        arch_summary.invalid_count,
        rollback_frame,
        checksum,
        scrap_total,
    )
}

pub(super) fn summarize_modules(
    children: &Children,
    module_query: &Query<
        (
            Entity,
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&StorageCommandState>,
            Option<&ManipulatorModule>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorModule>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
    balance: &BalanceConfig,
) -> (i32, i32, usize, usize, usize) {
    let mut current_integrity = 0;
    let mut max_integrity = 0;
    let mut active_modules = 0;
    let mut degraded = 0;
    let mut disabled = 0;

    for child in children.iter() {
        let Ok((_, _, integrity, runtime_state, _, _, _, _, _, _, _, _, _, _, destroyed)) =
            module_query.get(child)
        else {
            continue;
        };
        max_integrity += integrity.max;
        let condition = module_condition(integrity, runtime_state, destroyed.is_some(), balance);
        if condition != ModuleCondition::Destroyed {
            current_integrity += integrity.current;
            active_modules += 1;
        }
        match condition {
            ModuleCondition::Degraded => degraded += 1,
            ModuleCondition::Disabled => disabled += 1,
            _ => {}
        }
    }

    (
        current_integrity,
        max_integrity,
        active_modules,
        degraded,
        disabled,
    )
}

pub(super) fn summarize_arch(
    children: &Children,
    module_query: &Query<
        (
            Entity,
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&RuntimeArchComputer>,
            Option<&StorageModule>,
            Option<&StorageCommandState>,
            Option<&ManipulatorModule>,
            Option<&ManipulatorCommandState>,
            Option<&ProcessorModule>,
            Option<&ProcessorCommandState>,
            Option<&ReactorCommandState>,
            Option<&TurretCommandState>,
            Option<&AirlockCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> ArchSummary {
    for child in children.iter() {
        let Ok((_, _, _, _, computer, _, _, _, _, _, _, _, _, _, destroyed)) =
            module_query.get(child)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        if let Some(computer) = computer {
            return ArchSummary {
                program_name: if computer.last_result.program_name.is_empty() {
                    computer.program.name.clone()
                } else {
                    computer.last_result.program_name.clone()
                },
                exec_summary: format!(
                    "{}/{}",
                    computer.last_result.executed, computer.last_result.budget
                ),
                invalid_count: u32::from(computer.last_result.halted_reason.is_some()),
                recent_writes: if computer.last_result.recent_writes.is_empty() {
                    "none".to_string()
                } else {
                    computer.last_result.recent_writes.join(", ")
                },
            };
        }
    }

    ArchSummary {
        program_name: "none".to_string(),
        exec_summary: "0/0".to_string(),
        invalid_count: 0,
        recent_writes: "none".to_string(),
    }
}

pub(super) fn controls_help_text(mode: ShipControlMode) -> String {
    match mode {
        ShipControlMode::Interior => {
            "Walk / EVA\nWASD move or thrust\nE enter station\nF pick up/deposit cargo\nG drop cargo\nQ or Esc leave station\nF3 diagnostics  |  Tab station hub".to_string()
        }
        ShipControlMode::Cockpit => {
            "Helm\nW/S throttle  |  A/D steer\nMouse can also drive helm\nUse on-panel controls for coarse trim\nQ or Esc leave cockpit".to_string()
        }
        ShipControlMode::Turret => {
            "Turret\nMouse or A/D aim\nSpace / left mouse fire\nPanel buttons trim aim or hold fire\nQ or Esc leave turret".to_string()
        }
        ShipControlMode::Reactor => {
            "Reactor\nW/S reaction rate  |  A/D turbine load\nPanel controls support coarse adjustment\nQ or Esc leave reactor".to_string()
        }
        ShipControlMode::Logistics => {
            "Logistics\nSpace, M, R, [ and ] still work\nPanel buttons mirror key actions\nF/G handle carried cargo\nQ or Esc leave panel".to_string()
        }
        ShipControlMode::Computer => {
            "Computer\nSpace toggles online  |  T cycles starter template\nUse the refit ARCH panel for line editing\nQ or Esc leave console".to_string()
        }
    }
}

pub(super) fn reference_frame_label(
    player_motion: &PlayerMotionState,
    ship_identity_query: &Query<
        (Entity, Option<&PlayerShip>, Option<&HostileShip>),
        With<ShipRoot>,
    >,
) -> String {
    match player_motion.frame {
        PlayerReferenceFrame::World => "EVA / World".to_string(),
        PlayerReferenceFrame::Ship(ship_entity) => {
            format!(
                "Ship Local ({})",
                ship_affiliation_label(ship_entity, ship_identity_query)
            )
        }
    }
}

pub(super) fn ship_affiliation_label(
    ship_entity: Entity,
    ship_identity_query: &Query<
        (Entity, Option<&PlayerShip>, Option<&HostileShip>),
        With<ShipRoot>,
    >,
) -> &'static str {
    ship_identity_query
        .get(ship_entity)
        .map(|(_, player_ship, hostile_ship)| {
            if player_ship.is_some() {
                "Player Ship"
            } else if hostile_ship.is_some() {
                "Hostile Ship"
            } else {
                "Unmarked Ship"
            }
        })
        .unwrap_or("Unknown Ship")
}

pub(super) fn nearby_logistics_target_ids(
    focused_module_id: u64,
    candidate_query: &Query<&RuntimeShipModule>,
) -> Vec<u64> {
    candidate_query
        .iter()
        .filter(|runtime_module| {
            matches!(
                runtime_module.kind,
                crate::ship::ModuleKind::Cargo
                    | crate::ship::ModuleKind::Processor
                    | crate::ship::ModuleKind::Airlock
            ) && runtime_module.module_id != focused_module_id
        })
        .map(|runtime_module| runtime_module.module_id)
        .collect()
}

pub(super) fn percent(value: f32, max: f32) -> f32 {
    if max <= 0.0 {
        0.0
    } else {
        ((value / max) * 100.0).clamp(0.0, 100.0)
    }
}

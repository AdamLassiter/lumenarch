use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    NORMAL_BUTTON,
    SELECTED_BUTTON,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    UI_TITLE_FONT_SIZE,
    docked::DockedActionVisibility,
    state::{
        DockedState,
        DockedSurface,
        EditorShip,
        LastMissionReport,
        LocalPlayerProfile,
        Progression,
        SectorState,
    },
    stations,
};

pub(crate) fn spawn_action_button<T: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    color: Color,
    marker: T,
    font: &Handle<Font>,
    visibility: DockedActionVisibility,
    selected_surface: DockedSurface,
) {
    parent
        .spawn((
            Button,
            Node {
                display: if visibility.visible_on(selected_surface) {
                    Display::Flex
                } else {
                    Display::None
                },
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(color),
            visibility,
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: UI_TITLE_FONT_SIZE - 2.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(crate) fn spawn_dual_action_row<L: Bundle + 'static, R: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    left: (&str, L),
    right: (&str, R),
    font: &Handle<Font>,
    visibility: DockedActionVisibility,
    selected_surface: DockedSurface,
) {
    parent
        .spawn((
            Node {
                display: if visibility.visible_on(selected_surface) {
                    Display::Flex
                } else {
                    Display::None
                },
                width: Val::Percent(100.0),
                column_gap: Val::Px(10.0),
                ..default()
            },
            visibility,
        ))
        .with_children(|row| {
            spawn_half_width_action_button(row, left.0, left.1, font);
            spawn_half_width_action_button(row, right.0, right.1, font);
        });
}

fn spawn_half_width_action_button<T: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: T,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(50.0),
                height: Val::Px(42.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.22, 0.30, 0.44)),
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: UI_BODY_FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(crate) fn docked_status_text(
    docked_state: &DockedState,
    progression: &Progression,
    editor_ship: &EditorShip,
    last_mission_report: &LastMissionReport,
    local_profile: &LocalPlayerProfile,
) -> String {
    let mission = match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!("{headline}\n{detail}"),
        (Some(headline), None) => headline.clone(),
        _ => "No completed sorties yet.".to_string(),
    };

    let damaged_components = progression
        .damaged_components()
        .map(|entry| format!("{} x{}", entry.label(), entry.damaged))
        .collect::<Vec<_>>();

    format!(
        "Hub: {}\nCrew: {} ({})\nDefault Suit: {}\nScrap: {}\nDamaged Parts: {}\nHull Wear: {}\nJumps: {}\nShip: {}\nModules: {}\n\nLast Result\n{}",
        docked_state.station_title,
        local_profile.name,
        local_profile.role.as_str(),
        local_profile.starting_suit().as_str(),
        progression.scrap,
        if damaged_components.is_empty() {
            "none".to_string()
        } else {
            damaged_components.join(", ")
        },
        progression.hull_wear,
        progression.jump_count,
        editor_ship.ship.name,
        editor_ship.ship.modules.len(),
        mission
    )
}

pub(crate) fn docked_surface_color(selected: DockedSurface, surface: DockedSurface) -> Color {
    if selected == surface {
        SELECTED_BUTTON
    } else {
        Color::srgb(0.18, 0.24, 0.34)
    }
}

pub(crate) fn button_default_color(
    is_repair: bool,
    is_cycle: bool,
    is_accept: bool,
    is_launch: bool,
) -> Color {
    if is_repair {
        Color::srgb(0.45, 0.34, 0.16)
    } else if is_cycle {
        Color::srgb(0.22, 0.30, 0.44)
    } else if is_accept {
        Color::srgb(0.28, 0.46, 0.74)
    } else if is_launch {
        Color::srgb(0.18, 0.50, 0.30)
    } else {
        NORMAL_BUTTON
    }
}

pub(crate) fn docked_help_text(surface: DockedSurface) -> String {
    match surface {
        DockedSurface::Shipyard => "Dock Controls\nClick a hub surface to move around the station UI\nRefit: open shipyard refit\nRepair: spend scrap to clear hull wear\nSector Map: inspect the local route graph".to_string(),
        DockedSurface::Quartermaster => "Quartermaster Controls\nReview scrap, damaged components, and service availability\nRepair Ship spends scrap immediately when available".to_string(),
        DockedSurface::Contracts => "Contract Board Controls\nPrevious/Next: browse offers\nAccept: make the selected contract active\nLaunch: depart on the active contract".to_string(),
        DockedSurface::Archives => "Archives Controls\nBrowse station contacts and recovered lore entries\nViewing lore is local; progression unlocks are shared when earned".to_string(),
    }
}

pub(crate) fn docked_content_text(
    station: Option<&stations::StationDefinition>,
    docked_state: &DockedState,
    progression: &Progression,
    last_mission_report: &LastMissionReport,
    sector_state: &SectorState,
) -> String {
    let Some(station) = station else {
        return "No station record available for this dock.".to_string();
    };

    match docked_state.selected_surface {
        DockedSurface::Shipyard => {
            let repair_cost = progression.hull_wear.saturating_mul(2);
            format!(
                "{}\nFaction: {}\n\n{}\n\nServices: {}\n\nHull wear repair cost: {} scrap\nCurrent route node: {}\nLast report: {}",
                station.name,
                station.faction.as_str(),
                station.flavor,
                station
                    .services
                    .iter()
                    .map(|service| service.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                repair_cost,
                sector_state
                    .current_node()
                    .map(|node| node.label.as_str())
                    .unwrap_or("unknown"),
                last_mission_report
                    .headline
                    .clone()
                    .unwrap_or_else(|| "No mission report yet".to_string()),
            )
        }
        DockedSurface::Quartermaster => {
            let damaged = progression
                .damaged_components()
                .map(|entry| {
                    format!(
                        "{} x{} (repair {} scrap)",
                        entry.label(),
                        entry.damaged,
                        entry.repair_cost()
                    )
                })
                .collect::<Vec<_>>();
            format!(
                "Quartermaster Ledger\n\nScrap on hand: {}\nStored component stacks: {}\nDamaged inventory:\n{}\n\nQuartermaster note\n'Scrap is no good until it becomes pressure, wiring, or time.'",
                progression.scrap,
                progression.stored_components.len(),
                if damaged.is_empty() {
                    "  none".to_string()
                } else {
                    damaged
                        .into_iter()
                        .map(|line| format!("  {line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
            )
        }
        DockedSurface::Contracts => {
            let contract = station
                .contracts
                .get(docked_state.selected_contract_index % station.contracts.len().max(1));
            let active = progression
                .active_contract_id
                .as_ref()
                .map_or("none".to_string(), |id| id.clone());
            if let Some(contract) = contract {
                let contact_name = station
                    .contacts
                    .iter()
                    .find(|contact| contact.id == contract.contact_id)
                    .map(|contact| contact.name.as_str())
                    .unwrap_or("Unknown Contact");
                let status =
                    if progression.active_contract_id.as_deref() == Some(contract.id.as_str()) {
                        "ACTIVE"
                    } else if progression.contract_completed(&contract.id) {
                        "COMPLETED"
                    } else {
                        "AVAILABLE"
                    };
                format!(
                    "Contract Board\n\nOffer: {} [{}]\nContact: {}\nTarget Node: {}\nType: {}\nReward Bonus: {} scrap\nStatus: {}\n\nBriefing\n{}\n\nLaunch Note\n{}\n\nCurrent active contract: {}",
                    contract.title,
                    contract.id,
                    contact_name,
                    contract.target_node_id,
                    contract.kind.as_str(),
                    contract.reward_bonus_scrap,
                    status,
                    contract.briefing,
                    contract.launch_blurb,
                    active,
                )
            } else {
                "No contracts are currently posted.".to_string()
            }
        }
        DockedSurface::Archives => {
            let contact = station
                .contacts
                .get(docked_state.selected_contact_index % station.contacts.len().max(1));
            let lore = station
                .lore_entries
                .get(docked_state.selected_lore_index % station.lore_entries.len().max(1));
            let contact_block = contact.map_or_else(
                || "No contact selected.".to_string(),
                |contact| {
                    format!(
                        "Contact\n{} // {}\n{}\n\n'{}'",
                        contact.name, contact.role, contact.bio, contact.brief
                    )
                },
            );
            let lore_block = lore.map_or_else(
                || "No lore entry selected.".to_string(),
                |entry| {
                    if progression.lore_unlocked(&entry.id) {
                        format!("Lore\n{}\n{}", entry.title, entry.body)
                    } else {
                        format!(
                            "Lore\n{}\nEntry locked. Recover more field intel or complete station work to unlock it.",
                            entry.title
                        )
                    }
                },
            );
            format!("{contact_block}\n\n{lore_block}")
        }
    }
}

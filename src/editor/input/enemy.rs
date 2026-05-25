use bevy::{log, prelude::*};

use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    ship::enemy::{
        EnemyShipEntryValidationStatus,
        save_default_enemy_library,
        validate_enemy_ship_definition,
    },
    state::{
        EditorMode,
        EditorSessionState,
        EditorShip,
        EnemyEditorState,
        EnemyNextButton,
        EnemyPrevButton,
        EnemyShipLibraryState,
        FocusedTextBox,
        SectorState,
    },
};

pub(super) fn sync_selected_enemy_entry(
    editor_ship: &EditorShip,
    enemy_library_state: &mut EnemyShipLibraryState,
) -> bool {
    enemy_library_state.library.ensure_seeded();
    let selected_index = enemy_library_state.selected_index;
    let Some(entry) = enemy_library_state
        .library
        .selected_or_first_mut(selected_index)
    else {
        return true;
    };
    entry.ship = editor_ship.ship.clone();
    entry.display_name = editor_ship.ship.name.clone();
    let entry_id = entry.id.clone();
    let status = match validate_enemy_ship_definition(&editor_ship.ship) {
        Ok(()) => EnemyShipEntryValidationStatus::Valid,
        Err(error) => {
            log::warn!(
                "Current enemy ship entry '{}' is invalid and will not be saved yet: {}",
                entry_id,
                error
            );
            EnemyShipEntryValidationStatus::Invalid
        }
    };
    enemy_library_state.entry_statuses.insert(entry_id, status);
    status == EnemyShipEntryValidationStatus::Valid
}

pub(super) fn save_enemy_library_if_valid(enemy_library_state: &EnemyShipLibraryState) -> bool {
    if let Some((entry_id, error)) = enemy_library_state
        .library
        .entries
        .iter()
        .find_map(|entry| {
            validate_enemy_ship_definition(&entry.ship)
                .err()
                .map(|error| (entry.id.clone(), error))
        })
    {
        log::warn!(
            "Skipped enemy library save because entry '{}' is invalid: {}",
            entry_id,
            error
        );
        return false;
    }

    if let Err(error) = save_default_enemy_library(&enemy_library_state.library) {
        eprintln!("editor: failed to save enemy ship library: {error}");
        return false;
    }

    true
}

pub(super) fn referenced_enemy_ids(sector_state: &SectorState) -> Vec<String> {
    let mut ids = Vec::new();
    for node in &sector_state.nodes {
        for enemy_id in &node.encounter.enemy_ship_ids {
            if !ids.iter().any(|existing| existing == enemy_id) {
                ids.push(enemy_id.clone());
            }
        }
    }
    ids
}

pub(crate) fn ensure_selected_enemy_reference(
    sector_state: &SectorState,
    enemy_library_state: &mut EnemyShipLibraryState,
) {
    let referenced = referenced_enemy_ids(sector_state);
    if referenced.is_empty() {
        enemy_library_state.library.ensure_seeded();
        enemy_library_state.selected_index = enemy_library_state
            .selected_index
            .min(enemy_library_state.library.entries.len().saturating_sub(1));
        return;
    }
    let current_id = enemy_library_state
        .library
        .selected_or_first(enemy_library_state.selected_index)
        .map(|entry| entry.id.as_str());
    if current_id.is_some_and(|id| referenced.iter().any(|reference| reference == id)) {
        return;
    }
    enemy_library_state.selected_index = enemy_library_state
        .library
        .ensure_entry_for_id(referenced.first().expect("referenced enemy id"));
}

fn cycle_referenced_enemy(
    sector_state: &SectorState,
    enemy_library_state: &mut EnemyShipLibraryState,
    direction: i32,
) {
    let referenced = referenced_enemy_ids(sector_state);
    if referenced.is_empty() {
        enemy_library_state.library.ensure_seeded();
        let len = enemy_library_state.library.entries.len();
        if len > 0 {
            enemy_library_state.selected_index = (enemy_library_state.selected_index as i32
                + direction)
                .rem_euclid(len as i32) as usize;
        }
        return;
    }
    let current_id = enemy_library_state
        .library
        .selected_or_first(enemy_library_state.selected_index)
        .map(|entry| entry.id.as_str());
    let current_reference_index = current_id
        .and_then(|id| referenced.iter().position(|reference| reference == id))
        .unwrap_or(0);
    let next_reference_index =
        (current_reference_index as i32 + direction).rem_euclid(referenced.len() as i32) as usize;
    enemy_library_state.selected_index = enemy_library_state
        .library
        .ensure_entry_for_id(&referenced[next_reference_index]);
}

/// Handles enemy-library browser buttons so debug authors can step between referenced hostile ship entries.
pub(crate) fn enemy_library_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&EnemyPrevButton>,
            Option<&EnemyNextButton>,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<EnemyPrevButton>, With<EnemyNextButton>)>,
        ),
    >,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    sector_state: Res<SectorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    for (interaction, mut background, prev, next) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                let current_entry_saved =
                    sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
                        && save_enemy_library_if_valid(&enemy_library_state);
                if prev.is_some() {
                    cycle_referenced_enemy(&sector_state, &mut enemy_library_state, -1);
                } else if next.is_some() {
                    cycle_referenced_enemy(&sector_state, &mut enemy_library_state, 1);
                }
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                    enemy_editor_state.dirty = !current_entry_saved;
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.24, 0.32, 0.48));
            }
        }
    }
}

/// Mirrors enemy-library navigation onto the keyboard so hostile ship iteration stays quick while editing.
pub(crate) fn enemy_library_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    sector_state: Res<SectorState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if !netcode::is_host_authority(&status) {
        return;
    }
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    let mut changed = false;
    let mut current_entry_saved = true;
    if keys.just_pressed(KeyCode::BracketLeft) {
        current_entry_saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
            && save_enemy_library_if_valid(&enemy_library_state);
        cycle_referenced_enemy(&sector_state, &mut enemy_library_state, -1);
        changed = true;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        current_entry_saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
            && save_enemy_library_if_valid(&enemy_library_state);
        cycle_referenced_enemy(&sector_state, &mut enemy_library_state, 1);
        changed = true;
    }

    if changed
        && let Some(entry) = enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
    {
        editor_ship.ship = entry.ship.clone();
        enemy_editor_state.dirty = !current_entry_saved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ship::{ShipDefinition, enemy::EnemyShipLibrary},
        state::{EncounterBackdrop, EncounterSpec, SectorNode, SectorNodeKind, SectorNodeStatus},
        stations::FactionId,
    };

    fn sector_with_enemy_refs(enemy_refs: &[&[&str]]) -> SectorState {
        SectorState {
            seed: 1,
            current_node_id: 0,
            selected_node_id: None,
            active_encounter_node_id: None,
            nodes: enemy_refs
                .iter()
                .enumerate()
                .map(|(index, refs)| SectorNode {
                    id: index as u32,
                    label: format!("Node {index}"),
                    kind: SectorNodeKind::HostileHold,
                    station_id: None,
                    risk_tier: 1,
                    reward_hint: String::new(),
                    neighbors: Vec::new(),
                    status: SectorNodeStatus::Fresh,
                    position: [0.0, 0.0],
                    encounter: EncounterSpec {
                        enemy_ship_ids: refs.iter().map(|id| id.to_string()).collect(),
                        hostile_count: refs.len() as u32,
                        salvage_value: 0,
                        ambient_heat_pressure: 0,
                        ambient_electrical_pressure: 0,
                        reward_multiplier: 1,
                        arena_variant: "test".to_string(),
                        backdrop: EncounterBackdrop::default(),
                    },
                })
                .collect(),
        }
    }

    fn enemy_entry(id: &str) -> crate::ship::enemy::EnemyShipEntry {
        crate::ship::enemy::EnemyShipEntry {
            id: id.to_string(),
            display_name: format!("{id} display"),
            threat_tier: 1,
            behavior_tag: "test".to_string(),
            faction_id: FactionId::RogueContinuants,
            ship_name: Some(id.to_string()),
            captain_name: None,
            comms_intro: None,
            comms_outro: None,
            is_crewed: true,
            ship: ShipDefinition::core_only(id),
        }
    }

    #[test]
    fn enemy_references_define_navigation_order_without_duplicates() {
        let sector = sector_with_enemy_refs(&[&["alpha", "beta"], &["alpha", "gamma"]]);

        assert_eq!(
            referenced_enemy_ids(&sector),
            vec!["alpha".to_string(), "beta".to_string(), "gamma".to_string()]
        );
    }

    #[test]
    fn missing_referenced_enemy_creates_fixed_id_entry() {
        let sector = sector_with_enemy_refs(&[&["missing_enemy"]]);
        let mut state = EnemyShipLibraryState {
            library: EnemyShipLibrary {
                entries: vec![enemy_entry("unreferenced")],
            },
            selected_index: 0,
            entry_statuses: Default::default(),
        };

        ensure_selected_enemy_reference(&sector, &mut state);

        let selected = state
            .library
            .selected_or_first(state.selected_index)
            .unwrap();
        assert_eq!(selected.id, "missing_enemy");
        assert_eq!(selected.display_name, "missing_enemy");
    }

    #[test]
    fn unreferenced_enemy_entries_are_not_navigable() {
        let sector = sector_with_enemy_refs(&[&["alpha", "beta"]]);
        let mut state = EnemyShipLibraryState {
            library: EnemyShipLibrary {
                entries: vec![
                    enemy_entry("unreferenced"),
                    enemy_entry("alpha"),
                    enemy_entry("beta"),
                ],
            },
            selected_index: 0,
            entry_statuses: Default::default(),
        };

        ensure_selected_enemy_reference(&sector, &mut state);
        assert_eq!(state.library.entries[state.selected_index].id, "alpha");

        cycle_referenced_enemy(&sector, &mut state, 1);
        assert_eq!(state.library.entries[state.selected_index].id, "beta");

        cycle_referenced_enemy(&sector, &mut state, 1);
        assert_eq!(state.library.entries[state.selected_index].id, "alpha");
    }
}

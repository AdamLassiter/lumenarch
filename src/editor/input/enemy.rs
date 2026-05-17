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
        EnemyNewButton,
        EnemyNextButton,
        EnemyPrevButton,
        EnemyShipLibraryState,
        FocusedTextBox,
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

/// Handles enemy-library browser buttons so debug authors can step between, create, and load hostile ship entries.
pub(crate) fn enemy_library_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&EnemyPrevButton>,
            Option<&EnemyNextButton>,
            Option<&EnemyNewButton>,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(
                With<EnemyPrevButton>,
                With<EnemyNextButton>,
                With<EnemyNewButton>,
            )>,
        ),
    >,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    for (interaction, mut background, prev, next, new_entry) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                let current_entry_saved =
                    sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
                        && save_enemy_library_if_valid(&enemy_library_state);
                enemy_library_state.library.ensure_seeded();
                if prev.is_some() && !enemy_library_state.library.entries.is_empty() {
                    let len = enemy_library_state.library.entries.len();
                    enemy_library_state.selected_index =
                        (enemy_library_state.selected_index + len - 1) % len;
                } else if next.is_some() && !enemy_library_state.library.entries.is_empty() {
                    enemy_library_state.selected_index = (enemy_library_state.selected_index + 1)
                        % enemy_library_state.library.entries.len();
                } else if new_entry.is_some() {
                    enemy_library_state.selected_index =
                        enemy_library_state.library.add_blank_entry();
                    let selected_entry_id = enemy_library_state
                        .library
                        .selected_or_first(enemy_library_state.selected_index)
                        .map(|entry| entry.id.clone());
                    if let Some(entry_id) = selected_entry_id {
                        enemy_library_state
                            .entry_statuses
                            .insert(entry_id, EnemyShipEntryValidationStatus::Valid);
                    }
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
    enemy_library_state.library.ensure_seeded();
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketLeft) {
        current_entry_saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
            && save_enemy_library_if_valid(&enemy_library_state);
        let len = enemy_library_state.library.entries.len();
        enemy_library_state.selected_index = (enemy_library_state.selected_index + len - 1) % len;
        changed = true;
    }
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketRight) {
        current_entry_saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
            && save_enemy_library_if_valid(&enemy_library_state);
        enemy_library_state.selected_index =
            (enemy_library_state.selected_index + 1) % enemy_library_state.library.entries.len();
        changed = true;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        current_entry_saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
            && save_enemy_library_if_valid(&enemy_library_state);
        enemy_library_state.selected_index = enemy_library_state.library.add_blank_entry();
        let selected_entry_id = enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
            .map(|entry| entry.id.clone());
        if let Some(entry_id) = selected_entry_id {
            enemy_library_state
                .entry_statuses
                .insert(entry_id, EnemyShipEntryValidationStatus::Valid);
        }
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

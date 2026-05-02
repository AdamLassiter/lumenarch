use bevy::prelude::*;

use crate::{
    netcode,
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    state::{
        EditorMode, EditorSessionState, EditorShip, EnemyNewButton, EnemyNextButton,
        EnemyPrevButton, EnemyShipLibraryState,
    },
};

pub(crate) fn enemy_library_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&EnemyPrevButton>,
            Option<&EnemyNextButton>,
            Option<&EnemyNewButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
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
                }
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                    rollback_state.editor_ship = editor_ship.ship.clone();
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

pub(crate) fn enemy_library_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    let mut changed = false;
    enemy_library_state.library.ensure_seeded();
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketLeft) {
        let len = enemy_library_state.library.entries.len();
        enemy_library_state.selected_index = (enemy_library_state.selected_index + len - 1) % len;
        changed = true;
    }
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketRight) {
        enemy_library_state.selected_index =
            (enemy_library_state.selected_index + 1) % enemy_library_state.library.entries.len();
        changed = true;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        enemy_library_state.selected_index = enemy_library_state.library.add_blank_entry();
        changed = true;
    }

    if changed
        && let Some(entry) = enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
    {
        editor_ship.ship = entry.ship.clone();
        rollback_state.editor_ship = editor_ship.ship.clone();
    }
}

mod textbox;
mod view;

use bevy::{log, prelude::*};

pub(crate) use self::{
    textbox::{edit_lobby_textboxes, focus_textbox_on_click, update_lobby_textboxes},
    view::{
        cleanup_lobby_ui,
        lobby_ui_missing,
        lobby_ui_present,
        spawn_lobby_ui,
        update_lobby_status_text,
    },
};
use super::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    netcode,
    state::{
        DebugEnemyEditorButton,
        EditorMode,
        EditorSessionState,
        FocusedTextBox,
        FrontendMode,
        JoinButton,
        LobbyCycleColorButton,
        LobbyCycleRoleButton,
        LocalPlayerProfile,
        TextBoxRoot,
    },
};

/// Processes lobby button presses so joining, hosting, role changes, and debug entry all share one UI path.
pub(crate) fn lobby_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&JoinButton>,
            Option<&DebugEnemyEditorButton>,
            Option<&LobbyCycleRoleButton>,
            Option<&LobbyCycleColorButton>,
            Option<&TextBoxRoot>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    config: Res<netcode::SessionConfig>,
    mut local_profile: ResMut<LocalPlayerProfile>,
    mut status: ResMut<netcode::SessionStatus>,
    mut bootstrap: ResMut<netcode::SessionBootstrapConfig>,
    mut lobby_runtime: ResMut<netcode::LobbyRuntime>,
    mut editor_session: ResMut<EditorSessionState>,
    mut focused_textbox: ResMut<FocusedTextBox>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    for (interaction, mut background, join, debug_enemy, cycle_role, cycle_color, textbox) in
        &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if textbox.is_some() {
                    *background = BackgroundColor(Color::srgb(0.20, 0.28, 0.40));
                    continue;
                }
                focused_textbox.field = None;
                focused_textbox.select_all = false;
                if join.is_some() {
                    *background = BackgroundColor(PRESSED_BUTTON);
                    if matches!(status.phase, netcode::SessionPhase::Lobby)
                        && status.role == Some(netcode::SessionRole::Host)
                    {
                        netcode::request_lobby_session_start(
                            &mut status,
                            &bootstrap,
                            lobby_runtime.as_ref(),
                        );
                    } else {
                        editor_session.mode = EditorMode::Player;
                        log::info!(
                            "Lobby join requested with session descriptor '{}'",
                            config.session_descriptor
                        );
                        netcode::begin_session_attempt(
                            &config,
                            &local_profile,
                            &mut status,
                            &mut bootstrap,
                            lobby_runtime.as_mut(),
                        );
                        commands.insert_resource(netcode::LocalPlayerHandle::default());
                    }
                } else if debug_enemy.is_some() {
                    *background = BackgroundColor(Color::srgb(0.36, 0.24, 0.16));
                    editor_session.mode = EditorMode::Enemy;
                    next_mode.set(FrontendMode::DebugEnemyEditor);
                    log::info!("Debug Enemy Editor button pressed");
                    log::info!("Switching to Editing mode");
                } else if cycle_role.is_some() {
                    *background = BackgroundColor(PRESSED_BUTTON);
                    local_profile.role = local_profile.role.cycle(1);
                } else if cycle_color.is_some() {
                    *background = BackgroundColor(PRESSED_BUTTON);
                    local_profile.cycle_color(1);
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(if textbox.is_some() {
                    Color::srgb(0.16, 0.22, 0.32)
                } else {
                    HOVERED_BUTTON
                });
            }
            Interaction::None => {
                *background = BackgroundColor(if textbox.is_some() {
                    Color::srgba(0.13, 0.17, 0.24, 1.0)
                } else if join.is_some() {
                    NORMAL_BUTTON
                } else if cycle_role.is_some() || cycle_color.is_some() {
                    Color::srgb(0.24, 0.32, 0.48)
                } else {
                    Color::srgb(0.46, 0.34, 0.22)
                });
            }
        }
    }
}

/// Mirrors key lobby actions onto the keyboard so session setup stays fast during iteration.
pub(crate) fn lobby_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<netcode::SessionConfig>,
    local_profile: Res<LocalPlayerProfile>,
    focused_textbox: Res<FocusedTextBox>,
    mut status: ResMut<netcode::SessionStatus>,
    mut bootstrap: ResMut<netcode::SessionBootstrapConfig>,
    mut lobby_runtime: ResMut<netcode::LobbyRuntime>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::Enter) {
        if matches!(status.phase, netcode::SessionPhase::Lobby)
            && status.role == Some(netcode::SessionRole::Host)
        {
            netcode::request_lobby_session_start(&mut status, &bootstrap, lobby_runtime.as_ref());
        } else {
            log::info!(
                "Lobby keyboard shortcut requested session start for descriptor '{}'",
                config.session_descriptor
            );
            netcode::begin_session_attempt(
                &config,
                &local_profile,
                &mut status,
                &mut bootstrap,
                lobby_runtime.as_mut(),
            );
        }
    }
}

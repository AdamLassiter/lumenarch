use bevy::{
    input::keyboard::{Key, KeyboardInput},
    log,
    prelude::*,
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
        FrontendMode,
        HostAddressText,
        JoinButton,
        JoinButtonText,
        LobbyRoot,
        StatusText,
    },
};

pub(crate) fn spawn_lobby_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<netcode::SessionConfig>,
    status: Res<netcode::SessionStatus>,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::NONE),
            LobbyRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(560.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.09, 0.12, 0.18, 0.94)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("LUMEN//ARCH"),
                        TextFont {
                            font: title_font.clone(),
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    panel.spawn((
                        Text::new(format!(
                            "Type a session descriptor, Backspace to delete, Enter to start.\nExamples: host@{} or client1@{}>{}",
                            super::DEFAULT_HOST_ADDR,
                            super::DEFAULT_CLIENT_ADDR,
                            super::DEFAULT_HOST_ADDR
                        )),
                        TextFont {
                            font: title_font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.74, 0.78, 0.86)),
                    ));

                    panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(14.0)),
                                border_radius: BorderRadius::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.13, 0.17, 0.24, 1.0)),
                        ))
                        .with_children(|field| {
                            field.spawn((
                                Text::new(host_address_line(&config.session_descriptor)),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                HostAddressText,
                            ));
                        });

                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(240.0),
                                height: Val::Px(52.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(NORMAL_BUTTON),
                            JoinButton,
                        ))
                        .with_child((
                            Text::new(join_button_label(&config.session_descriptor, &status)),
                            TextFont {
                                font: title_font,
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            JoinButtonText,
                        ));

                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(240.0),
                                height: Val::Px(44.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border_radius: BorderRadius::all(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.46, 0.34, 0.22)),
                            DebugEnemyEditorButton,
                        ))
                        .with_child((
                            Text::new("Debug Enemy Ships"),
                            TextFont {
                                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                    panel.spawn((
                        Text::new(lobby_status_line(&status, &config.session_descriptor)),
                        TextFont {
                            font: mono_font,
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.70, 0.78, 0.86)),
                        StatusText,
                    ));
                });
        });
}

pub(crate) fn edit_host_address(
    mut keyboard_events: MessageReader<KeyboardInput>,
    mut config: ResMut<netcode::SessionConfig>,
    status: Res<netcode::SessionStatus>,
) {
    if matches!(
        status.phase,
        netcode::SessionPhase::Connecting
            | netcode::SessionPhase::Lobby
            | netcode::SessionPhase::Starting
    ) {
        return;
    }

    if matches!(status.phase, netcode::SessionPhase::Failed(_))
        && config.session_descriptor.starts_with("host@")
    {
        config.session_descriptor = format!(
            "client1@{}>{}",
            super::DEFAULT_CLIENT_ADDR,
            super::DEFAULT_HOST_ADDR
        );
    }

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                config.session_descriptor.pop();
            }
            Key::Character(chars) if chars.chars().all(is_host_address_character) => {
                config.session_descriptor.push_str(chars);
            }
            _ => {}
        }
    }
}

pub(crate) fn update_host_address_text(
    config: Res<netcode::SessionConfig>,
    mut query: Query<&mut Text, With<HostAddressText>>,
) {
    if !config.is_changed() {
        return;
    }

    for mut text in &mut query {
        **text = host_address_line(&config.session_descriptor);
    }
}

pub(crate) fn lobby_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&JoinButton>,
            Option<&DebugEnemyEditorButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    config: Res<netcode::SessionConfig>,
    mut status: ResMut<netcode::SessionStatus>,
    mut bootstrap: ResMut<netcode::SessionBootstrapConfig>,
    mut lobby_runtime: ResMut<netcode::LobbyRuntime>,
    mut editor_session: ResMut<EditorSessionState>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    for (interaction, mut background, join, debug_enemy) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
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
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if join.is_some() {
                    NORMAL_BUTTON
                } else {
                    Color::srgb(0.46, 0.34, 0.22)
                });
            }
        }
    }
}

pub(crate) fn lobby_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<netcode::SessionConfig>,
    mut status: ResMut<netcode::SessionStatus>,
    mut bootstrap: ResMut<netcode::SessionBootstrapConfig>,
    mut lobby_runtime: ResMut<netcode::LobbyRuntime>,
) {
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
                &mut status,
                &mut bootstrap,
                lobby_runtime.as_mut(),
            );
        }
    }
}

pub(crate) fn update_lobby_status_text(
    status: Res<netcode::SessionStatus>,
    config: Res<netcode::SessionConfig>,
    mut status_query: Query<&mut Text, With<StatusText>>,
    mut join_button_query: Query<&mut Text, (With<JoinButtonText>, Without<StatusText>)>,
) {
    if !status.is_changed() && !config.is_changed() {
        return;
    }

    for mut text in &mut status_query {
        **text = lobby_status_line(&status, &config.session_descriptor);
    }

    for mut text in &mut join_button_query {
        **text = join_button_label(&config.session_descriptor, &status).to_string();
    }
}

pub(crate) fn cleanup_lobby_ui(mut commands: Commands, ui_query: Query<Entity, With<LobbyRoot>>) {
    for entity in &ui_query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn lobby_ui_missing(query: Query<Entity, With<LobbyRoot>>) -> bool {
    query.is_empty()
}

pub(crate) fn lobby_ui_present(query: Query<Entity, With<LobbyRoot>>) -> bool {
    !query.is_empty()
}

fn is_host_address_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '.' | ':' | '-' | '[' | ']' | '@' | '>' | ',')
}

fn host_address_line(server_addr: &str) -> String {
    format!("Session: {server_addr}")
}

fn join_button_label(server_addr: &str, status: &netcode::SessionStatus) -> &'static str {
    match (&status.phase, session_descriptor_role(server_addr)) {
        (netcode::SessionPhase::Lobby, Some(netcode::SessionRole::Host)) => "Start Lobby",
        (netcode::SessionPhase::Lobby, Some(netcode::SessionRole::Client)) => "Waiting for Host",
        (_, Some(netcode::SessionRole::Host)) => "Host Lobby",
        _ => "Join Lobby",
    }
}

fn lobby_status_line(status: &netcode::SessionStatus, server_addr: &str) -> String {
    let lobby_count = status
        .lobby_snapshot
        .as_ref()
        .map(|snapshot| snapshot.players.len());

    match &status.phase {
        netcode::SessionPhase::Idle => {
            format!("Ready to start a rollback session at {server_addr}.")
        }
        netcode::SessionPhase::Connecting => {
            format!("Opening lobby channel for {server_addr}...")
        }
        netcode::SessionPhase::Lobby => {
            let players = lobby_count.unwrap_or(1);
            if status.role == Some(netcode::SessionRole::Host) {
                format!(
                    "Lobby active for {server_addr}. {players} player(s) connected. Press Start Lobby or Enter once everyone is present."
                )
            } else {
                format!(
                    "Lobby active for {server_addr}. {players} player(s) connected. Waiting for host to start the match."
                )
            }
        }
        netcode::SessionPhase::Starting => {
            let players = lobby_count.unwrap_or(status.total_players.max(1));
            format!(
                "Lobby locked with {players} player(s). Starting deterministic rollback session..."
            )
        }
        netcode::SessionPhase::Connected => {
            "Rollback session running. Loading deterministic game state...".to_string()
        }
        netcode::SessionPhase::Failed(message) => format!("Session bootstrap failed: {message}"),
    }
}

fn session_descriptor_role(server_addr: &str) -> Option<netcode::SessionRole> {
    if server_addr.starts_with("host@") {
        Some(netcode::SessionRole::Host)
    } else if server_addr.starts_with("client") {
        Some(netcode::SessionRole::Client)
    } else {
        None
    }
}

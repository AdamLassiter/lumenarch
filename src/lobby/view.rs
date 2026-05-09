use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use super::{NORMAL_BUTTON, netcode};
use crate::{
    DEFAULT_CLIENT_ADDR,
    DEFAULT_HOST_ADDR,
    state::{
        DebugEnemyEditorButton,
        FocusedTextBox,
        JoinButton,
        JoinButtonText,
        LobbyColorText,
        LobbyCycleColorButton,
        LobbyCycleRoleButton,
        LobbyRoleText,
        LobbyRoot,
        LocalPlayerProfile,
        StatusText,
        TextBoxField,
        TextBoxRoot,
        TextBoxText,
    },
};

pub(crate) fn spawn_lobby_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<netcode::SessionConfig>,
    status: Res<netcode::SessionStatus>,
    local_profile: Res<LocalPlayerProfile>,
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
                    spawn_labeled_textbox(
                        panel,
                        "Session",
                        TextBoxField::SessionDescriptor,
                        &mono_font,
                        &config.session_descriptor,
                    );
                    panel.spawn((
                        Text::new(format!(
                            "Examples: host@{} or client1@{}>{}",
                            DEFAULT_HOST_ADDR, DEFAULT_CLIENT_ADDR, DEFAULT_HOST_ADDR
                        )),
                        TextFont {
                            font: title_font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.74, 0.78, 0.86)),
                    ));

                    spawn_labeled_textbox(
                        panel,
                        "Player Name",
                        TextBoxField::PlayerName,
                        &mono_font,
                        &local_profile.name,
                    );

                    panel
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!(
                                    "Role: {} ({})",
                                    local_profile.role.as_str(),
                                    local_profile.starting_suit().as_str()
                                )),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                LobbyRoleText,
                            ));
                            cycle_button(row, "Cycle Role", &mono_font, LobbyCycleRoleButton);
                        });

                    panel
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },))
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!(
                                    "Avatar Color: {}",
                                    color_label(local_profile.color_index)
                                )),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(local_profile.color()),
                                LobbyColorText,
                            ));
                            cycle_button(row, "Cycle Color", &mono_font, LobbyCycleColorButton);
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

pub(crate) fn update_lobby_status_text(
    status: Res<netcode::SessionStatus>,
    config: Res<netcode::SessionConfig>,
    local_profile: Res<LocalPlayerProfile>,
    mut text_queries: ParamSet<(
        Query<&mut Text, With<StatusText>>,
        Query<&mut Text, (With<JoinButtonText>, Without<StatusText>)>,
        Query<&mut Text, (With<LobbyRoleText>, Without<StatusText>)>,
        Query<(&mut Text, &mut TextColor), (With<LobbyColorText>, Without<StatusText>)>,
    )>,
) {
    if !status.is_changed() && !config.is_changed() && !local_profile.is_changed() {
        return;
    }

    for mut text in &mut text_queries.p0() {
        **text = lobby_status_line(&status, &config.session_descriptor);
    }
    for mut text in &mut text_queries.p1() {
        **text = join_button_label(&config.session_descriptor, &status).to_string();
    }
    for mut text in &mut text_queries.p2() {
        **text = format!(
            "Role: {} ({})",
            local_profile.role.as_str(),
            local_profile.starting_suit().as_str()
        );
    }
    for (mut text, mut color) in &mut text_queries.p3() {
        **text = format!("Avatar Color: {}", color_label(local_profile.color_index));
        color.0 = local_profile.color();
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

pub(super) fn spawn_textbox(
    parent: &mut ChildSpawnerCommands<'_>,
    field: TextBoxField,
    font: &Handle<Font>,
    initial_value: &str,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.13, 0.17, 0.24, 1.0)),
            TextBoxRoot { field },
        ))
        .with_children(|field_parent| {
            field_parent.spawn((
                Text::new(initial_value),
                TextFont {
                    font: font.clone(),
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextBoxText { field },
            ));
        });
}

pub(super) fn format_textbox_value(
    value: &str,
    field: TextBoxField,
    focused_textbox: &FocusedTextBox,
) -> String {
    if focused_textbox.field != Some(field) {
        return value.to_string();
    }

    let mut display = if focused_textbox.select_all {
        format!("[{value}]")
    } else {
        value.to_string()
    };
    let cursor_index = focused_textbox.cursor_index.min(value.chars().count());
    let insert_at = super::textbox::char_to_byte_index(
        &display,
        if focused_textbox.select_all {
            display.chars().count()
        } else {
            cursor_index
        },
    );
    display.insert(insert_at, '|');
    display
}

pub(super) fn join_button_label(
    server_addr: &str,
    status: &netcode::SessionStatus,
) -> &'static str {
    match (&status.phase, session_descriptor_role(server_addr)) {
        (netcode::SessionPhase::Lobby, Some(netcode::SessionRole::Host)) => "Start Lobby",
        (netcode::SessionPhase::Lobby, Some(netcode::SessionRole::Client)) => "Waiting for Host",
        (_, Some(netcode::SessionRole::Host)) => "Host Lobby",
        _ => "Join Lobby",
    }
}

pub(super) fn color_label(index: u8) -> &'static str {
    match index % 8 {
        0 => "Mint",
        1 => "Sky",
        2 => "Sand",
        3 => "Rose",
        4 => "Amber",
        5 => "Steel",
        6 => "Lime",
        _ => "Coral",
    }
}

pub(super) fn lobby_status_line(status: &netcode::SessionStatus, server_addr: &str) -> String {
    let lobby_count = status
        .lobby_snapshot
        .as_ref()
        .map(|snapshot| snapshot.players.len());
    let roster = status
        .lobby_snapshot
        .as_ref()
        .map(|snapshot| {
            snapshot
                .players
                .iter()
                .map(|player| {
                    format!(
                        "{}{} ({})",
                        player.profile.name,
                        if player.is_host { " [host]" } else { "" },
                        player.profile.role.as_str()
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|players| !players.is_empty());

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
                    "Lobby active for {server_addr}. {players} player(s) connected. Press Start Lobby or Enter once everyone is present.{}",
                    roster
                        .as_ref()
                        .map(|players| format!("\nCrew: {players}"))
                        .unwrap_or_default()
                )
            } else {
                format!(
                    "Lobby active for {server_addr}. {players} player(s) connected. Waiting for host to start the match.{}",
                    roster
                        .as_ref()
                        .map(|players| format!("\nCrew: {players}"))
                        .unwrap_or_default()
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

pub(super) fn session_descriptor_role(server_addr: &str) -> Option<netcode::SessionRole> {
    if server_addr.starts_with("host@") {
        Some(netcode::SessionRole::Host)
    } else if server_addr.starts_with("client") {
        Some(netcode::SessionRole::Client)
    } else {
        None
    }
}

fn spawn_labeled_textbox(
    panel: &mut ChildSpawnerCommands<'_>,
    label: &str,
    field: TextBoxField,
    font: &Handle<Font>,
    initial_value: &str,
) {
    panel
        .spawn((Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },))
        .with_children(|field_parent| {
            field_parent.spawn((
                Text::new(label),
                TextFont {
                    font: font.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.74, 0.78, 0.86)),
            ));
            spawn_textbox(field_parent, field, font, initial_value);
        });
}

fn cycle_button<T: Component>(
    row: &mut ChildSpawnerCommands<'_>,
    label: &str,
    font: &Handle<Font>,
    marker: T,
) {
    row.spawn((
        Button,
        Node {
            width: Val::Px(140.0),
            height: Val::Px(34.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.24, 0.32, 0.48)),
        marker,
    ))
    .with_child((
        Text::new(label),
        TextFont {
            font: font.clone(),
            font_size: 15.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

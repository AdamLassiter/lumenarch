use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use super::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    net,
    state::{
        ConnectionConfig,
        ConnectionMailbox,
        ConnectionPhase,
        ConnectionStatus,
        HostAddressText,
        JoinButton,
        MenuRoot,
        StatusText,
    },
};

pub(crate) fn spawn_menu_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<ConnectionConfig>,
    status: Res<ConnectionStatus>,
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
            MenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(560.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.09, 0.12, 0.18, 0.94)),
                    BorderRadius::all(Val::Px(14.0)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("LUMEN//ARCH\nShip Editor Utility"),
                        TextFont {
                            font: title_font.clone(),
                            font_size: 34.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    panel.spawn((
                        Text::new(
                            "Type to edit the host address, Backspace to delete, Enter to join.",
                        ),
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
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.13, 0.17, 0.24, 1.0)),
                            BorderRadius::all(Val::Px(10.0)),
                        ))
                        .with_children(|field| {
                            field.spawn((
                                Text::new(host_address_line(&config.server_addr)),
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
                                ..default()
                            },
                            BorderRadius::all(Val::Px(10.0)),
                            BackgroundColor(NORMAL_BUTTON),
                            JoinButton,
                        ))
                        .with_child((
                            Text::new("Join Host"),
                            TextFont {
                                font: title_font,
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));

                    panel.spawn((
                        Text::new(menu_status_line(&status.phase, &config.server_addr)),
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
    mut keyboard_events: EventReader<KeyboardInput>,
    mut config: ResMut<ConnectionConfig>,
    status: Res<ConnectionStatus>,
) {
    if matches!(status.phase, ConnectionPhase::Connecting) {
        return;
    }

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Backspace => {
                config.server_addr.pop();
            }
            Key::Character(chars) => {
                if chars.chars().all(is_host_address_character) {
                    config.server_addr.push_str(chars);
                }
            }
            _ => {}
        }
    }

    if config.server_addr.is_empty() {
        config.server_addr = super::DEFAULT_HOST_ADDR.to_string();
    }
}

pub(crate) fn update_host_address_text(
    config: Res<ConnectionConfig>,
    mut query: Query<&mut Text, With<HostAddressText>>,
) {
    if !config.is_changed() {
        return;
    }

    for mut text in &mut query {
        **text = host_address_line(&config.server_addr);
    }
}

pub(crate) fn menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<JoinButton>),
    >,
    config: Res<ConnectionConfig>,
    mut status: ResMut<ConnectionStatus>,
    mailbox: Res<ConnectionMailbox>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                net::begin_connection_attempt(&config.server_addr, &mut status, &mailbox);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(NORMAL_BUTTON);
            }
        }
    }
}

pub(crate) fn menu_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<ConnectionConfig>,
    mut status: ResMut<ConnectionStatus>,
    mailbox: Res<ConnectionMailbox>,
) {
    if keys.just_pressed(KeyCode::Enter) {
        net::begin_connection_attempt(&config.server_addr, &mut status, &mailbox);
    }
}

pub(crate) fn update_menu_status_text(
    status: Res<ConnectionStatus>,
    config: Res<ConnectionConfig>,
    mut query: Query<&mut Text, With<StatusText>>,
) {
    if !status.is_changed() && !config.is_changed() {
        return;
    }

    for mut text in &mut query {
        **text = menu_status_line(&status.phase, &config.server_addr);
    }
}

pub(crate) fn cleanup_menu_ui(mut commands: Commands, query: Query<Entity, With<MenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn is_host_address_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | ':' | '-' | '[' | ']')
}

fn host_address_line(server_addr: &str) -> String {
    format!("Host Address: {server_addr}")
}

fn menu_status_line(phase: &ConnectionPhase, server_addr: &str) -> String {
    match phase {
        ConnectionPhase::Idle => format!("Ready to connect to {server_addr}."),
        ConnectionPhase::Connecting => format!("Connecting to {server_addr}..."),
        ConnectionPhase::Connected => "Connected. Loading ship editor...".to_string(),
        ConnectionPhase::Failed(message) => format!("Connection failed: {message}"),
    }
}

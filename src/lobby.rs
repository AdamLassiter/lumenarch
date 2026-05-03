use bevy::{
    ecs::hierarchy::ChildSpawnerCommands,
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
        FocusedTextBox,
        FrontendMode,
        JoinButton,
        JoinButtonText,
        LobbyColorText,
        LobbyCycleColorButton,
        LobbyCycleRoleButton,
        LobbyRoleText,
        LobbyRoot,
        LocalPlayerProfile,
        StatusText,
        TextBoxClipboard,
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
                            "Click a textbox to edit it.\nExamples: host@{} or client1@{}>{}",
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
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|field| {
                            field.spawn((
                                Text::new("Session"),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.74, 0.78, 0.86)),
                            ));
                            spawn_textbox(
                                field,
                                TextBoxField::SessionDescriptor,
                                &mono_font,
                                &config.session_descriptor,
                            );
                        });

                    panel
                        .spawn((Node {
                            width: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        },))
                        .with_children(|field| {
                            field.spawn((
                                Text::new("Player Name"),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.74, 0.78, 0.86)),
                            ));
                            spawn_textbox(
                                field,
                                TextBoxField::PlayerName,
                                &mono_font,
                                &local_profile.name,
                            );
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
                                LobbyCycleRoleButton,
                            ))
                            .with_child((
                                Text::new("Cycle Role"),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
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
                                LobbyCycleColorButton,
                            ))
                            .with_child((
                                Text::new("Cycle Color"),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 15.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
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

pub(crate) fn focus_textbox_on_click(
    mut interaction_query: Query<
        (&Interaction, &TextBoxRoot),
        (Changed<Interaction>, With<Button>),
    >,
    mut focused_textbox: ResMut<FocusedTextBox>,
) {
    for (interaction, textbox) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            focused_textbox.field = Some(textbox.field);
            focused_textbox.cursor_index = usize::MAX;
            focused_textbox.select_all = false;
        }
    }
}

pub(crate) fn edit_lobby_textboxes(
    mut keyboard_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<netcode::SessionConfig>,
    mut local_profile: ResMut<LocalPlayerProfile>,
    mut focused_textbox: ResMut<FocusedTextBox>,
    mut clipboard: ResMut<TextBoxClipboard>,
    status: Res<netcode::SessionStatus>,
) {
    let Some(field) = focused_textbox.field else {
        return;
    };
    let lobby_locked = matches!(
        status.phase,
        netcode::SessionPhase::Connecting
            | netcode::SessionPhase::Lobby
            | netcode::SessionPhase::Starting
    );
    if matches!(status.phase, netcode::SessionPhase::Failed(_))
        && config.session_descriptor.starts_with("host@")
    {
        config.session_descriptor = format!(
            "client1@{}>{}",
            super::DEFAULT_CLIENT_ADDR,
            super::DEFAULT_HOST_ADDR
        );
    }

    let ctrl_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    normalize_cursor(field, &config, &local_profile, &mut focused_textbox);

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::ArrowLeft => move_cursor_left(&mut focused_textbox),
            Key::ArrowRight => {
                move_cursor_right(field, &config, &local_profile, &mut focused_textbox)
            }
            Key::Home => {
                focused_textbox.cursor_index = 0;
                focused_textbox.select_all = false;
            }
            Key::End => {
                focused_textbox.cursor_index =
                    field_value(field, &config, &local_profile).chars().count();
                focused_textbox.select_all = false;
            }
            Key::Backspace => backspace_textbox(
                field,
                &mut config,
                &mut local_profile,
                &mut focused_textbox,
                lobby_locked,
            ),
            Key::Delete => delete_textbox(
                field,
                &mut config,
                &mut local_profile,
                &mut focused_textbox,
                lobby_locked,
            ),
            Key::Character(chars) if ctrl_pressed && chars.eq_ignore_ascii_case("a") => {
                focused_textbox.cursor_index =
                    field_value(field, &config, &local_profile).chars().count();
                focused_textbox.select_all = true;
            }
            Key::Character(chars)
                if ctrl_pressed
                    && chars.eq_ignore_ascii_case("c")
                    && focused_textbox.select_all =>
            {
                clipboard.contents = field_value(field, &config, &local_profile).to_string();
            }
            Key::Character(chars)
                if ctrl_pressed
                    && chars.eq_ignore_ascii_case("x")
                    && focused_textbox.select_all
                    && field_is_editable(field, lobby_locked) =>
            {
                clipboard.contents = field_value(field, &config, &local_profile).to_string();
                clear_field(field, &mut config, &mut local_profile);
                focused_textbox.cursor_index = 0;
                focused_textbox.select_all = false;
            }
            Key::Character(chars)
                if ctrl_pressed
                    && chars.eq_ignore_ascii_case("v")
                    && field_is_editable(field, lobby_locked)
                    && !clipboard.contents.is_empty() =>
            {
                insert_text(
                    field,
                    &mut config,
                    &mut local_profile,
                    &mut focused_textbox,
                    &clipboard.contents.clone(),
                );
            }
            Key::Character(chars)
                if !ctrl_pressed && field_accepts_input(field, lobby_locked, chars) =>
            {
                insert_text(
                    field,
                    &mut config,
                    &mut local_profile,
                    &mut focused_textbox,
                    chars,
                );
            }
            _ => {}
        }
    }
}

pub(crate) fn update_lobby_textboxes(
    config: Res<netcode::SessionConfig>,
    local_profile: Res<LocalPlayerProfile>,
    focused_textbox: Res<FocusedTextBox>,
    mut query: Query<(&TextBoxText, &mut Text)>,
) {
    if !config.is_changed() && !local_profile.is_changed() && !focused_textbox.is_changed() {
        return;
    }

    for (textbox, mut text) in &mut query {
        let value = field_value(textbox.field, &config, &local_profile);
        **text = format_textbox_value(value, textbox.field, &focused_textbox);
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

fn is_host_address_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '.' | ':' | '-' | '[' | ']' | '@' | '>' | ',')
}

fn spawn_textbox(
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

fn format_textbox_value(
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
    let insert_at = char_to_byte_index(
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

fn normalize_cursor(
    field: TextBoxField,
    config: &netcode::SessionConfig,
    local_profile: &LocalPlayerProfile,
    focused_textbox: &mut FocusedTextBox,
) {
    let len = field_value(field, config, local_profile).chars().count();
    if focused_textbox.cursor_index == usize::MAX {
        focused_textbox.cursor_index = len;
    } else {
        focused_textbox.cursor_index = focused_textbox.cursor_index.min(len);
    }
}

fn move_cursor_left(focused_textbox: &mut FocusedTextBox) {
    if focused_textbox.select_all {
        focused_textbox.cursor_index = 0;
        focused_textbox.select_all = false;
    } else {
        focused_textbox.cursor_index = focused_textbox.cursor_index.saturating_sub(1);
    }
}

fn move_cursor_right(
    field: TextBoxField,
    config: &netcode::SessionConfig,
    local_profile: &LocalPlayerProfile,
    focused_textbox: &mut FocusedTextBox,
) {
    let len = field_value(field, config, local_profile).chars().count();
    if focused_textbox.select_all {
        focused_textbox.cursor_index = len;
        focused_textbox.select_all = false;
    } else {
        focused_textbox.cursor_index = (focused_textbox.cursor_index + 1).min(len);
    }
}

fn backspace_textbox(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    focused_textbox: &mut FocusedTextBox,
    lobby_locked: bool,
) {
    if !field_is_editable(field, lobby_locked) {
        return;
    }
    if focused_textbox.select_all {
        clear_field(field, config, local_profile);
        focused_textbox.cursor_index = 0;
        focused_textbox.select_all = false;
        return;
    }
    if focused_textbox.cursor_index == 0 {
        return;
    }
    let current = field_value(field, config, local_profile).to_string();
    let start = char_to_byte_index(&current, focused_textbox.cursor_index - 1);
    let end = char_to_byte_index(&current, focused_textbox.cursor_index);
    let mut next = current;
    next.replace_range(start..end, "");
    set_field_value(field, config, local_profile, next);
    focused_textbox.cursor_index -= 1;
}

fn delete_textbox(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    focused_textbox: &mut FocusedTextBox,
    lobby_locked: bool,
) {
    if !field_is_editable(field, lobby_locked) {
        return;
    }
    if focused_textbox.select_all {
        clear_field(field, config, local_profile);
        focused_textbox.cursor_index = 0;
        focused_textbox.select_all = false;
        return;
    }
    let current = field_value(field, config, local_profile).to_string();
    let len = current.chars().count();
    if focused_textbox.cursor_index >= len {
        return;
    }
    let start = char_to_byte_index(&current, focused_textbox.cursor_index);
    let end = char_to_byte_index(&current, focused_textbox.cursor_index + 1);
    let mut next = current;
    next.replace_range(start..end, "");
    set_field_value(field, config, local_profile, next);
}

fn insert_text(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    focused_textbox: &mut FocusedTextBox,
    inserted_text: &str,
) {
    let sanitized = sanitize_textbox_input(field, inserted_text);
    if sanitized.is_empty() {
        return;
    }
    let mut current = field_value(field, config, local_profile).to_string();
    if focused_textbox.select_all {
        current.clear();
        focused_textbox.cursor_index = 0;
        focused_textbox.select_all = false;
    }
    let cursor = focused_textbox.cursor_index.min(current.chars().count());
    let byte_index = char_to_byte_index(&current, cursor);
    current.insert_str(byte_index, &sanitized);
    if matches!(field, TextBoxField::PlayerName) {
        let truncated: String = current.chars().take(18).collect();
        let inserted_count = sanitized.chars().count();
        set_field_value(field, config, local_profile, truncated);
        focused_textbox.cursor_index = (cursor + inserted_count)
            .min(field_value(field, config, local_profile).chars().count());
    } else {
        focused_textbox.cursor_index = cursor + sanitized.chars().count();
        set_field_value(field, config, local_profile, current);
    }
}

fn field_accepts_input(field: TextBoxField, lobby_locked: bool, chars: &str) -> bool {
    field_is_editable(field, lobby_locked) && !sanitize_textbox_input(field, chars).is_empty()
}

fn field_is_editable(field: TextBoxField, lobby_locked: bool) -> bool {
    match field {
        TextBoxField::SessionDescriptor => !lobby_locked,
        TextBoxField::PlayerName => true,
    }
}

fn sanitize_textbox_input(field: TextBoxField, chars: &str) -> String {
    chars
        .chars()
        .filter(|character| match field {
            TextBoxField::SessionDescriptor => is_host_address_character(*character),
            TextBoxField::PlayerName => {
                character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-')
            }
        })
        .collect()
}

fn field_value<'a>(
    field: TextBoxField,
    config: &'a netcode::SessionConfig,
    local_profile: &'a LocalPlayerProfile,
) -> &'a str {
    match field {
        TextBoxField::SessionDescriptor => &config.session_descriptor,
        TextBoxField::PlayerName => &local_profile.name,
    }
}

fn set_field_value(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    value: String,
) {
    match field {
        TextBoxField::SessionDescriptor => config.session_descriptor = value,
        TextBoxField::PlayerName => local_profile.name = value,
    }
}

fn clear_field(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
) {
    set_field_value(field, config, local_profile, String::new());
}

fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn join_button_label(server_addr: &str, status: &netcode::SessionStatus) -> &'static str {
    match (&status.phase, session_descriptor_role(server_addr)) {
        (netcode::SessionPhase::Lobby, Some(netcode::SessionRole::Host)) => "Start Lobby",
        (netcode::SessionPhase::Lobby, Some(netcode::SessionRole::Client)) => "Waiting for Host",
        (_, Some(netcode::SessionRole::Host)) => "Host Lobby",
        _ => "Join Lobby",
    }
}

fn color_label(index: u8) -> &'static str {
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

fn lobby_status_line(status: &netcode::SessionStatus, server_addr: &str) -> String {
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

fn session_descriptor_role(server_addr: &str) -> Option<netcode::SessionRole> {
    if server_addr.starts_with("host@") {
        Some(netcode::SessionRole::Host)
    } else if server_addr.starts_with("client") {
        Some(netcode::SessionRole::Client)
    } else {
        None
    }
}

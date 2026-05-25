use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

use super::{netcode, view::format_textbox_value};
use crate::state::{
    EditorMode,
    EditorSessionState,
    EditorShip,
    EnemyEditorState,
    EnemyShipLibraryState,
    FocusedTextBox,
    LocalPlayerProfile,
    SectorState,
    TextBoxClipboard,
    TextBoxField,
    TextBoxRoot,
    TextBoxText,
};

/// Moves focus into the clicked lobby textbox so keyboard editing goes to the intended field.
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

/// Clears textbox focus when another UI button is pressed so editor and lobby hotkeys can resume.
pub(crate) fn clear_textbox_focus_on_non_textbox_click(
    mut interaction_query: Query<
        (&Interaction, Option<&TextBoxRoot>),
        (Changed<Interaction>, With<Button>),
    >,
    mut focused_textbox: ResMut<FocusedTextBox>,
) {
    for (interaction, textbox) in &mut interaction_query {
        if *interaction == Interaction::Pressed && textbox.is_none() {
            focused_textbox.field = None;
            focused_textbox.select_all = false;
        }
    }
}

/// Applies keyboard text editing to the focused lobby field so session setup stays in-game.
pub(crate) fn edit_lobby_textboxes(
    mut keyboard_events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<netcode::SessionConfig>,
    mut local_profile: ResMut<LocalPlayerProfile>,
    mut editor_ship: ResMut<EditorShip>,
    editor_session: Res<EditorSessionState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    sector_state: Res<SectorState>,
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
    let ctrl_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    normalize_cursor(
        field,
        &config,
        &local_profile,
        &editor_ship,
        &mut focused_textbox,
    );

    for event in keyboard_events.read() {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::ArrowLeft => move_cursor_left(&mut focused_textbox),
            Key::ArrowRight => move_cursor_right(
                field,
                &config,
                &local_profile,
                &editor_ship,
                &mut focused_textbox,
            ),
            Key::Home => {
                focused_textbox.cursor_index = 0;
                focused_textbox.select_all = false;
            }
            Key::End => {
                focused_textbox.cursor_index =
                    field_value(field, &config, &local_profile, &editor_ship)
                        .chars()
                        .count();
                focused_textbox.select_all = false;
            }
            Key::Backspace => backspace_textbox(
                field,
                &mut config,
                &mut local_profile,
                &mut editor_ship,
                &editor_session,
                &mut enemy_editor_state,
                &mut enemy_library_state,
                &sector_state,
                &mut focused_textbox,
                lobby_locked,
            ),
            Key::Delete => delete_textbox(
                field,
                &mut config,
                &mut local_profile,
                &mut editor_ship,
                &editor_session,
                &mut enemy_editor_state,
                &mut enemy_library_state,
                &sector_state,
                &mut focused_textbox,
                lobby_locked,
            ),
            Key::Character(chars) if ctrl_pressed && chars.eq_ignore_ascii_case("a") => {
                focused_textbox.cursor_index =
                    field_value(field, &config, &local_profile, &editor_ship)
                        .chars()
                        .count();
                focused_textbox.select_all = true;
            }
            Key::Character(chars)
                if ctrl_pressed
                    && chars.eq_ignore_ascii_case("c")
                    && focused_textbox.select_all =>
            {
                set_textbox_clipboard(
                    &mut clipboard,
                    field_value(field, &config, &local_profile, &editor_ship).to_string(),
                );
            }
            Key::Character(chars)
                if ctrl_pressed
                    && chars.eq_ignore_ascii_case("x")
                    && focused_textbox.select_all
                    && field_is_editable(field, lobby_locked) =>
            {
                set_textbox_clipboard(
                    &mut clipboard,
                    field_value(field, &config, &local_profile, &editor_ship).to_string(),
                );
                clear_field(
                    field,
                    &mut config,
                    &mut local_profile,
                    &mut editor_ship,
                    &editor_session,
                    &mut enemy_editor_state,
                    &mut enemy_library_state,
                    &sector_state,
                );
                focused_textbox.cursor_index = 0;
                focused_textbox.select_all = false;
            }
            Key::Character(chars)
                if ctrl_pressed
                    && chars.eq_ignore_ascii_case("v")
                    && field_is_editable(field, lobby_locked) =>
            {
                if let Some(contents) = get_textbox_clipboard(&clipboard) {
                    insert_text(
                        field,
                        &mut config,
                        &mut local_profile,
                        &mut editor_ship,
                        &editor_session,
                        &mut enemy_editor_state,
                        &mut enemy_library_state,
                        &sector_state,
                        &mut focused_textbox,
                        &contents,
                    );
                }
            }
            Key::Space if field_accepts_input(field, lobby_locked, " ") => {
                insert_text(
                    field,
                    &mut config,
                    &mut local_profile,
                    &mut editor_ship,
                    &editor_session,
                    &mut enemy_editor_state,
                    &mut enemy_library_state,
                    &sector_state,
                    &mut focused_textbox,
                    " ",
                );
            }
            Key::Character(chars)
                if !ctrl_pressed && field_accepts_input(field, lobby_locked, chars) =>
            {
                insert_text(
                    field,
                    &mut config,
                    &mut local_profile,
                    &mut editor_ship,
                    &editor_session,
                    &mut enemy_editor_state,
                    &mut enemy_library_state,
                    &sector_state,
                    &mut focused_textbox,
                    chars,
                );
            }
            _ => {}
        }
    }
}

/// Rebuilds textbox text and caret visuals so the lobby editor stays in sync with state.
pub(crate) fn update_lobby_textboxes(
    config: Res<netcode::SessionConfig>,
    local_profile: Res<LocalPlayerProfile>,
    editor_ship: Res<EditorShip>,
    focused_textbox: Res<FocusedTextBox>,
    mut query: Query<(&TextBoxText, &mut Text)>,
) {
    if !config.is_changed()
        && !local_profile.is_changed()
        && !editor_ship.is_changed()
        && !focused_textbox.is_changed()
    {
        return;
    }

    for (textbox, mut text) in &mut query {
        let value = field_value(textbox.field, &config, &local_profile, &editor_ship);
        **text = format_textbox_value(value, textbox.field, &focused_textbox);
    }
}

pub(super) fn char_to_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(index, _)| index)
        .unwrap_or(text.len())
}

fn normalize_cursor(
    field: TextBoxField,
    config: &netcode::SessionConfig,
    local_profile: &LocalPlayerProfile,
    editor_ship: &EditorShip,
    focused_textbox: &mut FocusedTextBox,
) {
    let len = field_value(field, config, local_profile, editor_ship)
        .chars()
        .count();
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
    editor_ship: &EditorShip,
    focused_textbox: &mut FocusedTextBox,
) {
    let len = field_value(field, config, local_profile, editor_ship)
        .chars()
        .count();
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
    editor_ship: &mut EditorShip,
    editor_session: &EditorSessionState,
    enemy_editor_state: &mut EnemyEditorState,
    enemy_library_state: &mut EnemyShipLibraryState,
    sector_state: &SectorState,
    focused_textbox: &mut FocusedTextBox,
    lobby_locked: bool,
) {
    if !field_is_editable(field, lobby_locked) {
        return;
    }
    if focused_textbox.select_all {
        clear_field(
            field,
            config,
            local_profile,
            editor_ship,
            editor_session,
            enemy_editor_state,
            enemy_library_state,
            sector_state,
        );
        focused_textbox.cursor_index = 0;
        focused_textbox.select_all = false;
        return;
    }
    if focused_textbox.cursor_index == 0 {
        return;
    }
    let current = field_value(field, config, local_profile, editor_ship).to_string();
    let start = char_to_byte_index(&current, focused_textbox.cursor_index - 1);
    let end = char_to_byte_index(&current, focused_textbox.cursor_index);
    let mut next = current;
    next.replace_range(start..end, "");
    set_field_value(
        field,
        config,
        local_profile,
        editor_ship,
        editor_session,
        enemy_editor_state,
        enemy_library_state,
        sector_state,
        next,
    );
    focused_textbox.cursor_index -= 1;
}

fn delete_textbox(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    editor_ship: &mut EditorShip,
    editor_session: &EditorSessionState,
    enemy_editor_state: &mut EnemyEditorState,
    enemy_library_state: &mut EnemyShipLibraryState,
    sector_state: &SectorState,
    focused_textbox: &mut FocusedTextBox,
    lobby_locked: bool,
) {
    if !field_is_editable(field, lobby_locked) {
        return;
    }
    if focused_textbox.select_all {
        clear_field(
            field,
            config,
            local_profile,
            editor_ship,
            editor_session,
            enemy_editor_state,
            enemy_library_state,
            sector_state,
        );
        focused_textbox.cursor_index = 0;
        focused_textbox.select_all = false;
        return;
    }
    let current = field_value(field, config, local_profile, editor_ship).to_string();
    let len = current.chars().count();
    if focused_textbox.cursor_index >= len {
        return;
    }
    let start = char_to_byte_index(&current, focused_textbox.cursor_index);
    let end = char_to_byte_index(&current, focused_textbox.cursor_index + 1);
    let mut next = current;
    next.replace_range(start..end, "");
    set_field_value(
        field,
        config,
        local_profile,
        editor_ship,
        editor_session,
        enemy_editor_state,
        enemy_library_state,
        sector_state,
        next,
    );
}

fn insert_text(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    editor_ship: &mut EditorShip,
    editor_session: &EditorSessionState,
    enemy_editor_state: &mut EnemyEditorState,
    enemy_library_state: &mut EnemyShipLibraryState,
    sector_state: &SectorState,
    focused_textbox: &mut FocusedTextBox,
    inserted_text: &str,
) {
    let sanitized = sanitize_textbox_input(field, inserted_text);
    if sanitized.is_empty() {
        return;
    }
    let mut current = field_value(field, config, local_profile, editor_ship).to_string();
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
        set_field_value(
            field,
            config,
            local_profile,
            editor_ship,
            editor_session,
            enemy_editor_state,
            enemy_library_state,
            sector_state,
            truncated,
        );
        focused_textbox.cursor_index = (cursor + inserted_count).min(
            field_value(field, config, local_profile, editor_ship)
                .chars()
                .count(),
        );
    } else {
        focused_textbox.cursor_index = cursor + sanitized.chars().count();
        set_field_value(
            field,
            config,
            local_profile,
            editor_ship,
            editor_session,
            enemy_editor_state,
            enemy_library_state,
            sector_state,
            current,
        );
    }
}

fn field_accepts_input(field: TextBoxField, lobby_locked: bool, chars: &str) -> bool {
    field_is_editable(field, lobby_locked) && !sanitize_textbox_input(field, chars).is_empty()
}

fn field_is_editable(field: TextBoxField, lobby_locked: bool) -> bool {
    match field {
        TextBoxField::SessionDescriptor => !lobby_locked,
        TextBoxField::PlayerName => true,
        TextBoxField::ShipName => true,
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
            TextBoxField::ShipName => {
                character.is_ascii_alphanumeric() || matches!(character, ' ' | '_' | '-')
            }
        })
        .collect()
}

fn field_value<'a>(
    field: TextBoxField,
    config: &'a netcode::SessionConfig,
    local_profile: &'a LocalPlayerProfile,
    editor_ship: &'a EditorShip,
) -> &'a str {
    match field {
        TextBoxField::SessionDescriptor => &config.session_descriptor,
        TextBoxField::PlayerName => &local_profile.name,
        TextBoxField::ShipName => &editor_ship.ship.name,
    }
}

fn set_field_value(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    editor_ship: &mut EditorShip,
    editor_session: &EditorSessionState,
    enemy_editor_state: &mut EnemyEditorState,
    enemy_library_state: &mut EnemyShipLibraryState,
    _sector_state: &SectorState,
    value: String,
) {
    match field {
        TextBoxField::SessionDescriptor => config.session_descriptor = value,
        TextBoxField::PlayerName => local_profile.name = value,
        TextBoxField::ShipName => {
            let trimmed = value.trim().to_string();
            editor_ship.ship.name = value;
            if editor_session.mode == EditorMode::Enemy && !trimmed.is_empty() {
                sync_selected_enemy_name(
                    &trimmed,
                    editor_ship,
                    enemy_editor_state,
                    enemy_library_state,
                );
            }
        }
    }
}

fn clear_field(
    field: TextBoxField,
    config: &mut netcode::SessionConfig,
    local_profile: &mut LocalPlayerProfile,
    editor_ship: &mut EditorShip,
    editor_session: &EditorSessionState,
    enemy_editor_state: &mut EnemyEditorState,
    enemy_library_state: &mut EnemyShipLibraryState,
    sector_state: &SectorState,
) {
    set_field_value(
        field,
        config,
        local_profile,
        editor_ship,
        editor_session,
        enemy_editor_state,
        enemy_library_state,
        sector_state,
        String::new(),
    );
}

fn sync_selected_enemy_name(
    ship_name: &str,
    editor_ship: &EditorShip,
    enemy_editor_state: &mut EnemyEditorState,
    enemy_library_state: &mut EnemyShipLibraryState,
) {
    enemy_library_state.library.ensure_seeded();
    let selected_index = enemy_library_state
        .selected_index
        .min(enemy_library_state.library.entries.len().saturating_sub(1));
    let Some(entry) = enemy_library_state
        .library
        .selected_or_first_mut(selected_index)
    else {
        return;
    };
    entry.display_name = ship_name.to_string();
    entry.ship_name = Some(ship_name.to_string());
    entry.ship = editor_ship.ship.clone();
    enemy_editor_state.dirty = true;
}

fn is_host_address_character(character: char) -> bool {
    character.is_ascii_alphanumeric()
        || matches!(character, '.' | ':' | '-' | '[' | ']' | '@' | '>' | ',')
}

fn set_textbox_clipboard(clipboard: &mut TextBoxClipboard, contents: String) {
    clipboard.contents = contents.clone();
    if let Ok(mut system_clipboard) = arboard::Clipboard::new() {
        let _ = system_clipboard.set_text(contents);
    }
}

fn get_textbox_clipboard(clipboard: &TextBoxClipboard) -> Option<String> {
    if let Ok(mut system_clipboard) = arboard::Clipboard::new()
        && let Ok(contents) = system_clipboard.get_text()
        && !contents.is_empty()
    {
        return Some(contents);
    }
    (!clipboard.contents.is_empty()).then(|| clipboard.contents.clone())
}

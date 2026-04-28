mod helpers;
mod input;
mod render;
mod ui;

pub(crate) use input::{
    computer_program_button_system,
    enemy_library_button_system,
    enemy_library_keyboard_shortcuts,
    leave_editor_button_system,
    leave_editor_keyboard_shortcut,
    load_editor_ship_shortcut,
    pan_and_zoom_editor_view,
    persist_editor_ship,
    place_or_remove_tile,
    rotate_selected_tool,
    save_editor_ship_shortcut,
    toolbox_button_system,
};
pub(crate) use render::{
    draw_grid_overlay,
    spawn_preview_tile,
    sync_preview_tile,
    sync_ship_tile_entities,
    sync_toolbox_visuals,
};
pub(crate) use ui::{
    cleanup_editor_entities,
    initialize_editor_ship,
    spawn_editor_ui,
    sync_computer_program_entries,
    update_editor_status_text,
};

pub(super) const UNAFFORDABLE_BUTTON: bevy::prelude::Color =
    bevy::prelude::Color::srgb(0.22, 0.22, 0.26);
pub(super) const SELECTED_UNAFFORDABLE_BUTTON: bevy::prelude::Color =
    bevy::prelude::Color::srgb(0.44, 0.24, 0.16);

mod helpers;
mod input;
mod render;
mod ui;

pub(crate) use input::{
    arch_editor_button_system,
    computer_program_button_system,
    edit_program_text_editor,
    editor_station_panel_button_system,
    enemy_library_button_system,
    enemy_library_keyboard_shortcuts,
    focus_program_text_editor_on_click,
    leave_editor_button_system,
    leave_editor_keyboard_shortcut,
    load_editor_ship_shortcut,
    mission_report_button_system,
    pan_and_zoom_editor_view,
    persist_editor_ship,
    place_or_remove_tile,
    program_editor_action_button_system,
    repair_selected_component_shortcut,
    rotate_selected_tool,
    save_editor_ship_shortcut,
    sync_program_text_editor_state,
    toggle_editor_module_overlay_shortcuts,
    toolbox_button_system,
};
pub(crate) use render::{
    draw_grid_overlay,
    spawn_preview_tile,
    sync_preview_tile,
    sync_ship_tile_entities,
    sync_toolbox_scroll,
    sync_toolbox_visuals,
};
pub(crate) use ui::{
    cleanup_editor_entities,
    editor_ui_missing,
    editor_ui_present,
    initialize_editor_ship,
    spawn_editor_ui,
    update_editor_module_overlay,
    update_editor_status_text,
};

pub(crate) const UNAFFORDABLE_BUTTON: bevy::prelude::Color =
    bevy::prelude::Color::srgb(0.22, 0.22, 0.26);
pub(crate) const SELECTED_UNAFFORDABLE_BUTTON: bevy::prelude::Color =
    bevy::prelude::Color::srgb(0.44, 0.24, 0.16);

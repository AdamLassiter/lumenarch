use bevy::prelude::*;

use crate::{
    editor::helpers::normalize_editor_ship_layers,
    netcode,
    ship::{
        ModuleKind,
        ModuleVariant,
        ShipDefinition,
        enemy::{EnemyShipLibrary, load_validated_default_enemy_library},
        storage::load_default_ship,
    },
    state::{
        ArchEditorState,
        EditorLayer,
        EditorMode,
        EditorPointerState,
        EditorSelectionState,
        EditorSessionState,
        EditorShip,
        EditorToolMode,
        EditorToolState,
        EnemyEditorState,
        EnemyShipLibraryState,
        ProgramTextEditorState,
    },
};

/// Loads and normalizes the active ship into editor state so refit sessions always start from a clean baseline.
pub(crate) fn initialize_editor_ship(
    status: Res<netcode::SessionStatus>,
    rollback_state: Res<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut pointer_state: ResMut<EditorPointerState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
    mut program_editor_state: ResMut<ProgramTextEditorState>,
) {
    match editor_session.mode {
        EditorMode::Player => {
            if matches!(status.phase, netcode::SessionPhase::Connected) {
                editor_ship.ship = rollback_state.editor_ship.clone();
            } else {
                match load_default_ship() {
                    Ok(Some(saved_ship)) => {
                        editor_ship.ship = saved_ship;
                    }
                    Ok(None) => {
                        if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
                            editor_ship.ship = snapshot.clone();
                        } else if editor_ship.ship.name.is_empty()
                            && editor_ship.ship.modules.is_empty()
                        {
                            editor_ship.ship = ShipDefinition::empty("Untitled Knot");
                        }
                    }
                    Err(error) => {
                        eprintln!("editor: failed to load saved ship: {error}");
                        if let Some(snapshot) = status.active_ship_snapshot.as_ref() {
                            editor_ship.ship = snapshot.clone();
                        } else if editor_ship.ship.name.is_empty()
                            && editor_ship.ship.modules.is_empty()
                        {
                            editor_ship.ship = ShipDefinition::empty("Untitled Knot");
                        }
                    }
                }
            }
        }
        EditorMode::Enemy => {
            match load_validated_default_enemy_library() {
                Ok(Some(validated)) => {
                    bevy::log::info!(
                        "Loaded enemy ship library with {} entries for debug enemy editor",
                        validated.library.entries.len()
                    );
                    enemy_library_state.library = validated.library;
                    enemy_library_state.entry_statuses = validated.statuses;
                }
                Ok(None) => {
                    enemy_library_state.library = EnemyShipLibrary::seeded();
                    enemy_library_state.entry_statuses.clear();
                }
                Err(error) => {
                    eprintln!("editor: failed to load enemy ship library: {error}");
                    enemy_library_state.library = EnemyShipLibrary::seeded();
                    enemy_library_state.entry_statuses.clear();
                }
            }
            enemy_library_state.library.ensure_seeded();
            enemy_library_state.selected_index = enemy_library_state
                .selected_index
                .min(enemy_library_state.library.entries.len().saturating_sub(1));
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first(enemy_library_state.selected_index)
            {
                editor_ship.ship = entry.ship.clone();
            }
            enemy_editor_state.dirty = false;
        }
    }

    normalize_editor_ship_layers(&mut editor_ship.ship);

    tool_state.tool_mode = EditorToolMode::Build;
    tool_state.active_layer = EditorLayer::Logistics;
    tool_state.selected_foundation_kind = crate::ship::ShipFoundationKind::Floor;
    tool_state.selected_kind = ModuleKind::Core;
    tool_state.selected_variant = ModuleVariant::default_for_kind(ModuleKind::Core);
    tool_state.selected_rotation = 0;
    tool_state.selected_channel = 0;
    tool_state.ignore_component_limits = false;
    selection_state.selected_module_ids.clear();
    selection_state.selected_foundation_ids.clear();
    selection_state.clipboard.clear();
    selection_state.foundation_clipboard.clear();
    selection_state.marquee_origin = None;
    selection_state.marquee_current = None;
    pointer_state.last_build_cell = None;
    arch_editor_state.selected_module_id = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Computer)
        .map(|module| module.id);
    arch_editor_state.selected_line = 0;
    arch_editor_state.panel_open = false;
    program_editor_state.module_id = None;
    program_editor_state.draft_text.clear();
    program_editor_state.cursor_index = 0;
    program_editor_state.select_all = false;
    program_editor_state.focused = false;
    program_editor_state.dirty = false;
    program_editor_state.diagnostics.clear();
    program_editor_state.status_line.clear();
}

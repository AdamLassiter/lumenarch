use bevy::prelude::*;

use crate::{
    netcode,
    ship::{enemy::load_default_enemy_library, storage::load_default_ship, ModuleKind, ShipDefinition},
    state::{
        ArchEditorState, EditorMode, EditorSessionState, EditorShip, EditorToolState,
        EnemyShipLibraryState,
    },
};

pub(crate) fn initialize_editor_ship(
    status: Res<netcode::SessionStatus>,
    rollback_state: Res<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
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
            match load_default_enemy_library() {
                Ok(Some(library)) => {
                    enemy_library_state.library = library;
                }
                Ok(None) => {
                    enemy_library_state.library = crate::ship::enemy::EnemyShipLibrary::seeded();
                }
                Err(error) => {
                    eprintln!("editor: failed to load enemy ship library: {error}");
                    enemy_library_state.library = crate::ship::enemy::EnemyShipLibrary::seeded();
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
        }
    }

    tool_state.selected_kind = ModuleKind::Hull;
    tool_state.selected_variant = crate::ship::ModuleVariant::default_for_kind(ModuleKind::Hull);
    tool_state.selected_rotation = 0;
    arch_editor_state.selected_module_id = editor_ship
        .ship
        .modules
        .iter()
        .find(|module| module.kind == ModuleKind::Computer)
        .map(|module| module.id);
    arch_editor_state.selected_line = 0;
}

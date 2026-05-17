use bevy::{input::mouse::MouseButton, prelude::*, window::PrimaryWindow};

use super::{
    auto_hull::apply_auto_hull_to_ship,
    enemy::{save_enemy_library_if_valid, sync_selected_enemy_entry},
    selection::{
        delete_selected_group,
        foundation_snapshot,
        module_snapshot,
        move_selected_foundation_group,
        move_selected_group,
        paste_clipboard_group,
        paste_foundation_clipboard_group,
        selected_or_all_modules,
        ship_anchor,
    },
};
use crate::{
    HOVERED_BUTTON,
    NORMAL_BUTTON,
    PRESSED_BUTTON,
    helpers::editor::{
        cursor_grid_position,
        foundation_family_label,
        foundation_supports_module,
        is_cursor_over_editor_ui,
        is_hull_foundation_kind,
        module_belongs_to_components_layer,
        module_belongs_to_hull_layer,
        module_family_label,
        variant_tooltip_text,
    },
    netcode,
    ship::{
        ModuleKind,
        ShipDefinition,
        ShipFoundationTile,
        ShipModule,
        arch::{ArchProgram, ArchProgramTemplate},
        lumen::{LumenProgram, LumenProgramTemplate},
    },
    state::{
        ArchEditorState,
        EditorAutoHullButton,
        EditorCopySelectionButton,
        EditorDeleteSelectionButton,
        EditorLayer,
        EditorLayerButton,
        EditorMissionReportButton,
        EditorMode,
        EditorPasteSelectionButton,
        EditorPlacementBlocker,
        EditorPointerState,
        EditorSelectionState,
        EditorSessionState,
        EditorShip,
        EditorToolMode,
        EditorToolModeButton,
        EditorToolState,
        EditorUiState,
        EnemyEditorState,
        EnemyShipLibraryState,
        FocusedTextBox,
        FrontendMode,
        GameplayStationPanelButton,
        LeaveEditorButton,
        ProgrammingLanguageMode,
        Progression,
        StationPanelButtonAction,
        ToolboxFoundationButton,
        ToolboxVariantButton,
    },
};

/// Handles toolbox clicks so the editor's active tool, layer, and selected part follow UI intent.
pub(crate) fn toolbox_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ToolboxVariantButton>,
            Option<&ToolboxFoundationButton>,
            Option<&EditorToolModeButton>,
            Option<&EditorLayerButton>,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    editor_session: Res<EditorSessionState>,
    progression: Res<Progression>,
    mut tool_state: ResMut<EditorToolState>,
    mut editor_ui_state: ResMut<EditorUiState>,
) {
    for (
        interaction,
        variant_button,
        foundation_button,
        mode_button,
        layer_button,
        mut background,
    ) in &mut interaction_query
    {
        match *interaction {
            Interaction::Pressed => {
                if let Some(button) = mode_button {
                    tool_state.tool_mode = button.mode;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = layer_button {
                    tool_state.active_layer = button.layer;
                    tool_state.tool_mode = EditorToolMode::Build;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = foundation_button {
                    tool_state.active_layer = if is_hull_foundation_kind(button.kind) {
                        EditorLayer::Hull
                    } else {
                        EditorLayer::Logistics
                    };
                    tool_state.tool_mode = EditorToolMode::Build;
                    tool_state.selected_foundation_kind = button.kind;
                    *background = BackgroundColor(PRESSED_BUTTON);
                } else if let Some(button) = variant_button {
                    let available = tool_state.ignore_component_limits
                        || editor_session.mode == EditorMode::Enemy
                        || progression.ready_count(button.kind, button.variant) > 0
                        || progression.damaged_count(button.kind, button.variant) > 0;
                    if !available {
                        *background = BackgroundColor(super::super::SELECTED_UNAFFORDABLE_BUTTON);
                        continue;
                    }
                    tool_state.tool_mode = EditorToolMode::Build;
                    tool_state.active_layer = if module_belongs_to_hull_layer(button.kind) {
                        EditorLayer::Hull
                    } else {
                        EditorLayer::Components
                    };
                    tool_state.selected_kind = button.kind;
                    tool_state.selected_variant = button.variant;
                    *background = BackgroundColor(PRESSED_BUTTON);
                }
            }
            Interaction::Hovered => {
                if let Some(button) = variant_button {
                    editor_ui_state.toolbox_tooltip.title = format!(
                        "{} / {}",
                        module_family_label(button.kind),
                        button.kind.as_str()
                    );
                    editor_ui_state.toolbox_tooltip.detail = variant_tooltip_text(
                        editor_session.mode,
                        &progression,
                        button.kind,
                        button.variant,
                    );
                    let available = tool_state.ignore_component_limits
                        || editor_session.mode == EditorMode::Enemy
                        || progression.ready_count(button.kind, button.variant) > 0
                        || progression.damaged_count(button.kind, button.variant) > 0;
                    *background = BackgroundColor(if available {
                        HOVERED_BUTTON
                    } else {
                        super::super::UNAFFORDABLE_BUTTON
                    });
                } else if let Some(button) = foundation_button {
                    editor_ui_state.toolbox_tooltip.title = format!(
                        "{} / {}",
                        foundation_family_label(button.kind),
                        button.kind.as_str()
                    );
                    editor_ui_state.toolbox_tooltip.detail =
                        "Logistics tile. It can share a cell with hull and component placements."
                            .to_string();
                    *background = BackgroundColor(HOVERED_BUTTON);
                } else {
                    *background = BackgroundColor(HOVERED_BUTTON);
                }
            }
            Interaction::None => {}
        }
    }
}

/// Leaves the current editor flow from the UI so player refit or enemy debug editing can close cleanly.
pub(crate) fn leave_editor_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<LeaveEditorButton>),
    >,
    editor_session: Res<EditorSessionState>,
    editor_ship: Res<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if editor_session.mode == EditorMode::Player && !netcode::is_host_authority(&status) {
        return;
    }
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.42, 0.30, 0.20));
                match editor_session.mode {
                    EditorMode::Player => {
                        pending_meta.0 = Some(netcode::PendingMetaCommand {
                            op: netcode::RollbackMetaOp::LeaveEditor,
                            ..Default::default()
                        });
                    }
                    EditorMode::Enemy => {
                        let saved =
                            sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
                                && save_enemy_library_if_valid(&enemy_library_state);
                        enemy_editor_state.dirty = !saved;
                        next_mode.set(FrontendMode::Lobby);
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.56, 0.40, 0.26));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.46, 0.34, 0.22));
            }
        }
    }
}

/// Provides a keyboard exit path from the editor so navigation matches the UI button behavior.
pub(crate) fn leave_editor_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    editor_session: Res<EditorSessionState>,
    editor_ship: Res<EditorShip>,
    status: Res<netcode::SessionStatus>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut pending_meta: ResMut<netcode::PendingLocalMetaCommand>,
    mut next_mode: ResMut<NextState<FrontendMode>>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if editor_session.mode == EditorMode::Player && !netcode::is_host_authority(&status) {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        match editor_session.mode {
            EditorMode::Player => {
                pending_meta.0 = Some(netcode::PendingMetaCommand {
                    op: netcode::RollbackMetaOp::LeaveEditor,
                    ..Default::default()
                });
            }
            EditorMode::Enemy => {
                let saved = sync_selected_enemy_entry(&editor_ship, &mut enemy_library_state)
                    && save_enemy_library_if_valid(&enemy_library_state);
                enemy_editor_state.dirty = !saved;
                next_mode.set(FrontendMode::Lobby);
            }
        }
    }
}

/// Expands or collapses the mission report panel so recent sortie feedback is available on demand.
pub(crate) fn mission_report_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (
            Changed<Interaction>,
            With<Button>,
            With<EditorMissionReportButton>,
        ),
    >,
    mut editor_ui_state: ResMut<EditorUiState>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                editor_ui_state.mission_report_expanded = !editor_ui_state.mission_report_expanded;
                *background = BackgroundColor(PRESSED_BUTTON);
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

/// Rotates tools and cycles editor build options so common authoring actions stay on the keyboard.
pub(crate) fn rotate_selected_tool(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    mut tool_state: ResMut<EditorToolState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyL) {
        tool_state.active_layer = match tool_state.active_layer {
            EditorLayer::Logistics => EditorLayer::Hull,
            EditorLayer::Hull => EditorLayer::Components,
            EditorLayer::Components => EditorLayer::Logistics,
        };
    }

    if keys.just_pressed(KeyCode::F10) {
        tool_state.ignore_component_limits = !tool_state.ignore_component_limits;
    }

    if tool_state.tool_mode == EditorToolMode::Build && keys.just_pressed(KeyCode::KeyR) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 1) % 4;
    }

    if tool_state.tool_mode == EditorToolMode::Build && keys.just_pressed(KeyCode::KeyZ) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, -1);
    }

    if tool_state.tool_mode == EditorToolMode::Build && keys.just_pressed(KeyCode::KeyX) {
        tool_state.selected_variant = tool_state
            .selected_variant
            .cycle_for_kind(tool_state.selected_kind, 1);
    }

    if keys.just_pressed(KeyCode::KeyC) {
        tool_state.selected_channel = tool_state.selected_channel.wrapping_add(9) % 10;
    }

    if keys.just_pressed(KeyCode::KeyV) {
        tool_state.selected_channel = (tool_state.selected_channel + 1) % 10;
    }
}

/// Applies paint, erase, and marquee interactions to the editor grid so ship authoring stays direct.
pub(crate) fn place_or_remove_tile(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ui_blocker_query: Query<
        (
            &ComputedNode,
            &bevy::ui::UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        With<EditorPlacementBlocker>,
    >,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut pointer_state: ResMut<EditorPointerState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();

    if is_cursor_over_editor_ui(window, &ui_blocker_query) {
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };

    match tool_state.tool_mode {
        EditorToolMode::Build => {
            selection_state.marquee_origin = None;
            selection_state.marquee_current = None;

            if buttons.just_released(MouseButton::Left) || buttons.just_released(MouseButton::Right)
            {
                pointer_state.last_build_cell = None;
            }

            for (mouse_button, erase) in [(MouseButton::Left, false), (MouseButton::Right, true)] {
                if (buttons.just_pressed(mouse_button) || buttons.pressed(mouse_button))
                    && pointer_state.last_build_cell != Some((grid_x, grid_y, mouse_button))
                    && apply_build_action(
                        &mut editor_ship.ship,
                        &mut progression,
                        editor_session.mode,
                        tool_state.as_ref(),
                        grid_x,
                        grid_y,
                        erase,
                    )
                {
                    pointer_state.last_build_cell = Some((grid_x, grid_y, mouse_button));
                    sync_editor_resources(
                        &editor_ship.ship,
                        &progression,
                        editor_session.mode,
                        &mut rollback_state,
                        &mut enemy_editor_state,
                    );
                }
            }
        }
        EditorToolMode::Select => {
            pointer_state.last_build_cell = None;
            if buttons.just_pressed(MouseButton::Left) {
                selection_state.marquee_origin = Some((grid_x, grid_y));
                selection_state.marquee_current = Some((grid_x, grid_y));
            } else if buttons.pressed(MouseButton::Left) {
                selection_state.marquee_current = Some((grid_x, grid_y));
            } else if buttons.just_released(MouseButton::Left) {
                if let Some(origin) = selection_state.marquee_origin {
                    let current = selection_state.marquee_current.unwrap_or(origin);
                    if tool_state.active_layer != EditorLayer::Components {
                        selection_state.selected_foundation_ids = select_foundations_in_rect(
                            &editor_ship.ship,
                            origin,
                            current,
                            tool_state.active_layer,
                        );
                        selection_state.selected_module_ids.clear();
                    } else {
                        selection_state.selected_module_ids =
                            select_modules_in_rect(&editor_ship.ship, origin, current);
                        selection_state.selected_foundation_ids.clear();
                    }
                }
                selection_state.marquee_origin = None;
                selection_state.marquee_current = None;
            }
        }
    }
}

fn select_foundations_in_rect(
    ship: &ShipDefinition,
    origin: (i32, i32),
    current: (i32, i32),
    layer: EditorLayer,
) -> Vec<u64> {
    let min_x = origin.0.min(current.0);
    let max_x = origin.0.max(current.0);
    let min_y = origin.1.min(current.1);
    let max_y = origin.1.max(current.1);
    let tiles = match layer {
        EditorLayer::Logistics => &ship.foundation_tiles,
        EditorLayer::Hull => &ship.hull_tiles,
        EditorLayer::Components => return Vec::new(),
    };
    tiles
        .iter()
        .filter(|tile| {
            tile.grid_x >= min_x
                && tile.grid_x <= max_x
                && tile.grid_y >= min_y
                && tile.grid_y <= max_y
        })
        .map(|tile| tile.id)
        .collect()
}

fn apply_build_action(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    tool_state: &EditorToolState,
    grid_x: i32,
    grid_y: i32,
    erase: bool,
) -> bool {
    if erase {
        return match tool_state.active_layer {
            EditorLayer::Logistics => {
                apply_foundation_build_action(ship, tool_state, grid_x, grid_y, true, false)
            }
            EditorLayer::Hull => {
                if let Some(existing) = ship.module_at(grid_x, grid_y).cloned()
                    && module_belongs_to_hull_layer(existing.kind)
                {
                    if mode == EditorMode::Player && !tool_state.ignore_component_limits {
                        progression.add_ready_component(existing.kind, existing.variant, 1);
                    }
                    ship.remove_module_at(grid_x, grid_y);
                    true
                } else {
                    apply_foundation_build_action(ship, tool_state, grid_x, grid_y, true, true)
                }
            }
            EditorLayer::Components => {
                apply_module_build_action(ship, progression, mode, tool_state, grid_x, grid_y, true)
            }
        };
    }

    match tool_state.active_layer {
        EditorLayer::Logistics => {
            apply_foundation_build_action(ship, tool_state, grid_x, grid_y, false, false)
        }
        EditorLayer::Hull => {
            if module_belongs_to_hull_layer(tool_state.selected_kind) {
                apply_module_build_action(
                    ship,
                    progression,
                    mode,
                    tool_state,
                    grid_x,
                    grid_y,
                    false,
                )
            } else {
                apply_foundation_build_action(ship, tool_state, grid_x, grid_y, false, true)
            }
        }
        EditorLayer::Components => {
            if !module_belongs_to_components_layer(tool_state.selected_kind) {
                return false;
            }
            apply_module_build_action(ship, progression, mode, tool_state, grid_x, grid_y, false)
        }
    }
}

fn apply_module_build_action(
    ship: &mut ShipDefinition,
    progression: &mut Progression,
    mode: EditorMode,
    tool_state: &EditorToolState,
    grid_x: i32,
    grid_y: i32,
    erase: bool,
) -> bool {
    if erase {
        if let Some(existing) = ship.module_at(grid_x, grid_y).cloned() {
            if mode == EditorMode::Player && !tool_state.ignore_component_limits {
                progression.add_ready_component(existing.kind, existing.variant, 1);
            }
            ship.remove_module_at(grid_x, grid_y);
            return true;
        }
        return false;
    }

    let selected_variant = tool_state
        .selected_variant
        .normalize_for_kind(tool_state.selected_kind);
    if !foundation_supports_module(
        ship.logistics_at(grid_x, grid_y).map(|tile| tile.kind),
        ship.hull_at(grid_x, grid_y).map(|tile| tile.kind),
        tool_state.selected_kind,
    ) {
        return false;
    }
    if let Some(existing) = ship.module_at_mut(grid_x, grid_y) {
        if existing.kind == tool_state.selected_kind && existing.variant == selected_variant {
            existing.rotation_quadrants = tool_state.selected_rotation % 4;
            existing.channel = tool_state.selected_channel;
            return true;
        }

        if mode == EditorMode::Player
            && !tool_state.ignore_component_limits
            && !progression.try_consume_ready_component(tool_state.selected_kind, selected_variant)
        {
            return false;
        }
        if mode == EditorMode::Player && !tool_state.ignore_component_limits {
            progression.add_ready_component(existing.kind, existing.variant, 1);
        }
        existing.kind = tool_state.selected_kind;
        existing.variant = selected_variant;
        existing.rotation_quadrants = tool_state.selected_rotation % 4;
        existing.channel = tool_state.selected_channel;
        return true;
    }

    if mode == EditorMode::Player
        && !tool_state.ignore_component_limits
        && !progression.try_consume_ready_component(tool_state.selected_kind, selected_variant)
    {
        return false;
    }
    let next_id = ship.next_module_id();
    let mut module = ShipModule::new(
        next_id,
        tool_state.selected_kind,
        grid_x,
        grid_y,
        tool_state.selected_rotation % 4,
    );
    module.variant = selected_variant;
    module.channel = tool_state.selected_channel;
    ship.replace_module(module);
    true
}

fn apply_foundation_build_action(
    ship: &mut ShipDefinition,
    tool_state: &EditorToolState,
    grid_x: i32,
    grid_y: i32,
    erase: bool,
    hull_layer: bool,
) -> bool {
    let replacement_kind = (!erase).then_some(tool_state.selected_foundation_kind);
    if let Some(existing_module) = ship.module_at(grid_x, grid_y)
        && !foundation_supports_module(
            if hull_layer {
                ship.logistics_at(grid_x, grid_y).map(|tile| tile.kind)
            } else {
                replacement_kind
            },
            if hull_layer {
                replacement_kind
            } else {
                ship.hull_at(grid_x, grid_y).map(|tile| tile.kind)
            },
            existing_module.kind,
        )
    {
        return false;
    }

    if erase {
        let removed = if hull_layer {
            ship.hull_at(grid_x, grid_y)
                .is_some()
                .then(|| ship.remove_hull_at(grid_x, grid_y))
        } else {
            ship.logistics_at(grid_x, grid_y)
                .is_some()
                .then(|| ship.remove_logistics_at(grid_x, grid_y))
        };
        if removed.is_some() {
            return true;
        }
        return false;
    }

    if let Some(existing) = if hull_layer {
        ship.hull_at_mut(grid_x, grid_y)
    } else {
        ship.logistics_at_mut(grid_x, grid_y)
    } {
        existing.kind = tool_state.selected_foundation_kind;
        existing.rotation_quadrants = tool_state.selected_rotation % 4;
        return true;
    }

    let tile = ShipFoundationTile::new(
        ship.next_foundation_id(),
        tool_state.selected_foundation_kind,
        grid_x,
        grid_y,
        tool_state.selected_rotation % 4,
    );
    if hull_layer {
        ship.replace_hull_tile(tile);
    } else {
        ship.replace_logistics_tile(tile);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{apply_auto_hull_to_ship, apply_build_action};
    use crate::{
        ship::{
            ModuleKind,
            ModuleVariant,
            ShipDefinition,
            ShipFoundationKind,
            ShipFoundationTile,
            ShipModule,
        },
        state::{EditorLayer, EditorMode, EditorToolState, Progression},
    };

    fn tool_state_for_components(kind: ModuleKind) -> EditorToolState {
        EditorToolState {
            active_layer: EditorLayer::Components,
            selected_kind: kind,
            selected_variant: ModuleVariant::default_for_kind(kind),
            ..Default::default()
        }
    }

    fn tool_state_for_hull_fixture(kind: ModuleKind) -> EditorToolState {
        EditorToolState {
            active_layer: EditorLayer::Hull,
            selected_kind: kind,
            selected_variant: ModuleVariant::default_for_kind(kind),
            ..Default::default()
        }
    }

    #[test]
    fn components_require_logistics_support() {
        let mut ship = ShipDefinition::empty("Support Test");
        let mut progression = Progression::default();
        let placed = apply_build_action(
            &mut ship,
            &mut progression,
            EditorMode::Enemy,
            &tool_state_for_components(ModuleKind::Core),
            0,
            0,
            false,
        );
        assert!(!placed);

        ship.replace_foundation_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Floor,
            0,
            0,
            0,
        ));
        let placed = apply_build_action(
            &mut ship,
            &mut progression,
            EditorMode::Enemy,
            &tool_state_for_components(ModuleKind::Core),
            0,
            0,
            false,
        );
        assert!(placed);
    }

    #[test]
    fn exterior_modules_require_hull_foundation() {
        let mut ship = ShipDefinition::empty("Exterior Test");
        ship.replace_logistics_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Floor,
            0,
            0,
            0,
        ));
        let mut progression = Progression::default();
        let placed = apply_build_action(
            &mut ship,
            &mut progression,
            EditorMode::Enemy,
            &tool_state_for_hull_fixture(ModuleKind::Turret),
            0,
            0,
            false,
        );
        assert!(!placed);

        ship.replace_hull_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Hull,
            0,
            0,
            0,
        ));
        let placed = apply_build_action(
            &mut ship,
            &mut progression,
            EditorMode::Enemy,
            &tool_state_for_hull_fixture(ModuleKind::Turret),
            0,
            0,
            false,
        );
        assert!(placed);
    }

    #[test]
    fn hull_layer_airlock_selection_places_module_on_hull() {
        let mut ship = ShipDefinition::empty("Airlock Hull Test");
        ship.replace_hull_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Hull,
            0,
            0,
            0,
        ));
        let mut progression = Progression::default();
        let placed = apply_build_action(
            &mut ship,
            &mut progression,
            EditorMode::Enemy,
            &EditorToolState {
                active_layer: EditorLayer::Hull,
                selected_kind: ModuleKind::Airlock,
                selected_variant: ModuleVariant::Standard,
                ..Default::default()
            },
            0,
            0,
            false,
        );
        assert!(placed);
        assert_eq!(
            ship.module_at(0, 0).map(|module| module.kind),
            Some(ModuleKind::Airlock)
        );
    }

    #[test]
    fn auto_hull_writes_foundation_hull_tiles() {
        let mut ship = ShipDefinition::empty("Auto Hull Foundations");
        ship.replace_logistics_tile(ShipFoundationTile::new(
            1,
            ShipFoundationKind::Floor,
            0,
            0,
            0,
        ));
        ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));

        let changed = apply_auto_hull_to_ship(&mut ship);

        assert!(changed);
        assert!(
            ship.hull_tiles
                .iter()
                .any(|tile| matches!(tile.kind, ShipFoundationKind::HullOuterCorner))
        );
        assert!(ship.modules.iter().all(|module| !matches!(
            module.kind,
            ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
        )));
    }
}

fn sync_editor_resources(
    ship: &ShipDefinition,
    progression: &Progression,
    mode: EditorMode,
    rollback_state: &mut netcode::RollbackGameState,
    enemy_editor_state: &mut EnemyEditorState,
) {
    if mode == EditorMode::Player {
        rollback_state.editor_ship = ship.clone();
        rollback_state.progression = progression.clone();
    } else {
        enemy_editor_state.dirty = true;
    }
}

fn select_modules_in_rect(
    ship: &ShipDefinition,
    origin: (i32, i32),
    current: (i32, i32),
) -> Vec<u64> {
    let min_x = origin.0.min(current.0);
    let max_x = origin.0.max(current.0);
    let min_y = origin.1.min(current.1);
    let max_y = origin.1.max(current.1);
    ship.modules
        .iter()
        .filter(|module| {
            module.grid_x >= min_x
                && module.grid_x <= max_x
                && module.grid_y >= min_y
                && module.grid_y <= max_y
        })
        .map(|module| module.id)
        .collect()
}

/// Opens and closes module inspection panels from the grid so part configuration stays contextual.
pub(crate) fn toggle_editor_module_overlay_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_ship: Res<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut arch_editor_state: ResMut<ArchEditorState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if keys.just_pressed(KeyCode::KeyQ) {
        arch_editor_state.panel_open = false;
        return;
    }

    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }

    let window = window.into_inner();
    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };
    let Some(module) = editor_ship.ship.module_at(grid_x, grid_y) else {
        return;
    };
    arch_editor_state.selected_module_id = Some(module.id);
    arch_editor_state.panel_open = true;
    selection_state.selected_module_ids = vec![module.id];
    tool_state.tool_mode = EditorToolMode::Select;
    tool_state.selected_kind = module.kind;
    tool_state.selected_variant = module.variant;
    tool_state.selected_channel = module.effective_channel();
}

/// Routes station-panel clicks in the editor to part-specific configuration state changes.
pub(crate) fn editor_station_panel_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &GameplayStationPanelButton,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_ship: ResMut<EditorShip>,
    arch_editor_state: Res<ArchEditorState>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !arch_editor_state.panel_open || !netcode::is_host_authority(&status) {
        return;
    }
    let Some(module_id) = arch_editor_state.selected_module_id else {
        return;
    };
    let mut changed = false;
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                let Some(module) = editor_ship
                    .ship
                    .modules
                    .iter_mut()
                    .find(|module| module.id == module_id)
                else {
                    continue;
                };
                match button.action {
                    StationPanelButtonAction::HelmThrottle { .. }
                    | StationPanelButtonAction::HelmTurn { .. } => {}
                    StationPanelButtonAction::TurretAdjustAim { .. } => {}
                    StationPanelButtonAction::TurretFireToggle => {
                        module.defaults.turret_fire_intent = !module.defaults.turret_fire_intent;
                    }
                    StationPanelButtonAction::ReactorAdjustRate { delta } => {
                        let current = module.defaults.reaction_rate_milli as i32;
                        module.defaults.reaction_rate_milli =
                            (current + (delta * 1000.0) as i32).clamp(0, 1000) as u16;
                    }
                    StationPanelButtonAction::ReactorAdjustTurbine { delta } => {
                        let current = module.defaults.turbine_load_milli as i32;
                        module.defaults.turbine_load_milli =
                            (current + (delta * 1000.0) as i32).clamp(0, 1000) as u16;
                    }
                    StationPanelButtonAction::LogisticsToggleStorageIntake => {
                        module.defaults.storage_allow_intake =
                            !module.defaults.storage_allow_intake;
                    }
                    StationPanelButtonAction::LogisticsToggleAirlock => {
                        module.defaults.airlock_open = !module.defaults.airlock_open;
                    }
                    StationPanelButtonAction::InfrastructureToggleBlocker => {
                        // Runtime-only until blocker defaults are persisted in ship prefab data.
                    }
                    StationPanelButtonAction::LogisticsToggleManipulator => {
                        module.defaults.manipulator_transfer_enabled =
                            !module.defaults.manipulator_transfer_enabled;
                    }
                    StationPanelButtonAction::LogisticsCycleManipulatorTarget { .. } => {
                        module.defaults.manipulator_manual_mode =
                            !module.defaults.manipulator_manual_mode;
                    }
                    StationPanelButtonAction::LogisticsCycleResource => {
                        module.defaults.manipulator_resource_kind =
                            module.defaults.manipulator_resource_kind.next();
                    }
                    StationPanelButtonAction::LogisticsToggleProcessor => {
                        if module.kind == ModuleKind::Processor {
                            module.defaults.processor_enabled = !module.defaults.processor_enabled;
                        } else {
                            module.defaults.processor_recipe =
                                module.defaults.processor_recipe.next();
                        }
                    }
                    StationPanelButtonAction::ComputerToggleEnabled => {
                        module.defaults.computer_enabled = !module.defaults.computer_enabled;
                    }
                    StationPanelButtonAction::ComputerCycleTemplate => {
                        match arch_editor_state.selected_language {
                            ProgrammingLanguageMode::Arch => {
                                let program = module.arch_program.get_or_insert_with(|| {
                                    ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                                });
                                *program = ArchProgram::from_template(program.template.next());
                            }
                            ProgrammingLanguageMode::Lumen => {
                                let program = module.lumen_program.get_or_insert_with(|| {
                                    LumenProgram::from_template(
                                        LumenProgramTemplate::BalancedSupervision,
                                    )
                                });
                                *program = LumenProgram::from_template(program.template.next());
                            }
                        }
                    }
                }
                changed = true;
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }

    if changed {
        if editor_session.mode == EditorMode::Player {
            rollback_state.editor_ship = editor_ship.ship.clone();
        } else {
            enemy_editor_state.dirty = true;
        }
    }
}

/// Repairs the selected player-owned component in refit mode so damaged inventory can be reused.
pub(crate) fn repair_selected_component_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
    mut progression: ResMut<Progression>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if editor_session.mode != EditorMode::Player || !keys.just_pressed(KeyCode::KeyT) {
        return;
    }
    let variant = tool_state
        .selected_variant
        .normalize_for_kind(tool_state.selected_kind);
    if progression.try_repair_component(tool_state.selected_kind, variant) {
        rollback_state.progression = progression.clone();
    }
}

/// Handles selection panel buttons for auto-hull, copy, paste, and delete group editing actions.
pub(crate) fn selection_action_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&EditorAutoHullButton>,
            Option<&EditorCopySelectionButton>,
            Option<&EditorPasteSelectionButton>,
            Option<&EditorDeleteSelectionButton>,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    mut selection_state: ResMut<EditorSelectionState>,
    tool_state: Res<EditorToolState>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if !netcode::is_host_authority(&status) {
        return;
    }
    let window = window.into_inner();
    let camera_query = *camera_query;

    for (interaction, auto_hull, copy, paste, delete, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                let mut changed = false;
                if auto_hull.is_some() {
                    changed = apply_auto_hull_to_ship(&mut editor_ship.ship);
                } else if copy.is_some() {
                    if !selection_state.selected_foundation_ids.is_empty() {
                        selection_state.foundation_clipboard = editor_ship
                            .ship
                            .foundation_tiles
                            .iter()
                            .chain(editor_ship.ship.hull_tiles.iter())
                            .filter(|tile| {
                                selection_state.selected_foundation_ids.contains(&tile.id)
                            })
                            .cloned()
                            .map(foundation_snapshot)
                            .collect();
                    } else {
                        selection_state.clipboard = selected_or_all_modules(
                            &editor_ship.ship,
                            &selection_state.selected_module_ids,
                        )
                        .into_iter()
                        .map(module_snapshot)
                        .collect();
                    }
                } else if paste.is_some() {
                    let anchor = cursor_grid_position(window, camera_query)
                        .unwrap_or_else(|| ship_anchor(&editor_ship.ship));
                    changed = if !selection_state.foundation_clipboard.is_empty() {
                        paste_foundation_clipboard_group(
                            &mut editor_ship.ship,
                            &mut selection_state,
                            anchor,
                        )
                    } else {
                        paste_clipboard_group(
                            &mut editor_ship.ship,
                            &mut progression,
                            editor_session.mode,
                            tool_state.ignore_component_limits,
                            &mut selection_state,
                            anchor,
                        )
                    };
                } else if delete.is_some() {
                    changed = delete_selected_group(
                        &mut editor_ship.ship,
                        &mut progression,
                        editor_session.mode,
                        tool_state.ignore_component_limits,
                        &mut selection_state,
                    );
                }

                if changed {
                    sync_editor_resources(
                        &editor_ship.ship,
                        &progression,
                        editor_session.mode,
                        &mut rollback_state,
                        &mut enemy_editor_state,
                    );
                }
                *background = BackgroundColor(PRESSED_BUTTON);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(if auto_hull.is_some() {
                    Color::srgb(0.46, 0.36, 0.18)
                } else if copy.is_some() {
                    Color::srgb(0.26, 0.42, 0.62)
                } else if paste.is_some() {
                    Color::srgb(0.22, 0.52, 0.34)
                } else {
                    Color::srgb(0.58, 0.26, 0.18)
                });
            }
        }
    }
}

/// Provides keyboard shortcuts for moving, copying, pasting, and rebuilding editor selections.
pub(crate) fn selection_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    editor_session: Res<EditorSessionState>,
    status: Res<netcode::SessionStatus>,
    tool_state: Res<EditorToolState>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<Progression>,
    mut selection_state: ResMut<EditorSelectionState>,
    mut rollback_state: ResMut<netcode::RollbackGameState>,
    mut enemy_editor_state: ResMut<EnemyEditorState>,
) {
    if focused_textbox.field.is_some() {
        return;
    }
    if !netcode::is_host_authority(&status) || tool_state.tool_mode != EditorToolMode::Select {
        return;
    }

    let ctrl_pressed = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let mut changed = false;

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyC) {
        if !selection_state.selected_foundation_ids.is_empty() {
            selection_state.foundation_clipboard = editor_ship
                .ship
                .foundation_tiles
                .iter()
                .chain(editor_ship.ship.hull_tiles.iter())
                .filter(|tile| selection_state.selected_foundation_ids.contains(&tile.id))
                .cloned()
                .map(foundation_snapshot)
                .collect();
        } else {
            selection_state.clipboard =
                selected_or_all_modules(&editor_ship.ship, &selection_state.selected_module_ids)
                    .into_iter()
                    .map(module_snapshot)
                    .collect();
        }
    }

    if ctrl_pressed && keys.just_pressed(KeyCode::KeyV) {
        let anchor = cursor_grid_position(window.into_inner(), *camera_query)
            .unwrap_or_else(|| ship_anchor(&editor_ship.ship));
        changed |= if !selection_state.foundation_clipboard.is_empty() {
            paste_foundation_clipboard_group(&mut editor_ship.ship, &mut selection_state, anchor)
        } else {
            paste_clipboard_group(
                &mut editor_ship.ship,
                &mut progression,
                editor_session.mode,
                tool_state.ignore_component_limits,
                &mut selection_state,
                anchor,
            )
        };
    }

    if keys.just_pressed(KeyCode::Delete) || keys.just_pressed(KeyCode::Backspace) {
        changed |= delete_selected_group(
            &mut editor_ship.ship,
            &mut progression,
            editor_session.mode,
            tool_state.ignore_component_limits,
            &mut selection_state,
        );
    }

    if keys.just_pressed(KeyCode::KeyH) {
        changed |= apply_auto_hull_to_ship(&mut editor_ship.ship);
    }

    for (key, dx, dy) in [
        (KeyCode::ArrowLeft, -1, 0),
        (KeyCode::ArrowRight, 1, 0),
        (KeyCode::ArrowUp, 0, -1),
        (KeyCode::ArrowDown, 0, 1),
    ] {
        if keys.just_pressed(key) {
            changed |= if !selection_state.selected_foundation_ids.is_empty() {
                move_selected_foundation_group(&mut editor_ship.ship, &selection_state, dx, dy)
            } else {
                move_selected_group(&mut editor_ship.ship, &selection_state, dx, dy)
            };
        }
    }

    if changed {
        sync_editor_resources(
            &editor_ship.ship,
            &progression,
            editor_session.mode,
            &mut rollback_state,
            &mut enemy_editor_state,
        );
    }
}

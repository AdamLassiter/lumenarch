use super::*;

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
    mut station_editor_state: ResMut<station_editor::StationEditorState>,
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

    if editor_session.mode == EditorMode::Station && station_editor_state.selected_tool.is_some() {
        selection_state.marquee_origin = None;
        selection_state.marquee_current = None;
        if tool_state.tool_mode != EditorToolMode::Build {
            return;
        }
        if buttons.just_released(MouseButton::Left) || buttons.just_released(MouseButton::Right) {
            pointer_state.last_build_cell = None;
        }
        for (mouse_button, erase) in [(MouseButton::Left, false), (MouseButton::Right, true)] {
            if (buttons.just_pressed(mouse_button) || buttons.pressed(mouse_button))
                && pointer_state.last_build_cell != Some((grid_x, grid_y, mouse_button))
                && station_editor::apply_station_editor_build_action(
                    &mut station_editor_state,
                    &mut editor_ship,
                    grid_x,
                    grid_y,
                    tool_state.selected_rotation,
                    erase,
                )
            {
                pointer_state.last_build_cell = Some((grid_x, grid_y, mouse_button));
            }
        }
        return;
    }

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

pub(super) fn select_foundations_in_rect(
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

pub(super) fn apply_build_action(
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

pub(super) fn apply_module_build_action(
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

pub(super) fn apply_foundation_build_action(
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

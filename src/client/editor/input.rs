use bevy::{
    input::mouse::{MouseButton, MouseMotion, MouseWheel},
    prelude::*,
    window::PrimaryWindow,
};

use super::{
    super::{
        HOVERED_BUTTON,
        PRESSED_BUTTON,
        state::{
            ClientAppState,
            ComputerProgramButton,
            DemoProgression,
            EditorMode,
            EditorSessionState,
            EditorShip,
            EditorToolState,
            EditorViewState,
            EnemyNewButton,
            EnemyNextButton,
            EnemyPrevButton,
            EnemyShipLibraryState,
            LeaveEditorButton,
            ProgramButtonAction,
            ToolboxButton,
        },
    },
    helpers::{cursor_grid_position, is_cursor_over_editor_ui, module_kind_cost},
};
use crate::ship::{
    ShipModule,
    arch::{ArchProgram, ArchProgramTemplate},
    enemy::{load_default_enemy_library, save_default_enemy_library},
    storage::{load_default_ship, save_default_ship},
};

pub(crate) fn toolbox_button_system(
    mut interaction_query: Query<
        (&Interaction, &ToolboxButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    progression: Res<DemoProgression>,
    mut tool_state: ResMut<EditorToolState>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        let affordable = progression.scrap >= module_kind_cost(button.kind);
        match *interaction {
            Interaction::Pressed => {
                tool_state.selected_kind = button.kind;
                *background = BackgroundColor(if affordable {
                    PRESSED_BUTTON
                } else {
                    super::SELECTED_UNAFFORDABLE_BUTTON
                });
            }
            Interaction::Hovered => {
                *background = BackgroundColor(if affordable {
                    HOVERED_BUTTON
                } else {
                    super::UNAFFORDABLE_BUTTON
                });
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn leave_editor_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<LeaveEditorButton>),
    >,
    editor_session: Res<EditorSessionState>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.42, 0.30, 0.20));
                next_state.set(match editor_session.mode {
                    EditorMode::Player => ClientAppState::Docked,
                    EditorMode::Enemy => ClientAppState::Menu,
                });
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

pub(crate) fn leave_editor_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        next_state.set(match editor_session.mode {
            EditorMode::Player => ClientAppState::Docked,
            EditorMode::Enemy => ClientAppState::Menu,
        });
    }
}

pub(crate) fn computer_program_button_system(
    mut interaction_query: Query<
        (&Interaction, &ComputerProgramButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_ship: ResMut<EditorShip>,
) {
    for (interaction, button, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                let Some(module) = editor_ship
                    .ship
                    .modules
                    .iter_mut()
                    .find(|module| module.id == button.module_id)
                else {
                    continue;
                };
                let program = module.arch_program.get_or_insert_with(|| {
                    ArchProgram::from_template(ArchProgramTemplate::BalancedOps)
                });
                match button.action {
                    ProgramButtonAction::CycleTemplate => {
                        *program = ArchProgram::from_template(program.template.next());
                    }
                    ProgramButtonAction::AdjustConstant { index, delta } => {
                        if let Some(constant) = program.constants.get_mut(index) {
                            *constant = (*constant + delta).clamp(0, 20);
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {}
        }
    }
}

pub(crate) fn rotate_selected_tool(
    keys: Res<ButtonInput<KeyCode>>,
    mut tool_state: ResMut<EditorToolState>,
) {
    if keys.just_pressed(KeyCode::KeyQ) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 3) % 4;
    }

    if keys.just_pressed(KeyCode::KeyE) {
        tool_state.selected_rotation = (tool_state.selected_rotation + 1) % 4;
    }
}

pub(crate) fn place_or_remove_tile(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut editor_ship: ResMut<EditorShip>,
    mut progression: ResMut<DemoProgression>,
    editor_session: Res<EditorSessionState>,
    tool_state: Res<EditorToolState>,
) {
    let window = window.into_inner();

    if is_cursor_over_editor_ui(window) {
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        let selected_cost = if editor_session.mode == EditorMode::Player {
            module_kind_cost(tool_state.selected_kind)
        } else {
            0
        };
        if let Some(existing) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
            if existing.kind == tool_state.selected_kind {
                existing.rotation_quadrants = tool_state.selected_rotation;
                return;
            }

            let existing_cost = if editor_session.mode == EditorMode::Player {
                module_kind_cost(existing.kind)
            } else {
                0
            };
            let additional_cost = selected_cost.saturating_sub(existing_cost);
            if progression.scrap < additional_cost {
                return;
            }

            progression.scrap -= additional_cost;
            existing.kind = tool_state.selected_kind;
            existing.rotation_quadrants = tool_state.selected_rotation;
        } else {
            if progression.scrap < selected_cost {
                return;
            }
            progression.scrap -= selected_cost;
            let next_id = editor_ship.ship.next_module_id();
            editor_ship.ship.replace_module(ShipModule::new(
                next_id,
                tool_state.selected_kind,
                grid_x,
                grid_y,
                tool_state.selected_rotation,
            ));
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        editor_ship.ship.remove_module_at(grid_x, grid_y);
    }
}

pub(crate) fn save_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    enemy_library_state: Res<EnemyShipLibraryState>,
) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => save_default_enemy_library(&enemy_library_state.library),
    };
    if let Err(error) = result {
        eprintln!("editor: failed to save ship data: {error}");
    }
}

pub(crate) fn pan_and_zoom_editor_view(
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut view_state: ResMut<EditorViewState>,
    camera_query: Single<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, With<crate::client::state::MainCamera>),
    >,
) {
    let window = window.into_inner();
    let (mut camera_transform, mut projection) = camera_query.into_inner();

    for event in mouse_wheel.read() {
        let zoom_step = (1.0 - event.y * 0.08).clamp(0.75, 1.25);
        view_state.zoom = (view_state.zoom * zoom_step).clamp(0.35, 2.75);
    }

    if mouse_buttons.pressed(MouseButton::Middle) && !is_cursor_over_editor_ui(window) {
        for event in mouse_motion.read() {
            view_state.center.x -= event.delta.x * view_state.zoom;
            view_state.center.y += event.delta.y * view_state.zoom;
        }
    } else {
        mouse_motion.clear();
    }

    camera_transform.translation.x = view_state.center.x;
    camera_transform.translation.y = view_state.center.y;
    projection.scale = view_state.zoom;
}

pub(crate) fn load_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
) {
    if !keys.just_pressed(KeyCode::F9) {
        return;
    }

    match editor_session.mode {
        EditorMode::Player => match load_default_ship() {
            Ok(Some(saved_ship)) => {
                editor_ship.ship = saved_ship;
            }
            Ok(None) => {
                eprintln!("editor: no saved ship file was found to load");
            }
            Err(error) => {
                eprintln!("editor: failed to load ship: {error}");
            }
        },
        EditorMode::Enemy => match load_default_enemy_library() {
            Ok(Some(library)) => {
                enemy_library_state.library = library;
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
            Ok(None) => {
                eprintln!("editor: no enemy ship library file was found to load");
            }
            Err(error) => {
                eprintln!("editor: failed to load enemy ship library: {error}");
            }
        },
    }
}

pub(crate) fn persist_editor_ship(
    editor_ship: Res<EditorShip>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
) {
    if !editor_ship.is_changed() {
        return;
    }

    let result = match editor_session.mode {
        EditorMode::Player => save_default_ship(&editor_ship.ship),
        EditorMode::Enemy => {
            enemy_library_state.library.ensure_seeded();
            let selected_index = enemy_library_state.selected_index;
            if let Some(entry) = enemy_library_state
                .library
                .selected_or_first_mut(selected_index)
            {
                entry.ship = editor_ship.ship.clone();
                entry.display_name = editor_ship.ship.name.clone();
            }
            save_default_enemy_library(&enemy_library_state.library)
        }
    };

    if let Err(error) = result {
        eprintln!("editor: failed to autosave ship: {error}");
    }
}

pub(crate) fn enemy_library_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&EnemyPrevButton>,
            Option<&EnemyNextButton>,
            Option<&EnemyNewButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
) {
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    for (interaction, mut background, prev, next, new_entry) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                enemy_library_state.library.ensure_seeded();
                if prev.is_some() && !enemy_library_state.library.entries.is_empty() {
                    let len = enemy_library_state.library.entries.len();
                    enemy_library_state.selected_index =
                        (enemy_library_state.selected_index + len - 1) % len;
                } else if next.is_some() && !enemy_library_state.library.entries.is_empty() {
                    enemy_library_state.selected_index = (enemy_library_state.selected_index + 1)
                        % enemy_library_state.library.entries.len();
                } else if new_entry.is_some() {
                    enemy_library_state.selected_index =
                        enemy_library_state.library.add_blank_entry();
                }
                if let Some(entry) = enemy_library_state
                    .library
                    .selected_or_first(enemy_library_state.selected_index)
                {
                    editor_ship.ship = entry.ship.clone();
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.24, 0.32, 0.48));
            }
        }
    }
}

pub(crate) fn enemy_library_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    editor_session: Res<EditorSessionState>,
    mut enemy_library_state: ResMut<EnemyShipLibraryState>,
    mut editor_ship: ResMut<EditorShip>,
) {
    if editor_session.mode != EditorMode::Enemy {
        return;
    }

    let mut changed = false;
    enemy_library_state.library.ensure_seeded();
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketLeft) {
        let len = enemy_library_state.library.entries.len();
        enemy_library_state.selected_index = (enemy_library_state.selected_index + len - 1) % len;
        changed = true;
    }
    if !enemy_library_state.library.entries.is_empty() && keys.just_pressed(KeyCode::BracketRight) {
        enemy_library_state.selected_index =
            (enemy_library_state.selected_index + 1) % enemy_library_state.library.entries.len();
        changed = true;
    }
    if keys.just_pressed(KeyCode::KeyN) {
        enemy_library_state.selected_index = enemy_library_state.library.add_blank_entry();
        changed = true;
    }

    if changed
        && let Some(entry) = enemy_library_state
            .library
            .selected_or_first(enemy_library_state.selected_index)
    {
        editor_ship.ship = entry.ship.clone();
    }
}

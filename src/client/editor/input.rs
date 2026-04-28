use bevy::{input::mouse::MouseButton, prelude::*, window::PrimaryWindow};

use super::{
    super::{
        HOVERED_BUTTON,
        PRESSED_BUTTON,
        state::{
            ClientAppState,
            ComputerProgramButton,
            DemoProgression,
            EditorShip,
            EditorToolState,
            LaunchButton,
            ProgramButtonAction,
            ToolboxButton,
        },
    },
    helpers::{cursor_grid_position, is_cursor_over_toolbox, module_kind_cost},
};
use crate::ship::{
    arch::{ArchProgram, ArchProgramTemplate},
    ShipModule,
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

pub(crate) fn launch_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, With<LaunchButton>),
    >,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    for (interaction, mut background) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgb(0.12, 0.40, 0.24));
                next_state.set(ClientAppState::Playing);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgb(0.24, 0.62, 0.38));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.18, 0.50, 0.30));
            }
        }
    }
}

pub(crate) fn launch_keyboard_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    if keys.just_pressed(KeyCode::KeyL) {
        next_state.set(ClientAppState::Playing);
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
                let program = module
                    .arch_program
                    .get_or_insert_with(|| ArchProgram::from_template(ArchProgramTemplate::BalancedOps));
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
    tool_state: Res<EditorToolState>,
) {
    let window = window.into_inner();

    if is_cursor_over_toolbox(window) {
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        let selected_cost = module_kind_cost(tool_state.selected_kind);
        if let Some(existing) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
            if existing.kind == tool_state.selected_kind {
                existing.rotation_quadrants = tool_state.selected_rotation;
                return;
            }

            let existing_cost = module_kind_cost(existing.kind);
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
) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }

    if let Err(error) = save_default_ship(&editor_ship.ship) {
        eprintln!("editor: failed to save ship: {error}");
    }
}

pub(crate) fn load_editor_ship_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut editor_ship: ResMut<EditorShip>,
) {
    if !keys.just_pressed(KeyCode::F9) {
        return;
    }

    match load_default_ship() {
        Ok(Some(saved_ship)) => {
            editor_ship.ship = saved_ship;
        }
        Ok(None) => {
            eprintln!("editor: no saved ship file was found to load");
        }
        Err(error) => {
            eprintln!("editor: failed to load ship: {error}");
        }
    }
}

pub(crate) fn persist_editor_ship(editor_ship: Res<EditorShip>) {
    if !editor_ship.is_changed() {
        return;
    }

    if let Err(error) = save_default_ship(&editor_ship.ship) {
        eprintln!("editor: failed to autosave ship: {error}");
    }
}

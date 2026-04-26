use std::f32::consts::FRAC_PI_2;

use bevy::{
    input::mouse::MouseButton,
    prelude::*,
    window::PrimaryWindow,
};

use crate::ship::{ModuleKind, ShipDefinition, ShipModule};

use super::{
    GRID_COLOR, HALF_TILE_SIZE, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON, SELECTED_BUTTON,
    TILE_SIZE, TOOLBOX_COMPONENTS, TOOLBOX_WIDTH,
    state::{
        ClientAppState, ConnectionStatus, DemoProgression, EditorShip, EditorStatusText,
        EditorToolState, EditingCleanup, EditorRoot, LastMissionReport, LaunchButton,
        PreviewTile, ShipTileSprite, ToolboxButton,
    },
};

const UNAFFORDABLE_BUTTON: Color = Color::srgb(0.22, 0.22, 0.26);
const SELECTED_UNAFFORDABLE_BUTTON: Color = Color::srgb(0.44, 0.24, 0.16);

pub(crate) fn initialize_editor_ship(
    status: Res<ConnectionStatus>,
    mut editor_ship: ResMut<EditorShip>,
    mut tool_state: ResMut<EditorToolState>,
) {
    if let Some(snapshot) = status.active_snapshot.as_ref() {
        editor_ship.ship = snapshot.clone();
    } else {
        editor_ship.ship = ShipDefinition::empty("Untitled Knot");
    }

    tool_state.selected_kind = ModuleKind::Hull;
    tool_state.selected_rotation = 0;
}

pub(crate) fn spawn_editor_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
) {
    let title_font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let mono_font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            EditorRoot,
            EditingCleanup,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(TOOLBOX_WIDTH),
                    height: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
            ))
            .with_children(|toolbox| {
                toolbox.spawn((
                    Text::new("Toolbox"),
                    TextFont {
                        font: title_font.clone(),
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                toolbox.spawn((
                    Text::new(
                        "Click a component, then place it on the grid.\nLeft click: place/replace\nRight click: erase\nQ/E: rotate\nL or Launch: runtime scene",
                    ),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.74, 0.78, 0.86)),
                ));

                for kind in TOOLBOX_COMPONENTS {
                    toolbox
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(38.0),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                padding: UiRect::horizontal(Val::Px(10.0)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(8.0)),
                            BackgroundColor(NORMAL_BUTTON),
                            ToolboxButton { kind },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(format!("{} [{}]", kind.as_str(), module_kind_cost(kind))),
                                TextFont {
                                    font: mono_font.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }

                toolbox
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(46.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderRadius::all(Val::Px(8.0)),
                        BackgroundColor(Color::srgb(0.18, 0.50, 0.30)),
                        LaunchButton,
                    ))
                    .with_child((
                        Text::new("Launch"),
                        TextFont {
                            font: mono_font.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    top: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|hud| {
                hud.spawn((
                    Text::new(editor_status_line(
                        &editor_ship.ship.name,
                        &TOOLBOX_COMPONENTS[0],
                        0,
                        editor_ship.ship.modules.len(),
                        progression.scrap,
                        &last_mission_report,
                    )),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.94, 0.98)),
                    EditorStatusText,
                ));
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    bottom: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    max_width: Val::Px(320.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.10, 0.15, 0.92)),
                BorderRadius::all(Val::Px(10.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(
                        "Editor Controls\nLeft click: place or replace\nRight click: erase\nQ / E: rotate selected part\nL: launch mission\nCosts are shown in [scrap]",
                    ),
                    TextFont {
                        font: mono_font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.82, 0.86, 0.92)),
                ));
            });
        });
}

pub(crate) fn spawn_preview_tile(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tool_state: Res<EditorToolState>,
) {
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.72),
            ..Sprite::from_image(asset_server.load(sprite_path_for_kind(
                &tool_state.selected_kind,
            )))
        },
        Transform::from_xyz(0.0, 0.0, 5.0),
        Visibility::Hidden,
        PreviewTile,
        EditingCleanup,
    ));
}

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
                    SELECTED_UNAFFORDABLE_BUTTON
                });
            }
            Interaction::Hovered => {
                *background = BackgroundColor(if affordable {
                    HOVERED_BUTTON
                } else {
                    UNAFFORDABLE_BUTTON
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
        if let Some(existing) = editor_ship.ship.module_at_mut(grid_x, grid_y) {
            progression.scrap += module_kind_cost(existing.kind);
        }
        editor_ship.ship.remove_module_at(grid_x, grid_y);
    }
}

pub(crate) fn sync_preview_tile(
    window: Single<&Window, With<PrimaryWindow>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    tool_state: Res<EditorToolState>,
    asset_server: Res<AssetServer>,
    preview_query: Single<(&mut Sprite, &mut Transform, &mut Visibility), With<PreviewTile>>,
) {
    let (mut sprite, mut transform, mut visibility) = preview_query.into_inner();
    let window = window.into_inner();

    if is_cursor_over_toolbox(window) {
        *visibility = Visibility::Hidden;
        return;
    }

    let Some((grid_x, grid_y)) = cursor_grid_position(window, *camera_query) else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Visible;
    sprite.image = asset_server.load(sprite_path_for_kind(&tool_state.selected_kind));
    transform.translation = grid_to_world(grid_x, grid_y, 5.0);
    transform.rotation =
        Quat::from_rotation_z(-(tool_state.selected_rotation as f32) * FRAC_PI_2);
}

pub(crate) fn sync_ship_tile_entities(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    editor_ship: Res<EditorShip>,
    existing_tiles: Query<Entity, With<ShipTileSprite>>,
) {
    if !editor_ship.is_changed() {
        return;
    }

    for entity in &existing_tiles {
        commands.entity(entity).despawn();
    }

    for module in &editor_ship.ship.modules {
        commands.spawn((
            Sprite::from_image(asset_server.load(sprite_path_for_kind(&module.kind))),
            Transform {
                translation: grid_to_world(module.grid_x, module.grid_y, 1.0),
                rotation: Quat::from_rotation_z(-(module.rotation_quadrants as f32) * FRAC_PI_2),
                ..default()
            },
            ShipTileSprite,
            EditingCleanup,
        ));
    }
}

pub(crate) fn sync_toolbox_visuals(
    tool_state: Res<EditorToolState>,
    progression: Res<DemoProgression>,
    mut query: Query<(&ToolboxButton, &mut BackgroundColor)>,
) {
    if !tool_state.is_changed() && !progression.is_changed() {
        return;
    }

    for (button, mut background) in &mut query {
        let affordable = progression.scrap >= module_kind_cost(button.kind);
        if button.kind == tool_state.selected_kind {
            *background = BackgroundColor(if affordable {
                SELECTED_BUTTON
            } else {
                SELECTED_UNAFFORDABLE_BUTTON
            });
        } else {
            *background = BackgroundColor(if affordable {
                NORMAL_BUTTON
            } else {
                UNAFFORDABLE_BUTTON
            });
        }
    }
}

pub(crate) fn update_editor_status_text(
    editor_ship: Res<EditorShip>,
    tool_state: Res<EditorToolState>,
    progression: Res<DemoProgression>,
    last_mission_report: Res<LastMissionReport>,
    mut query: Query<&mut Text, With<EditorStatusText>>,
) {
    if !editor_ship.is_changed()
        && !tool_state.is_changed()
        && !progression.is_changed()
        && !last_mission_report.is_changed()
    {
        return;
    }

    for mut text in &mut query {
        **text = editor_status_line(
            &editor_ship.ship.name,
            &tool_state.selected_kind,
            tool_state.selected_rotation,
            editor_ship.ship.modules.len(),
            progression.scrap,
            &last_mission_report,
        );
    }
}

pub(crate) fn draw_grid_overlay(window: Single<&Window, With<PrimaryWindow>>, mut gizmos: Gizmos) {
    let window = window.into_inner();
    let half_w = (window.width() * 0.5) + TILE_SIZE * 4.0;
    let half_h = (window.height() * 0.5) + TILE_SIZE * 4.0;

    let min_x = ((-half_w - HALF_TILE_SIZE) / TILE_SIZE).floor() as i32;
    let max_x = ((half_w + HALF_TILE_SIZE) / TILE_SIZE).ceil() as i32;
    let min_y = ((-half_h - HALF_TILE_SIZE) / TILE_SIZE).floor() as i32;
    let max_y = ((half_h + HALF_TILE_SIZE) / TILE_SIZE).ceil() as i32;

    for grid_x in min_x..=max_x {
        let x = grid_x as f32 * TILE_SIZE - HALF_TILE_SIZE;
        gizmos.line_2d(
            Vec2::new(x, min_y as f32 * TILE_SIZE - HALF_TILE_SIZE),
            Vec2::new(x, max_y as f32 * TILE_SIZE - HALF_TILE_SIZE),
            GRID_COLOR,
        );
    }

    for grid_y in min_y..=max_y {
        let y = grid_y as f32 * TILE_SIZE - HALF_TILE_SIZE;
        gizmos.line_2d(
            Vec2::new(min_x as f32 * TILE_SIZE - HALF_TILE_SIZE, y),
            Vec2::new(max_x as f32 * TILE_SIZE - HALF_TILE_SIZE, y),
            GRID_COLOR,
        );
    }
}

pub(crate) fn cleanup_editor_entities(
    mut commands: Commands,
    query: Query<Entity, With<EditingCleanup>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn editor_status_line(
    ship_name: &str,
    selected_kind: &ModuleKind,
    selected_rotation: u8,
    module_count: usize,
    scrap_total: u32,
    last_mission_report: &LastMissionReport,
) -> String {
    let selected_cost = module_kind_cost(*selected_kind);
    let affordability = if scrap_total >= selected_cost {
        "ready"
    } else {
        "need more scrap"
    };
    let mission_summary = match (&last_mission_report.headline, &last_mission_report.detail) {
        (Some(headline), Some(detail)) => format!(
            "\nLast Mission: {headline}\n{detail}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded,
            last_mission_report.total_scrap,
        ),
        (Some(headline), None) => format!(
            "\nLast Mission: {headline}\nScrap Awarded: {}\nTotal Scrap: {}",
            last_mission_report.scrap_awarded,
            last_mission_report.total_scrap,
        ),
        _ => String::new(),
    };

    format!(
        "Editor Status\nShip: {ship_name}\nSelected Tool: {selected_kind}\nRotation: {selected_rotation}\nPlaced Modules: {module_count}\nScrap: {scrap_total}\nPlacement Cost: {selected_cost} ({affordability}){mission_summary}"
    )
}

fn module_kind_cost(kind: ModuleKind) -> u32 {
    match kind {
        ModuleKind::Interior => 0,
        ModuleKind::Hull | ModuleKind::HullCorner => 1,
        ModuleKind::Battery | ModuleKind::Cargo | ModuleKind::Airlock => 2,
        ModuleKind::Engine => 3,
        ModuleKind::Cockpit | ModuleKind::Turret => 4,
        ModuleKind::Reactor => 5,
        ModuleKind::Core => 6,
    }
}

fn cursor_grid_position(
    window: &Window,
    (camera, camera_transform): (&Camera, &GlobalTransform),
) -> Option<(i32, i32)> {
    let cursor = window.cursor_position()?;
    let world = camera.viewport_to_world_2d(camera_transform, cursor).ok()?;
    Some((
        ((world.x + HALF_TILE_SIZE) / TILE_SIZE).floor() as i32,
        ((-world.y + HALF_TILE_SIZE) / TILE_SIZE).floor() as i32,
    ))
}

fn grid_to_world(grid_x: i32, grid_y: i32, z: f32) -> Vec3 {
    Vec3::new(grid_x as f32 * TILE_SIZE, -(grid_y as f32) * TILE_SIZE, z)
}

fn is_cursor_over_toolbox(window: &Window) -> bool {
    let Some(cursor) = window.cursor_position() else {
        return false;
    };
    cursor.x <= TOOLBOX_WIDTH
}

fn sprite_path_for_kind(kind: &ModuleKind) -> String {
    format!("tiles/{}.png", kind.as_str())
}

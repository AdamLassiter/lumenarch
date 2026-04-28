mod editor;
mod gameplay;
mod menu;
mod net;
mod state;

use bevy::{app::AppExit, prelude::*};

use self::state::{
    ClientAppState,
    ConnectionConfig,
    ConnectionMailbox,
    ConnectionStatus,
    DebugOverlayState,
    DemoProgression,
    EditorShip,
    EditorToolState,
    LastMissionReport,
    MainCamera,
};
use crate::ship::ModuleKind;

pub(crate) const DEFAULT_HOST_ADDR: &str = "127.0.0.1:5000";
pub(crate) const TILE_SIZE: f32 = 32.0;
pub(crate) const HALF_TILE_SIZE: f32 = TILE_SIZE * 0.5;
pub(crate) const TOOLBOX_WIDTH: f32 = 280.0;
pub(crate) const NORMAL_BUTTON: Color = Color::srgb(0.24, 0.47, 0.78);
pub(crate) const HOVERED_BUTTON: Color = Color::srgb(0.30, 0.55, 0.88);
pub(crate) const PRESSED_BUTTON: Color = Color::srgb(0.18, 0.36, 0.62);
pub(crate) const SELECTED_BUTTON: Color = Color::srgb(0.78, 0.48, 0.20);
pub(crate) const GRID_COLOR: Color = Color::srgba(0.38, 0.45, 0.56, 0.28);
pub(crate) const TOOLBOX_COMPONENTS: [ModuleKind; 13] = ModuleKind::ALL;

pub fn run_client() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.08)))
        .insert_resource(ConnectionConfig::default())
        .insert_resource(ConnectionStatus::default())
        .insert_resource(ConnectionMailbox::default())
        .insert_resource(EditorShip::default())
        .insert_resource(DemoProgression::default())
        .insert_resource(DebugOverlayState::default())
        .insert_resource(LastMissionReport::default())
        .insert_resource(EditorToolState::default())
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LUMEN//ARCH Ship Editor".to_string(),
                        resolution: (1280.0, 720.0).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<ClientAppState>()
        .add_event::<gameplay::components::InteractWithModule>()
        .add_event::<gameplay::components::BeginHeldInteraction>()
        .add_event::<gameplay::components::CompleteHeldInteraction>()
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(ClientAppState::Menu), menu::spawn_menu_ui)
        .add_systems(
            Update,
            (
                menu::edit_host_address.run_if(in_state(ClientAppState::Menu)),
                menu::menu_button_system.run_if(in_state(ClientAppState::Menu)),
                menu::menu_keyboard_shortcuts.run_if(in_state(ClientAppState::Menu)),
                net::poll_connection_events,
                menu::update_menu_status_text.run_if(in_state(ClientAppState::Menu)),
                menu::update_host_address_text.run_if(in_state(ClientAppState::Menu)),
                exit_on_escape,
            ),
        )
        .add_systems(OnExit(ClientAppState::Menu), menu::cleanup_menu_ui)
        .add_systems(
            OnEnter(ClientAppState::Editing),
            (
                editor::initialize_editor_ship,
                editor::spawn_editor_ui,
                editor::spawn_preview_tile,
            ),
        )
        .add_systems(
            Update,
            (
                editor::draw_grid_overlay.run_if(in_state(ClientAppState::Editing)),
                editor::toolbox_button_system.run_if(in_state(ClientAppState::Editing)),
                editor::computer_program_button_system.run_if(in_state(ClientAppState::Editing)),
                editor::launch_button_system.run_if(in_state(ClientAppState::Editing)),
                editor::launch_keyboard_shortcut.run_if(in_state(ClientAppState::Editing)),
                editor::rotate_selected_tool.run_if(in_state(ClientAppState::Editing)),
                editor::place_or_remove_tile.run_if(in_state(ClientAppState::Editing)),
                editor::save_editor_ship_shortcut.run_if(in_state(ClientAppState::Editing)),
                editor::load_editor_ship_shortcut.run_if(in_state(ClientAppState::Editing)),
                editor::persist_editor_ship.run_if(in_state(ClientAppState::Editing)),
                editor::sync_preview_tile.run_if(in_state(ClientAppState::Editing)),
                editor::sync_ship_tile_entities.run_if(in_state(ClientAppState::Editing)),
                editor::sync_computer_program_entries.run_if(in_state(ClientAppState::Editing)),
                editor::sync_toolbox_visuals.run_if(in_state(ClientAppState::Editing)),
                editor::update_editor_status_text.run_if(in_state(ClientAppState::Editing)),
            ),
        )
        .add_systems(
            OnExit(ClientAppState::Editing),
            editor::cleanup_editor_entities,
        )
        .add_systems(
            OnEnter(ClientAppState::Playing),
            gameplay::spawn_runtime_scene,
        )
        .add_systems(
            Update,
            (
                gameplay::return_button_system.run_if(in_state(ClientAppState::Playing)),
                gameplay::return_keyboard_shortcut.run_if(in_state(ClientAppState::Playing)),
                gameplay::toggle_debug_overlay.run_if(in_state(ClientAppState::Playing)),
                (
                    gameplay::toggle_shipboard_control_mode,
                    gameplay::exit_focused_station,
                    gameplay::move_shipboard_player,
                    gameplay::detect_nearby_interactions,
                    gameplay::run_shipboard_interaction_input,
                    gameplay::begin_held_interactions,
                    gameplay::complete_held_interactions,
                    gameplay::apply_module_interactions,
                    gameplay::update_station_command_input,
                    gameplay::sample_ship_fields,
                    gameplay::update_module_runtime_state,
                    gameplay::run_arch_automation,
                    gameplay::run_logistics_transfers,
                    gameplay::run_processors,
                    gameplay::update_mission_telemetry,
                    gameplay::tick_recent_action_feedback,
                )
                    .chain()
                    .run_if(in_state(ClientAppState::Playing)),
                (
                    gameplay::sync_runtime_ship_state,
                    gameplay::apply_player_ship_controls,
                    gameplay::fire_player_weapons,
                    gameplay::aim_hostile_turrets,
                    gameplay::fire_hostile_targets,
                    gameplay::advance_projectiles,
                    gameplay::handle_projectile_hits,
                    gameplay::update_mission_state,
                    gameplay::collect_salvage,
                    gameplay::return_after_mission_resolution,
                    gameplay::update_destroyed_module_visuals,
                    gameplay::sync_shipboard_player_visual,
                    gameplay::integrate_player_ship_motion,
                    gameplay::camera_follow_player_ship,
                    gameplay::draw_debug_overlay,
                    gameplay::update_gameplay_status_text,
                    gameplay::update_inspection_and_alerts_text,
                )
                    .chain()
                    .run_if(in_state(ClientAppState::Playing)),
            ),
        )
        .add_systems(
            OnExit(ClientAppState::Playing),
            gameplay::cleanup_runtime_entities,
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

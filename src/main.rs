mod ship;
#[cfg(test)]
mod sim_tests;

mod balance;
pub(crate) mod campaign;
mod docked;
mod editor;
pub(crate) mod gameplay;
pub(crate) mod helpers;
mod lobby;
mod netcode;
mod sector_map;
mod splash;
pub(crate) mod state;
mod station_editor;
mod stations;

use std::{env, time::Duration};

use bevy::{app::AppExit, prelude::*};
use bevy_ggrs::{GgrsPlugin, RollbackApp, RollbackFrameRate};
use bevy_yarnspinner::prelude::{YarnFileSource, YarnSpinnerPlugin};

use self::state::{
    ArchEditorState,
    CampaignLoadState,
    DebugOverlayState,
    DockedState,
    EditorPanState,
    EditorPointerState,
    EditorSelectionState,
    EditorSessionState,
    EditorShip,
    EditorToolState,
    EditorUiState,
    EditorViewState,
    EnemyEditorState,
    EnemyShipLibraryState,
    FrontendMode,
    GraphicsOptions,
    LastMissionReport,
    MainCamera,
    Progression,
    SectorMapPanState,
    SectorMapViewState,
    SectorState,
    SplashScreenState,
};
use crate::netcode::LumenGgrsConfig;

pub(crate) const TICK_FPS: u64 = 30;
pub(crate) const TICK_MILLIS: u64 = 1000 / TICK_FPS;

pub(crate) const DEFAULT_HOST_ADDR: &str = "127.0.0.1:5000";
pub(crate) const TILE_SIZE: f32 = 32.0;
pub(crate) const HALF_TILE_SIZE: f32 = TILE_SIZE * 0.5;
pub(crate) const TOOLBOX_WIDTH: f32 = 280.0;
pub(crate) const UI_PANEL_RADIUS: f32 = 12.0;
pub(crate) const UI_BUTTON_RADIUS: f32 = 10.0;
pub(crate) const UI_TITLE_FONT_SIZE: f32 = 20.0;
pub(crate) const UI_BODY_FONT_SIZE: f32 = 14.0;
pub(crate) const UI_HELP_FONT_SIZE: f32 = 14.0;
pub(crate) const NORMAL_BUTTON: Color = Color::srgb(0.24, 0.47, 0.78);
pub(crate) const HOVERED_BUTTON: Color = Color::srgb(0.30, 0.55, 0.88);
pub(crate) const PRESSED_BUTTON: Color = Color::srgb(0.18, 0.36, 0.62);
pub(crate) const SELECTED_BUTTON: Color = Color::srgb(0.78, 0.48, 0.20);
pub(crate) const GRID_COLOR: Color = Color::srgba(0.38, 0.45, 0.56, 0.28);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppRuntimeMode {
    Interactive,
    Headless,
}

impl AppRuntimeMode {
    fn is_interactive(self) -> bool {
        matches!(self, Self::Interactive)
    }
}

pub fn run_client() {
    build_app(parse_runtime_mode()).run();
}

pub(crate) fn build_app(mode: AppRuntimeMode) -> App {
    let balance_config = balance::load_or_create_default_balance().unwrap_or_else(|error| {
        eprintln!("client: failed to load balance config, using defaults: {error}");
        balance::BalanceConfig::default()
    });

    let mut app = App::new();

    insert_core_resources(&mut app, balance_config);
    if mode.is_interactive() {
        app.insert_resource(SplashScreenState::interactive_startup());
    }
    add_core_plugins(&mut app, mode);
    register_rollback_state(&mut app);
    register_messages(&mut app);
    add_core_systems(&mut app, mode);
    if mode.is_interactive() {
        add_ui_systems(&mut app);
    }

    app
}

fn parse_runtime_mode() -> AppRuntimeMode {
    if env::args().any(|argument| argument == "--headless") {
        AppRuntimeMode::Headless
    } else {
        AppRuntimeMode::Interactive
    }
}

fn insert_core_resources(app: &mut App, balance_config: balance::BalanceConfig) {
    app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(
        TICK_MILLIS / 2,
    )))
    .insert_resource(RollbackFrameRate(TICK_FPS as usize))
    .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.08)))
    .insert_resource(ButtonInput::<KeyCode>::default())
    .insert_resource(ButtonInput::<MouseButton>::default())
    .insert_resource(balance_config)
    .insert_resource(netcode::SessionConfig::default())
    .insert_resource(netcode::SessionStatus::default())
    .insert_resource(netcode::SessionBootstrapConfig::default())
    .insert_resource(netcode::LobbyRuntime::default())
    .insert_resource(netcode::RollbackGameState::default())
    .insert_resource(netcode::LocalPlayerHandle::default())
    .insert_resource(netcode::PlayerHandleMap::default())
    .insert_resource(netcode::ObservedLocalPlayer::default())
    .insert_resource(netcode::PendingLocalMetaCommand::default())
    .insert_resource(netcode::PendingLocalStationCommand::default())
    .insert_resource(netcode::DecodedPlayerCommands::default())
    .insert_resource(netcode::ChecksumHistory::default())
    .insert_resource(netcode::ActivePresentationPhase::default())
    .insert_resource(state::GameplayInfoPanelMode::default())
    .insert_resource(state::LocalPlayerProfile::default())
    .insert_resource(state::FocusedTextBox::default())
    .insert_resource(state::TextBoxClipboard::default())
    .insert_resource(EditorShip::default())
    .insert_resource(EditorSessionState::default())
    .insert_resource(EnemyEditorState::default())
    .insert_resource(EnemyShipLibraryState::default())
    .insert_resource(Progression::default())
    .insert_resource(DockedState::default())
    .insert_resource(docked::DockedBoardState::default())
    .insert_resource(docked::DockedAvatarMemory::default())
    .insert_resource(docked::DockedDialogueState::default())
    .insert_resource(station_editor::StationEditorState::default())
    .insert_resource(SectorState::default())
    .insert_resource(CampaignLoadState::default())
    .insert_resource(DebugOverlayState::default())
    .insert_resource(GraphicsOptions::default())
    .insert_resource(LastMissionReport::default())
    .insert_resource(stations::StationCatalogResource::load_or_default())
    .insert_resource(EditorToolState::default())
    .insert_resource(EditorSelectionState::default())
    .insert_resource(EditorPointerState::default())
    .insert_resource(EditorUiState::default())
    .insert_resource(ArchEditorState::default())
    .insert_resource(state::ProgramTextEditorState::default())
    .insert_resource(EditorPanState::default())
    .insert_resource(EditorViewState::default())
    .insert_resource(SectorMapPanState::default())
    .insert_resource(SectorMapViewState::default())
    .insert_resource(SplashScreenState::default());
}

/// Reports whether shader-backed presentation effects should run for the current session.
fn shaders_enabled(options: Res<GraphicsOptions>) -> bool {
    options.shaders_enabled
}

fn add_core_plugins(app: &mut App, mode: AppRuntimeMode) {
    if mode.is_interactive() {
        app.add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "LUMEN//ARCH".to_string(),
                        resolution: (1280, 720).into(),
                        ..default()
                    }),
                    ..default()
                }),
        );
        app.add_plugins(gameplay::effects::GameplayEffectsPlugin);
    } else {
        app.add_plugins(MinimalPlugins)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_plugins(ImagePlugin::default_nearest())
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::transform::TransformPlugin);
        app.init_asset::<bevy::image::Image>()
            .init_asset::<bevy::text::Font>()
            .init_asset::<Mesh>()
            .init_asset::<gameplay::effects::EngineFlameMaterial>()
            .init_asset::<gameplay::effects::ReactorGlowMaterial>()
            .init_asset::<gameplay::effects::SpaceBackdropMaterial>();
    }

    app.add_plugins(GgrsPlugin::<LumenGgrsConfig>::default());
    app.add_plugins(YarnSpinnerPlugin::with_yarn_source(YarnFileSource::folder(
        "dialog",
    )));
    app.add_observer(docked::handle_docked_yarn_line)
        .add_observer(docked::handle_docked_yarn_options)
        .add_observer(docked::handle_docked_yarn_dialogue_completed);
}

fn register_rollback_state(app: &mut App) {
    app.rollback_resource_with_clone::<netcode::RollbackGameState>()
        .rollback_resource_with_clone::<netcode::PlayerHandleMap>()
        .rollback_resource_with_clone::<netcode::DecodedPlayerCommands>()
        .update_resource_with_map_entities::<netcode::PlayerHandleMap>()
        .rollback_component_with_clone::<gameplay::components::PlayerHandleComponent>()
        .rollback_component_with_clone::<gameplay::components::PlayerShipAssignment>()
        .rollback_component_with_clone::<gameplay::components::PlayerMotionState>()
        .rollback_component_with_clone::<gameplay::components::CarriedResource>()
        .rollback_component_with_clone::<gameplay::components::EquippedSuit>()
        .rollback_component_with_clone::<gameplay::components::PlayerIdentity>()
        .rollback_component_with_clone::<gameplay::components::PlayerConditionState>()
        .rollback_component_with_clone::<gameplay::components::PlayerFocusedTile>()
        .rollback_component_with_clone::<gameplay::components::CurrentStation>()
        .rollback_component_with_clone::<gameplay::components::InternalPosition>()
        .rollback_component_with_clone::<gameplay::components::ShipboardControlState>()
        .rollback_component_with_clone::<gameplay::components::NearbyInteraction>()
        .rollback_component_with_clone::<gameplay::components::HeldInteraction>()
        .rollback_component_with_clone::<gameplay::components::PlayerFieldState>()
        .rollback_component_with_clone::<gameplay::components::ShipInertiaField>()
        .rollback_component_with_clone::<gameplay::components::ShipInteriorMap>()
        .rollback_component_with_clone::<gameplay::components::ShipAtmosphereState>()
        .rollback_component_with_clone::<gameplay::components::RuntimeShipModule>()
        .rollback_component_with_clone::<gameplay::components::Integrity>()
        .rollback_component_with_clone::<gameplay::components::RuntimeArchComputer>()
        .rollback_component_with_clone::<gameplay::components::TurretCommandState>()
        .rollback_component_with_clone::<gameplay::components::ReactorCommandState>()
        .rollback_component_with_clone::<gameplay::components::JunctionCommandState>()
        .rollback_component_with_clone::<gameplay::components::ValveCommandState>()
        .rollback_component_with_clone::<gameplay::components::StorageCommandState>()
        .rollback_component_with_clone::<gameplay::components::AirlockCommandState>()
        .rollback_component_with_clone::<gameplay::components::ManipulatorCommandState>()
        .rollback_component_with_clone::<gameplay::components::ProcessorCommandState>()
        .rollback_component_with_clone::<gameplay::components::DroneStationCommandState>()
        .rollback_component_with_clone::<gameplay::components::ModuleRuntimeState>()
        .rollback_component_with_clone::<gameplay::components::ModuleFieldEmitter>()
        .rollback_component_with_clone::<gameplay::components::ShipMovementModel>()
        .rollback_component_with_clone::<gameplay::components::ShipPowerModel>()
        .rollback_component_with_clone::<gameplay::components::ShipPowerState>()
        .rollback_component_with_clone::<gameplay::components::ShipInfrastructureState>()
        .rollback_component_with_clone::<gameplay::components::ShipControlState>()
        .rollback_component_with_clone::<gameplay::components::ShipWeaponState>()
        .rollback_component_with_clone::<gameplay::components::ShipAutomationState>()
        .rollback_component_with_clone::<gameplay::components::ShipArchCommandState>()
        .rollback_component_with_clone::<gameplay::components::MissionState>()
        .rollback_component_with_clone::<gameplay::components::LinearVelocity>()
        .rollback_component_with_clone::<gameplay::components::AngularVelocity>()
        .rollback_component_with_clone::<gameplay::components::SimPosition>()
        .rollback_component_with_clone::<gameplay::components::SimRotation>()
        .rollback_component_with_clone::<gameplay::components::StorageModule>()
        .rollback_component_with_clone::<gameplay::components::ManipulatorModule>()
        .rollback_component_with_clone::<gameplay::components::ProcessorModule>()
        .rollback_component_with_clone::<gameplay::components::DroneStationModule>()
        .rollback_component_with_clone::<gameplay::components::DroneUnit>()
        .rollback_component_with_clone::<gameplay::components::LooseCargo>()
        .rollback_component_with_clone::<gameplay::components::Projectile>()
        .rollback_component_with_clone::<gameplay::components::HostileWeaponState>()
        .rollback_component_with_clone::<gameplay::components::SalvagePickup>()
        .update_component_with_map_entities::<gameplay::components::PlayerShipAssignment>()
        .update_component_with_map_entities::<gameplay::components::PlayerMotionState>()
        .update_component_with_map_entities::<gameplay::components::PlayerFocusedTile>()
        .update_component_with_map_entities::<gameplay::components::ShipboardControlState>()
        .update_component_with_map_entities::<gameplay::components::NearbyInteraction>()
        .update_component_with_map_entities::<gameplay::components::HeldInteraction>()
        .update_component_with_map_entities::<gameplay::components::DroneUnit>()
        .init_state::<FrontendMode>();
}

fn register_messages(app: &mut App) {
    app.add_message::<gameplay::components::InteractWithModule>()
        .add_message::<gameplay::components::BeginHeldInteraction>()
        .add_message::<gameplay::components::CompleteHeldInteraction>();
}

fn add_core_systems(app: &mut App, mode: AppRuntimeMode) {
    if mode.is_interactive() {
        app.add_systems(Startup, setup_camera);
    }

    app.add_systems(bevy_ggrs::ReadInputs, netcode::read_local_inputs)
        .add_systems(PreUpdate, netcode::sync_local_player_handle);

    add_rollback_systems(app);
    add_shared_update_systems(app);
    add_session_bootstrap_systems(app, mode);
}

fn add_rollback_systems(app: &mut App) {
    app.add_systems(
        bevy_ggrs::GgrsSchedule,
        (
            (netcode::decode_player_inputs, netcode::apply_host_meta_ops)
                .chain()
                .ambiguous_with_all(),
            (
                gameplay::exit_focused_station,
                gameplay::update_player_reference_frame,
                gameplay::update_ship_atmosphere,
                gameplay::sample_player_atmosphere,
                gameplay::move_shipboard_player,
                gameplay::update_player_focused_tiles,
                gameplay::update_current_station,
                gameplay::detect_nearby_interactions,
                gameplay::run_shipboard_interaction_input,
                gameplay::handle_player_cargo_interaction,
                gameplay::begin_held_interactions,
                gameplay::complete_held_interactions,
                gameplay::apply_module_interactions,
                gameplay::update_station_command_input,
            )
                .chain()
                .run_if(netcode::rollback_phase_is_encounter)
                .ambiguous_with_all(),
            (
                gameplay::rebuild_infrastructure_networks,
                gameplay::sample_ship_fields,
                gameplay::apply_player_environmental_effects,
                gameplay::update_module_runtime_state,
                gameplay::rebuild_infrastructure_networks,
                gameplay::update_routed_ship_power,
                gameplay::update_detector_modules,
                gameplay::run_arch_automation,
                gameplay::run_logistics_transfers,
                gameplay::sync_drone_station_population,
                gameplay::run_drone_logistics,
                gameplay::run_processors,
                gameplay::update_mission_telemetry,
                gameplay::tick_recent_action_feedback,
                gameplay::sync_hostile_ship_state,
            )
                .chain()
                .run_if(netcode::rollback_phase_is_encounter)
                .ambiguous_with_all(),
            (
                gameplay::sync_runtime_ship_state,
                gameplay::apply_player_ship_controls,
                gameplay::drive_hostile_ships,
                gameplay::fire_player_weapons,
                gameplay::fire_hostile_ship_weapons,
                gameplay::aim_hostile_turrets,
                gameplay::fire_hostile_targets,
                gameplay::advance_projectiles,
                gameplay::handle_projectile_hits,
                gameplay::update_mission_state,
                gameplay::collect_salvage,
                gameplay::return_after_mission_resolution,
            )
                .chain()
                .run_if(netcode::rollback_phase_is_encounter)
                .ambiguous_with_all(),
            netcode::advance_rollback_state.ambiguous_with_all(),
        ),
    );
}

fn add_shared_update_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            netcode::sync_presentation_from_rollback,
            netcode::sync_active_presentation_phase,
            netcode::sync_player_editor_mode,
            docked::initialize_campaign_state.run_if(netcode::session_presents_docked),
            docked::spawn_docked_spaceport_scene
                .run_if(netcode::session_presents_docked)
                .run_if(docked::docked_spaceport_scene_missing),
            docked::cleanup_docked_spaceport_scene
                .run_if(netcode::session_not_presents_docked)
                .run_if(docked::docked_spaceport_scene_present),
            docked::cleanup_docked_board_ui
                .run_if(netcode::session_not_presents_docked)
                .run_if(docked::docked_board_ui_present),
            gameplay::spawn_runtime_scene
                .run_if(netcode::session_presents_encounter)
                .run_if(gameplay::runtime_scene_missing),
            gameplay::cleanup_runtime_entities
                .run_if(netcode::session_not_presents_encounter)
                .run_if(gameplay::runtime_scene_present),
            sync_controls_help_panel_visibility,
        ),
    )
    .add_systems(Update, docked::persist_campaign_state);
}

/// Shows the controls help panel only while Alt is held so the HUD stays quiet until needed.
fn sync_controls_help_panel_visibility(
    keys: Res<ButtonInput<KeyCode>>,
    mut panels: Query<&mut Node, With<state::ControlsHelpPanel>>,
) {
    let show_help = keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]);
    let display = if show_help {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut panels {
        if node.display != display {
            node.display = display;
        }
    }
}

fn add_session_bootstrap_systems(app: &mut App, mode: AppRuntimeMode) {
    if mode.is_interactive() {
        app.add_systems(
            FixedUpdate,
            (
                netcode::poll_lobby_runtime_events,
                netcode::finalize_pending_session_bootstrap,
                exit_on_escape,
            ),
        );
    } else {
        app.add_systems(
            FixedUpdate,
            (
                netcode::poll_lobby_runtime_events,
                netcode::finalize_pending_session_bootstrap,
            ),
        );
    }
}

fn add_ui_systems(app: &mut App) {
    add_editor_keyboard_update_systems(app);
    add_session_ui_update_systems(app);
    add_lobby_ui_fixed_systems(app);
    add_docked_fixed_systems(app);
    add_sector_map_ui_fixed_systems(app);
    add_player_editor_ui_fixed_systems(app);
    add_debug_enemy_editor_fixed_systems(app);
    add_debug_station_editor_fixed_systems(app);
    add_encounter_presentation_systems(app);
}

fn add_editor_keyboard_update_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            (
                editor::leave_editor_keyboard_shortcut,
                editor::toggle_editor_module_overlay_shortcuts,
                editor::rotate_selected_tool,
            )
                .run_if(netcode::session_presents_player_editor),
            (
                editor::repair_selected_component_shortcut,
                editor::selection_shortcuts,
                editor::save_editor_ship_shortcut,
                editor::load_editor_ship_shortcut,
            )
                .run_if(netcode::session_presents_player_editor),
            (
                editor::enemy_library_keyboard_shortcuts,
                editor::leave_editor_keyboard_shortcut,
                editor::toggle_editor_module_overlay_shortcuts,
                editor::rotate_selected_tool,
            )
                .run_if(in_state(FrontendMode::DebugEnemyEditor)),
            (
                editor::selection_shortcuts,
                editor::save_editor_ship_shortcut,
                editor::load_editor_ship_shortcut,
            )
                .run_if(in_state(FrontendMode::DebugEnemyEditor)),
            (
                editor::leave_editor_keyboard_shortcut,
                station_editor::station_library_keyboard_shortcuts,
                editor::toggle_editor_module_overlay_shortcuts,
                editor::rotate_selected_tool,
            )
                .run_if(in_state(FrontendMode::DebugStationEditor)),
            (
                editor::selection_shortcuts,
                editor::save_editor_ship_shortcut,
                editor::load_editor_ship_shortcut,
            )
                .run_if(in_state(FrontendMode::DebugStationEditor)),
        ),
    );
}

fn add_session_ui_update_systems(app: &mut App) {
    app.add_systems(
        Update,
        (
            splash::spawn_splash_ui.run_if(splash::splash_active),
            splash::dismiss_splash_screen.run_if(netcode::session_presents_docked),
            splash::cleanup_splash_ui.run_if(splash::splash_inactive),
            lobby::spawn_lobby_ui
                .run_if(netcode::frontend_mode_is_lobby)
                .run_if(lobby::lobby_ui_missing),
            lobby::cleanup_lobby_ui
                .run_if(netcode::frontend_mode_is_not_lobby)
                .run_if(lobby::lobby_ui_present),
            sector_map::spawn_sector_map_ui
                .run_if(netcode::session_presents_sector_map)
                .run_if(sector_map::sector_map_ui_missing),
            sector_map::cleanup_sector_map_ui
                .run_if(netcode::session_not_presents_sector_map)
                .run_if(sector_map::sector_map_ui_present),
            (
                editor::initialize_editor_ship,
                editor::spawn_editor_ui,
                editor::spawn_preview_tile,
            )
                .chain()
                .run_if(netcode::session_presents_player_editor)
                .run_if(editor::editor_ui_missing),
            editor::cleanup_editor_entities
                .run_if(netcode::editor_ui_should_not_be_present)
                .run_if(editor::editor_ui_present),
            (
                editor::initialize_editor_ship,
                editor::spawn_editor_ui,
                editor::spawn_preview_tile,
            )
                .chain()
                .run_if(netcode::frontend_mode_is_debug_enemy_editor)
                .run_if(editor::editor_ui_missing),
            (
                editor::initialize_editor_ship,
                editor::spawn_editor_ui,
                editor::spawn_preview_tile,
            )
                .chain()
                .run_if(in_state(FrontendMode::DebugStationEditor))
                .run_if(editor::editor_ui_missing),
        ),
    );
}

fn add_lobby_ui_fixed_systems(app: &mut App) {
    app.add_systems(
        Update,
        splash::tick_splash_screen.run_if(splash::splash_active),
    );
    app.add_systems(
        FixedUpdate,
        (
            lobby::focus_textbox_on_click.run_if(in_state(FrontendMode::Lobby)),
            lobby::edit_lobby_textboxes.run_if(in_state(FrontendMode::Lobby)),
            lobby::lobby_button_system.run_if(in_state(FrontendMode::Lobby)),
            lobby::lobby_keyboard_shortcuts.run_if(in_state(FrontendMode::Lobby)),
            netcode::sync_lobby_profile_changes.run_if(in_state(FrontendMode::Lobby)),
            lobby::update_lobby_textboxes.run_if(in_state(FrontendMode::Lobby)),
            lobby::update_lobby_status_text.run_if(in_state(FrontendMode::Lobby)),
        ),
    );
}

fn add_docked_fixed_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            docked::move_docked_local_avatar
                .run_if(netcode::session_presents_docked)
                .run_if(docked::docked_board_closed),
            (
                docked::docked_board_keyboard_system,
                docked::docked_board_button_system,
                docked::sync_docked_board_ui,
            )
                .chain()
                .run_if(netcode::session_presents_docked),
            (
                docked::docked_keyboard_interaction_system,
                docked::sync_docked_yarn_runner,
                docked::sync_docked_dialogue_overlay,
                docked::docked_dialogue_button_system,
            )
                .chain()
                .run_if(netcode::session_presents_docked)
                .run_if(docked::docked_board_closed),
        ),
    );
}

fn add_sector_map_ui_fixed_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            sector_map::sector_node_button_system.run_if(netcode::session_presents_sector_map),
            sector_map::sector_navigation_button_system
                .run_if(netcode::session_presents_sector_map),
            sector_map::sector_navigation_keyboard_system
                .run_if(netcode::session_presents_sector_map),
            sector_map::pan_and_zoom_sector_map.run_if(netcode::session_presents_sector_map),
            sector_map::sync_sector_map_layout.run_if(netcode::session_presents_sector_map),
            sector_map::update_sector_map_text.run_if(netcode::session_presents_sector_map),
        ),
    );
}

fn add_player_editor_ui_fixed_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            (
                editor::draw_grid_overlay,
                editor::draw_editor_selection_overlay,
                editor::toolbox_button_system,
                editor::selection_action_button_system,
                editor::mission_report_button_system,
                lobby::focus_textbox_on_click,
                lobby::clear_textbox_focus_on_non_textbox_click,
                lobby::edit_lobby_textboxes,
                editor::sync_program_text_editor_state,
                editor::focus_program_text_editor_on_click,
                editor::edit_program_text_editor,
                editor::program_editor_action_button_system,
                editor::editor_station_panel_button_system,
                editor::leave_editor_button_system,
            )
                .run_if(netcode::session_presents_player_editor),
            (
                (
                    editor::place_or_remove_tile,
                    editor::pan_and_zoom_editor_view,
                    editor::persist_editor_ship,
                    editor::sync_preview_tile,
                    editor::sync_ship_tile_entities,
                ),
                (
                    editor::update_editor_module_overlay,
                    editor::sync_toolbox_visuals,
                    editor::sync_toolbox_scroll,
                    lobby::update_lobby_textboxes,
                    editor::update_editor_status_text,
                ),
            )
                .run_if(netcode::session_presents_player_editor),
            editor::sync_editor_toolbox_layer_sections
                .run_if(netcode::session_presents_player_editor),
        ),
    );
}

fn add_debug_enemy_editor_fixed_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            (
                editor::draw_grid_overlay,
                editor::draw_editor_selection_overlay,
                editor::toolbox_button_system,
                editor::selection_action_button_system,
                editor::mission_report_button_system,
                editor::enemy_library_button_system,
                lobby::focus_textbox_on_click,
                lobby::clear_textbox_focus_on_non_textbox_click,
                lobby::edit_lobby_textboxes,
                editor::sync_program_text_editor_state,
                editor::focus_program_text_editor_on_click,
                editor::edit_program_text_editor,
                editor::program_editor_action_button_system,
                editor::editor_station_panel_button_system,
                editor::leave_editor_button_system,
            )
                .run_if(in_state(FrontendMode::DebugEnemyEditor)),
            (
                (
                    editor::place_or_remove_tile,
                    editor::pan_and_zoom_editor_view,
                    editor::persist_editor_ship,
                    editor::sync_preview_tile,
                    editor::sync_ship_tile_entities,
                ),
                (
                    editor::update_editor_module_overlay,
                    editor::sync_toolbox_visuals,
                    editor::sync_toolbox_scroll,
                    lobby::update_lobby_textboxes,
                    editor::update_editor_status_text,
                ),
            )
                .run_if(in_state(FrontendMode::DebugEnemyEditor)),
            editor::sync_editor_toolbox_layer_sections
                .run_if(in_state(FrontendMode::DebugEnemyEditor)),
        ),
    );
}

fn add_debug_station_editor_fixed_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            (
                editor::draw_grid_overlay,
                editor::draw_editor_selection_overlay,
                editor::toolbox_button_system,
                editor::selection_action_button_system,
                editor::mission_report_button_system,
                station_editor::station_library_button_system,
                lobby::focus_textbox_on_click,
                lobby::clear_textbox_focus_on_non_textbox_click,
                lobby::edit_lobby_textboxes,
                editor::sync_program_text_editor_state,
                editor::focus_program_text_editor_on_click,
                editor::edit_program_text_editor,
                editor::program_editor_action_button_system,
                editor::editor_station_panel_button_system,
                editor::leave_editor_button_system,
            )
                .run_if(in_state(FrontendMode::DebugStationEditor)),
            (
                (
                    editor::place_or_remove_tile,
                    editor::pan_and_zoom_editor_view,
                    editor::persist_editor_ship,
                    editor::sync_preview_tile,
                    editor::sync_ship_tile_entities,
                ),
                (
                    editor::update_editor_module_overlay,
                    editor::sync_toolbox_visuals,
                    editor::sync_toolbox_scroll,
                    lobby::update_lobby_textboxes,
                    editor::update_editor_status_text,
                    editor::update_station_config_references_text,
                ),
            )
                .run_if(in_state(FrontendMode::DebugStationEditor)),
            editor::sync_editor_toolbox_layer_sections
                .run_if(in_state(FrontendMode::DebugStationEditor)),
        ),
    );
}

fn add_encounter_presentation_systems(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            gameplay::return_button_system.run_if(netcode::session_presents_encounter),
            gameplay::return_keyboard_shortcut.run_if(netcode::session_presents_encounter),
            gameplay::toggle_gameplay_info_panel.run_if(netcode::session_presents_encounter),
            gameplay::log_runtime_hostile_scene_summary.run_if(netcode::session_presents_encounter),
            gameplay::sync_player_reference_frame_parenting
                .run_if(netcode::session_presents_encounter),
            (
                gameplay::update_destroyed_module_visuals,
                gameplay::sync_service_link_visuals,
                (
                    gameplay::sync_reactor_glow_visuals,
                    gameplay::sync_engine_flame_visuals,
                    gameplay::spawn_missing_effect_overlays,
                    gameplay::sync_turret_flash_visuals,
                    gameplay::sync_battery_pulse_visuals,
                    gameplay::sync_fabricator_dust_visuals,
                    gameplay::sync_hazard_effect_visuals,
                    gameplay::sync_ship_environment_effect_visuals,
                )
                    .chain()
                    .run_if(shaders_enabled),
                gameplay::sync_module_work_effect_visuals,
                gameplay::sync_eva_thruster_visuals,
                gameplay::sync_shipboard_player_visual,
                gameplay::sync_crew_name_labels,
                gameplay::integrate_player_ship_motion,
                gameplay::integrate_hostile_ship_motion,
                gameplay::handle_ship_collisions,
                gameplay::camera_follow_player_ship,
                gameplay::sync_backdrop_parallax,
                gameplay::draw_debug_overlay,
            )
                .chain()
                .run_if(netcode::session_presents_encounter),
            gameplay::update_gameplay_status_text.run_if(netcode::session_presents_encounter),
            gameplay::station_panel_button_system.run_if(netcode::session_presents_encounter),
        ),
    );
}

/// Spawns the shared camera used by UI, editors, and encounter presentation layers.
/// Spawns the primary 2D camera so interactive builds have a world and UI viewport.
fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, MainCamera));
}

/// Lets interactive builds quit quickly from the keyboard during development and testing.
/// Exits the app from interactive builds so local testing has a quick escape hatch.
fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

fn main() {
    run_client();
}

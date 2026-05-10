use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GameplayInfoPanelMode {
    #[default]
    Overview,
    FocusedModule,
    Alerts,
    StationConsole,
}

#[derive(Component)]
pub(crate) struct AbortEncounterButton;

#[derive(Component)]
pub(crate) struct BackToStationButton;

#[derive(Component)]
pub(crate) struct LaunchEncounterButton;

#[derive(Clone, Copy, Debug)]
pub(crate) enum StationPanelButtonAction {
    HelmThrottle { delta: f32 },
    HelmTurn { value: f32 },
    TurretAdjustAim { delta: f32 },
    TurretFireToggle,
    ReactorAdjustRate { delta: f32 },
    ReactorAdjustTurbine { delta: f32 },
    LogisticsToggleStorageIntake,
    LogisticsToggleAirlock,
    LogisticsToggleManipulator,
    LogisticsCycleManipulatorTarget { direction: i32 },
    LogisticsCycleResource,
    LogisticsToggleProcessor,
    ComputerToggleEnabled,
    ComputerCycleTemplate,
}

#[derive(Component)]
pub(crate) struct GameplayControlsText;

#[derive(Component)]
pub(crate) struct GameplayPanelTitleText;

#[derive(Component)]
pub(crate) struct GameplayPanelBodyText;

#[derive(Component)]
pub(crate) struct GameplayCompactStatusText;

#[derive(Component)]
pub(crate) struct GameplayTopBannerText;

#[derive(Component)]
pub(crate) struct GameplayInfoPanelRoot;

#[derive(Component)]
pub(crate) struct GameplayOverviewBarsPanel;

#[derive(Component)]
pub(crate) struct GameplayBlackoutOverlay;

#[derive(Component)]
pub(crate) struct GameplayControlsPanel;

#[derive(Component)]
pub(crate) struct ControlsHelpPanel;

#[derive(Component)]
pub(crate) struct GameplayStationPanel;

#[derive(Component)]
pub(crate) struct GameplayStationTitleText;

#[derive(Component, Clone, Copy)]
pub(crate) struct GameplayStationReadoutSlot {
    pub(crate) index: u8,
}

#[derive(Component)]
pub(crate) struct GameplayStationReadoutLabel;

#[derive(Component)]
pub(crate) struct GameplayStationReadoutValue;

#[derive(Component)]
pub(crate) struct GameplayStationReadoutBarTrack;

#[derive(Component)]
pub(crate) struct GameplayStationReadoutBarFill;

#[derive(Component)]
pub(crate) struct GameplayStationReadoutLight;

#[derive(Component)]
pub(crate) struct GameplayStationPanelButton {
    pub(crate) action: StationPanelButtonAction,
}

#[derive(Component)]
pub(crate) struct GameplayStationPanelButtonLabel {
    pub(crate) action: StationPanelButtonAction,
}

#[derive(Component)]
pub(crate) struct GameplayBarFill {
    pub(crate) kind: GameplayBarKind,
}

#[derive(Component)]
pub(crate) struct GameplayBarLabel {
    pub(crate) kind: GameplayBarKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GameplayBarKind {
    Hull,
    Power,
    Battery,
    Oxygen,
    Heat,
    Electrical,
}

#[derive(Component)]
pub(crate) struct PlayingCleanup;

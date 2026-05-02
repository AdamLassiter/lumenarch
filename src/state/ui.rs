use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct AbortEncounterButton;

#[derive(Component)]
pub(crate) struct BackToStationButton;

#[derive(Component)]
pub(crate) struct LaunchEncounterButton;

#[derive(Clone, Copy)]
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
pub(crate) struct GameplayStatusText;

#[derive(Component)]
pub(crate) struct GameplayInspectionText;

#[derive(Component)]
pub(crate) struct GameplayAlertsText;

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
pub(crate) struct GameplayStationPanel;

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

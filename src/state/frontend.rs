use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub(crate) enum FrontendMode {
    #[default]
    Menu,
    Session,
    DebugEnemyEditor,
}

#[derive(Resource, Default)]
pub(crate) struct DebugOverlayState {
    pub(crate) enabled: bool,
}

#[derive(Resource, Clone, Serialize, Deserialize)]
pub(crate) struct DockedState {
    pub(crate) station_title: String,
}

impl Default for DockedState {
    fn default() -> Self {
        Self {
            station_title: "Needle Rest".to_string(),
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct CampaignLoadState {
    pub(crate) hydrated: bool,
}

#[derive(Component)]
pub(crate) struct MenuRoot;

#[derive(Component)]
pub(crate) struct DockedRoot;

#[derive(Component)]
pub(crate) struct MainCamera;

#[derive(Component)]
pub(crate) struct JoinButton;

#[derive(Component)]
pub(crate) struct JoinButtonText;

#[derive(Component)]
pub(crate) struct DebugEnemyEditorButton;

#[derive(Component)]
pub(crate) struct RefitButton;

#[derive(Component)]
pub(crate) struct OpenSectorMapButton;

#[derive(Component)]
pub(crate) struct RepairShipButton;

#[derive(Component)]
pub(crate) struct StatusText;

#[derive(Component)]
pub(crate) struct HostAddressText;

#[derive(Component)]
pub(crate) struct DockedStatusText;

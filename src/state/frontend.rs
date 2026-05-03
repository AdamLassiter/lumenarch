use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::gameplay::components::PlayerSuit;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default, Serialize, Deserialize)]
pub(crate) enum FrontendMode {
    #[default]
    Lobby,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PlayerRole {
    #[default]
    Operator,
    Engineer,
    Surveyor,
    Runner,
}

impl PlayerRole {
    pub(crate) const ALL: [Self; 4] =
        [Self::Operator, Self::Engineer, Self::Surveyor, Self::Runner];

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Operator => "Operator",
            Self::Engineer => "Engineer",
            Self::Surveyor => "Surveyor",
            Self::Runner => "Runner",
        }
    }

    pub(crate) fn default_suit(self) -> PlayerSuit {
        match self {
            Self::Operator => PlayerSuit::Standard,
            Self::Engineer => PlayerSuit::Welder,
            Self::Surveyor => PlayerSuit::Radiation,
            Self::Runner => PlayerSuit::Eva,
        }
    }

    pub(crate) fn cycle(self, direction: i32) -> Self {
        let index = Self::ALL.iter().position(|role| *role == self).unwrap_or(0);
        let next = ((index as i32 + direction).rem_euclid(Self::ALL.len() as i32)) as usize;
        Self::ALL[next]
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LocalPlayerProfile {
    pub(crate) name: String,
    pub(crate) role: PlayerRole,
    pub(crate) color_index: u8,
}

impl Default for LocalPlayerProfile {
    fn default() -> Self {
        Self {
            name: "Crew".to_string(),
            role: PlayerRole::Operator,
            color_index: 0,
        }
    }
}

impl LocalPlayerProfile {
    pub(crate) const PALETTE: [[f32; 4]; 8] = [
        [0.82, 0.96, 0.62, 1.0],
        [0.62, 0.90, 0.96, 1.0],
        [0.96, 0.86, 0.62, 1.0],
        [0.94, 0.70, 0.82, 1.0],
        [0.98, 0.72, 0.44, 1.0],
        [0.70, 0.82, 0.98, 1.0],
        [0.72, 0.98, 0.78, 1.0],
        [0.96, 0.62, 0.62, 1.0],
    ];

    pub(crate) fn starting_suit(&self) -> PlayerSuit {
        self.role.default_suit()
    }

    pub(crate) fn color_rgba(&self) -> [f32; 4] {
        Self::PALETTE[self.color_index as usize % Self::PALETTE.len()]
    }

    pub(crate) fn color(&self) -> Color {
        let [r, g, b, a] = self.color_rgba();
        Color::srgba(r, g, b, a)
    }

    pub(crate) fn cycle_color(&mut self, direction: i32) {
        let palette_len = Self::PALETTE.len() as i32;
        self.color_index = ((self.color_index as i32 + direction).rem_euclid(palette_len)) as u8;
    }
}

#[derive(Resource, Default)]
pub(crate) struct LobbyProfileEditState {
    pub(crate) editing_name: bool,
}

#[derive(Component)]
pub(crate) struct LobbyRoot;

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
pub(crate) struct LobbyNameText;

#[derive(Component)]
pub(crate) struct LobbyRoleText;

#[derive(Component)]
pub(crate) struct LobbyColorText;

#[derive(Component)]
pub(crate) struct LobbyCycleRoleButton;

#[derive(Component)]
pub(crate) struct LobbyCycleColorButton;

#[derive(Component)]
pub(crate) struct DockedStatusText;

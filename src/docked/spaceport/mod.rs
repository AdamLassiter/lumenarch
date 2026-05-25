use std::{collections::BTreeMap, f32::consts::FRAC_PI_2, path::Path};

use bevy::prelude::*;
use bevy_yarnspinner::{
    events::{DialogueCompleted, PresentLine, PresentOptions},
    prelude::{DialogueRunner, OptionId, YarnProject},
};
use ggrs::PlayerHandle;

use super::board::{DockedBoardState, open_docked_board};
use crate::{
    TILE_SIZE,
    balance::BalanceConfig,
    helpers::{
        FixedVec2,
        Fx,
        angle_from_vector,
        damp_vec2,
        sprite_path_for_foundation_connections,
        sprite_path_for_kind,
    },
    netcode::{LocalPlayerHandle, PendingLocalMetaCommand, PendingMetaCommand, RollbackMetaOp},
    ship::{ModuleKind, ShipDefinition, ShipFoundationKind, ShipFoundationTile},
    state::{
        DockedState,
        DockedSurface,
        EditorMode,
        EditorSessionState,
        EditorShip,
        LocalPlayerProfile,
        Progression,
        SectorState,
    },
    station_editor,
    stations::{self, StationCatalogResource, StationDefinition, StationNpcServiceAction},
};

mod dialogue;
mod geometry;
mod scene;
#[cfg(test)]
mod tests;
mod types;

pub(crate) use dialogue::{
    docked_dialogue_button_system,
    docked_keyboard_interaction_system,
    handle_docked_yarn_dialogue_completed,
    handle_docked_yarn_line,
    handle_docked_yarn_options,
    sync_docked_dialogue_overlay,
    sync_docked_yarn_runner,
};
use geometry::*;
pub(crate) use geometry::{
    cleanup_docked_spaceport_scene,
    docked_spaceport_scene_missing,
    docked_spaceport_scene_present,
};
pub(crate) use scene::{move_docked_local_avatar, spawn_docked_spaceport_scene};
use types::*;
pub(crate) use types::{DockedAvatarMemory, DockedDialogueState};

use std::{collections::BTreeSet, ops::DerefMut};

use bevy::{ecs::relationship::Relationship, log, prelude::*};
use bevy_ggrs::PlayerInputs;

use super::super::{
    super::state::{AbortEncounterButton, MainCamera, PlayingCleanup},
    components::{
        AirlockCommandState,
        AngularVelocity,
        CarriedItemKind,
        CarriedResource,
        CrewNameBackdrop,
        CrewNameLabel,
        CurrentStation,
        DestroyedModule,
        EquippedSuit,
        InternalPosition,
        LinearVelocity,
        LooseCargo,
        ManipulatorCommandState,
        MissionState,
        ObservedLocalPlayerMarker,
        PlayerFieldState,
        PlayerHandleComponent,
        PlayerMotionState,
        PlayerReferenceFrame,
        PlayerShip,
        PlayerShipAssignment,
        PlayerSuit,
        ProcessorCommandState,
        ReactorCommandState,
        ResourceKind,
        RuntimeArchComputer,
        RuntimeShipModule,
        ShipArchCommandState,
        ShipAtmosphereState,
        ShipControlMode,
        ShipControlState,
        ShipInertiaField,
        ShipMovementModel,
        ShipPowerModel,
        ShipPowerState,
        ShipRoot,
        ShipWeaponState,
        ShipboardControlState,
        ShipboardMarker,
        ShipboardPlayer,
        SimPosition,
        SimRotation,
        StationFamily,
        StationFocusMode,
        StorageCommandState,
        StorageModule,
        TurretCommandState,
    },
    helpers::{
        FixedVec2,
        Fx,
        angle_from_vector,
        clamp_position_to_arena,
        damp_scalar,
        damp_vec2,
        facing_vector,
        fx_from_time_delta,
        render_translation,
        update_ship_power_state,
        wrap_radians,
    },
};
use crate::{
    balance::BalanceConfig,
    netcode::{self, LumenGgrsConfig},
    ship::ModuleKind,
};

mod camera;
mod helpers;
mod player;
mod ship;

pub(crate) use camera::{
    camera_follow_player_ship,
    return_button_system,
    return_keyboard_shortcut,
};
pub(crate) use player::{
    exit_focused_station,
    handle_player_cargo_interaction,
    move_shipboard_player,
    sync_crew_name_labels,
    sync_player_reference_frame_parenting,
    sync_shipboard_player_visual,
    toggle_shipboard_control_mode,
    update_current_station,
    update_player_reference_frame,
};
pub(crate) use ship::{
    apply_player_ship_controls,
    focus_station,
    integrate_player_ship_motion,
    update_station_command_input,
};

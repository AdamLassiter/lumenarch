use std::{collections::BTreeSet, ops::DerefMut};

use bevy::{ecs::relationship::Relationship, log, prelude::*};
use bevy_ggrs::PlayerInputs;

use super::super::{
    super::state::{AbortEncounterButton, MainCamera, PlayingCleanup},
    components::{
        AirlockCommandState,
        AngularVelocity,
        CarriedResource,
        CurrentStation,
        DestroyedModule,
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

const INTERIOR_CAMERA_SCALE: f32 = 0.58;
const EXTERIOR_CAMERA_SCALE: f32 = 1.0;
const PLAYER_WALK_ACCELERATION: f32 = 260.0;
const PLAYER_WALK_MAX_SPEED: f32 = 120.0;
const PLAYER_WALK_DAMPING: f32 = 8.0;
const PLAYER_EVA_ACCELERATION: f32 = 180.0;
const PLAYER_EVA_MAX_SPEED: f32 = 140.0;
const PLAYER_EVA_DAMPING: f32 = 1.6;
const PLAYER_INTERACT_RADIUS: f32 = 28.0;
const PLAYER_CARGO_PICKUP_RADIUS: f32 = 20.0;
const PLAYER_COLLISION_RADIUS: f32 = 7.0;

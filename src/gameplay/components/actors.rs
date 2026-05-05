use bevy::{
    ecs::entity::{EntityMapper, MapEntities},
    prelude::*,
};
use ggrs::PlayerHandle;
use serde::{Deserialize, Serialize};

use super::{
    super::helpers::{FixedVec2, Fx},
    logistics::ResourceKind,
};
use crate::{
    balance::PlayerBalanceConfig,
    ship::ModuleKind,
    state::PlayerRole,
    stations::FactionId,
};

#[derive(Component)]
pub(crate) struct PlayerShip;

#[derive(Component)]
pub(crate) struct ShipRoot;

#[derive(Component)]
pub(crate) struct HostileShip;

#[derive(Component, Clone)]
pub(crate) struct HostileShipAi {
    pub(crate) preferred_range: Fx,
    pub(crate) aggression: Fx,
    pub(crate) salvage_reward: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CaptainProfile {
    pub(crate) name: String,
    pub(crate) title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EncounterCommsScript {
    pub(crate) intro: String,
    pub(crate) outro: String,
}

#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ShipEncounterIdentity {
    pub(crate) faction_id: FactionId,
    pub(crate) ship_name: String,
    pub(crate) captain: CaptainProfile,
    pub(crate) comms: EncounterCommsScript,
    pub(crate) crewed: bool,
}

#[derive(Component)]
pub(crate) struct ShipboardPlayer;

#[derive(Component, Clone, Copy)]
pub(crate) struct PlayerHandleComponent {
    pub(crate) handle: PlayerHandle,
}

#[derive(Component)]
pub(crate) struct ObservedLocalPlayerMarker;

#[derive(Component)]
pub(crate) struct ShipboardMarker;

#[derive(Component, Clone)]
pub(crate) struct ShipInertiaField {
    pub(crate) radius: Fx,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct CrewNameLabel {
    pub(crate) player_entity: Entity,
}

#[derive(Component, Clone, Copy)]
pub(crate) struct CrewNameBackdrop {
    pub(crate) player_entity: Entity,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerShipAssignment {
    pub(crate) _ship_entity: Entity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlayerReferenceFrame {
    World,
    Ship(Entity),
}

#[derive(Component, Clone)]
pub(crate) struct PlayerMotionState {
    pub(crate) frame: PlayerReferenceFrame,
    pub(crate) world_position: FixedVec2,
    pub(crate) world_velocity: FixedVec2,
    pub(crate) local_position: FixedVec2,
    pub(crate) local_velocity: FixedVec2,
    pub(crate) facing_radians: Fx,
}

#[derive(Component, Default, Clone)]
pub(crate) struct CarriedResource {
    pub(crate) kind: Option<CarriedItemKind>,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PlayerSuit {
    Standard,
    Radiation,
    Welder,
    Eva,
}

impl PlayerSuit {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "Standard Suit",
            Self::Radiation => "Radiation Suit",
            Self::Welder => "Welder Suit",
            Self::Eva => "EVA Suit",
        }
    }

    pub(crate) fn heat_multiplier(self, balance: &PlayerBalanceConfig) -> Fx {
        match self {
            Self::Standard => Fx::from_num(balance.standard_heat_multiplier),
            Self::Radiation => Fx::from_num(balance.radiation_heat_multiplier),
            Self::Welder => Fx::from_num(balance.welder_heat_multiplier),
            Self::Eva => Fx::from_num(balance.eva_heat_multiplier),
        }
    }

    pub(crate) fn electrical_multiplier(self, balance: &PlayerBalanceConfig) -> Fx {
        match self {
            Self::Standard => Fx::from_num(balance.standard_electrical_multiplier),
            Self::Radiation => Fx::from_num(balance.radiation_electrical_multiplier),
            Self::Welder => Fx::from_num(balance.welder_electrical_multiplier),
            Self::Eva => Fx::from_num(balance.eva_electrical_multiplier),
        }
    }

    pub(crate) fn oxygen_warning_threshold(self, balance: &PlayerBalanceConfig) -> Fx {
        match self {
            Self::Standard => Fx::from_num(balance.standard_oxygen_warning_threshold),
            Self::Radiation => Fx::from_num(balance.radiation_oxygen_warning_threshold),
            Self::Welder => Fx::from_num(balance.welder_oxygen_warning_threshold),
            Self::Eva => Fx::from_num(balance.eva_oxygen_warning_threshold),
        }
    }

    pub(crate) fn oxygen_critical_threshold(self, balance: &PlayerBalanceConfig) -> Fx {
        match self {
            Self::Standard => Fx::from_num(balance.standard_oxygen_critical_threshold),
            Self::Radiation => Fx::from_num(balance.radiation_oxygen_critical_threshold),
            Self::Welder => Fx::from_num(balance.welder_oxygen_critical_threshold),
            Self::Eva => Fx::from_num(balance.eva_oxygen_critical_threshold),
        }
    }

    pub(crate) fn eva_speed_multiplier(self, balance: &PlayerBalanceConfig) -> Fx {
        match self {
            Self::Standard => Fx::from_num(balance.standard_eva_speed_multiplier),
            Self::Radiation => Fx::from_num(balance.radiation_eva_speed_multiplier),
            Self::Welder => Fx::from_num(balance.welder_eva_speed_multiplier),
            Self::Eva => Fx::from_num(balance.eva_eva_speed_multiplier),
        }
    }

    pub(crate) fn color(self) -> Color {
        match self {
            Self::Standard => Color::srgb(0.82, 0.96, 0.62),
            Self::Radiation => Color::srgb(0.54, 0.96, 0.54),
            Self::Welder => Color::srgb(0.98, 0.68, 0.26),
            Self::Eva => Color::srgb(0.56, 0.82, 0.98),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub(crate) struct EquippedSuit {
    pub(crate) suit: PlayerSuit,
}

impl Default for EquippedSuit {
    fn default() -> Self {
        Self {
            suit: PlayerSuit::Standard,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CarriedItemKind {
    Resource(ResourceKind),
    Suit(PlayerSuit),
    ExtractedComponent {
        kind: ModuleKind,
        variant: crate::ship::ModuleVariant,
    },
}

impl CarriedItemKind {
    pub(crate) fn label(self) -> String {
        match self {
            Self::Resource(kind) => match kind {
                ResourceKind::RawSalvage => "raw salvage".to_string(),
                ResourceKind::RepairCharge => "repair charge".to_string(),
                ResourceKind::Fuel => "fuel".to_string(),
                ResourceKind::Ammunition => "ammunition".to_string(),
            },
            Self::Suit(suit) => suit.as_str().to_string(),
            Self::ExtractedComponent { kind, variant } => {
                format!("{} {} component", variant.display_name(), kind.as_str())
            }
        }
    }

    pub(crate) fn color(self) -> Color {
        match self {
            Self::Resource(kind) => match kind {
                ResourceKind::RawSalvage => Color::srgb(0.90, 0.78, 0.34),
                ResourceKind::RepairCharge => Color::srgb(0.38, 0.88, 0.98),
                ResourceKind::Fuel => Color::srgb(0.98, 0.52, 0.22),
                ResourceKind::Ammunition => Color::srgb(0.86, 0.86, 0.90),
            },
            Self::Suit(suit) => suit.color(),
            Self::ExtractedComponent { .. } => Color::srgb(0.94, 0.76, 0.54),
        }
    }
}

#[derive(Component, Clone, Serialize, Deserialize)]
pub(crate) struct PlayerIdentity {
    pub(crate) name: String,
    pub(crate) role: PlayerRole,
    pub(crate) color_index: u8,
}

impl PlayerIdentity {
    pub(crate) fn color(&self) -> Color {
        let [r, g, b, a] = crate::state::LocalPlayerProfile::PALETTE
            [self.color_index as usize % crate::state::LocalPlayerProfile::PALETTE.len()];
        Color::srgba(r, g, b, a)
    }
}

#[derive(Clone)]
pub(crate) struct ShipInteriorNode {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component, Default, Clone)]
pub(crate) struct ShipInteriorMap {
    pub(crate) walkable_nodes: Vec<ShipInteriorNode>,
}

#[derive(Clone)]
pub(crate) struct ShipAtmosphereTile {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
    pub(crate) oxygen: Fx,
    pub(crate) exterior_edges: u8,
}

#[derive(Component, Default, Clone)]
pub(crate) struct ShipAtmosphereState {
    pub(crate) tiles: Vec<ShipAtmosphereTile>,
    pub(crate) average_oxygen: Fx,
    pub(crate) minimum_oxygen: Fx,
    pub(crate) venting_tiles: u32,
    pub(crate) decompression_reported: bool,
    pub(crate) decompression_signature: u64,
    pub(crate) decompression_vectors: Vec<FixedVec2>,
}

#[derive(Component, Clone)]
pub(crate) struct InternalPosition {
    pub(crate) grid_x: i32,
    pub(crate) grid_y: i32,
    pub(crate) local_position: FixedVec2,
}

#[derive(Component, Clone)]
pub(crate) struct CurrentStation {
    pub(crate) module_id: u64,
    pub(crate) kind: ModuleKind,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StationFamily {
    Cockpit,
    Turret,
    Reactor,
    Storage,
    Manipulator,
    Processor,
    Computer,
    Shield,
    Detector,
    Drone,
    Memory,
}

impl StationFamily {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Cockpit => "Cockpit",
            Self::Turret => "Turret",
            Self::Reactor => "Reactor",
            Self::Storage => "Storage",
            Self::Manipulator => "Manipulator",
            Self::Processor => "Processor",
            Self::Computer => "Computer",
            Self::Shield => "Shield",
            Self::Detector => "Detector",
            Self::Drone => "Drone Station",
            Self::Memory => "Memory Bank",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShipControlMode {
    Interior,
    Cockpit,
    Turret,
    Reactor,
    Logistics,
    Computer,
}

impl ShipControlMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Interior => "Interior",
            Self::Cockpit => "Cockpit",
            Self::Turret => "Turret",
            Self::Reactor => "Reactor",
            Self::Logistics => "Logistics",
            Self::Computer => "Computer",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StationFocusMode {
    Internal,
    Focused,
}

#[derive(Component, Clone)]
pub(crate) struct ShipboardControlState {
    pub(crate) mode: ShipControlMode,
    pub(crate) focus_mode: StationFocusMode,
    pub(crate) focused_entity: Option<Entity>,
    pub(crate) focused_module_id: Option<u64>,
    pub(crate) focused_kind: Option<ModuleKind>,
    pub(crate) focused_family: Option<StationFamily>,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerFieldState {
    pub(crate) local_heat: Fx,
    pub(crate) local_electrical: Fx,
    pub(crate) local_oxygen: Fx,
    pub(crate) heat_danger: bool,
    pub(crate) electrical_danger: bool,
    pub(crate) oxygen_warning: bool,
    pub(crate) oxygen_critical: bool,
}

#[derive(Component, Clone)]
pub(crate) struct PlayerConditionState {
    pub(crate) health: i32,
    pub(crate) max_health: i32,
    pub(crate) heat_buildup: Fx,
    pub(crate) electrical_buildup: Fx,
    pub(crate) blackout: Fx,
    pub(crate) stun_remaining: Fx,
    pub(crate) heat_damage_progress: Fx,
    pub(crate) blackout_damage_progress: Fx,
}

impl Default for PlayerConditionState {
    fn default() -> Self {
        Self {
            health: 10,
            max_health: 10,
            heat_buildup: Fx::from_num(0),
            electrical_buildup: Fx::from_num(0),
            blackout: Fx::from_num(0),
            stun_remaining: Fx::from_num(0),
            heat_damage_progress: Fx::from_num(0),
            blackout_damage_progress: Fx::from_num(0),
        }
    }
}

impl PlayerConditionState {
    pub(crate) fn control_disabled(&self) -> bool {
        self.health <= 0 || self.stun_remaining > Fx::from_num(0)
    }
}

impl MapEntities for PlayerShipAssignment {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self._ship_entity = entity_mapper.get_mapped(self._ship_entity);
    }
}

impl MapEntities for PlayerReferenceFrame {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Self::Ship(entity) = self {
            *entity = entity_mapper.get_mapped(*entity);
        }
    }
}

impl MapEntities for PlayerMotionState {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.frame.map_entities(entity_mapper);
    }
}

impl MapEntities for ShipboardControlState {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        if let Some(entity) = self.focused_entity {
            self.focused_entity = Some(entity_mapper.get_mapped(entity));
        }
    }
}

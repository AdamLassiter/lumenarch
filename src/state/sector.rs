use std::{fs, path::Path};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const DEFAULT_SECTOR_LAYOUT_PATH: &str = "saves/sector_layout.json";

use crate::ship::{ModuleKind, ModuleSpec, ModuleVariant};

const fn default_star_density() -> u32 {
    96
}

const fn default_dust_density() -> u32 {
    40
}

const fn default_parallax_strength() -> f32 {
    0.35
}

const fn default_haze_tint() -> [f32; 3] {
    [0.16, 0.22, 0.30]
}

const fn default_galaxy_tint() -> [f32; 3] {
    [0.72, 0.84, 1.0]
}

const fn default_galaxy_arc_strength() -> f32 {
    0.55
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StoredComponentStack {
    pub(crate) kind: ModuleKind,
    pub(crate) variant: ModuleVariant,
    pub(crate) ready: u32,
    pub(crate) damaged: u32,
}

impl StoredComponentStack {
    pub(crate) fn label(&self) -> String {
        format!("{} {}", self.variant.display_name(), self.kind.as_str())
    }

    pub(crate) fn repair_cost(&self) -> u32 {
        ModuleSpec::for_module(self.kind, self.variant)
            .placement_cost
            .max(1)
    }
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Progression {
    pub(crate) scrap: u32,
    pub(crate) hull_wear: u32,
    pub(crate) jump_count: u32,
    #[serde(default)]
    pub(crate) stored_components: Vec<StoredComponentStack>,
    #[serde(default)]
    pub(crate) known_station_ids: Vec<String>,
    #[serde(default)]
    pub(crate) unlocked_contact_ids: Vec<String>,
    #[serde(default)]
    pub(crate) unlocked_lore_ids: Vec<String>,
    #[serde(default)]
    pub(crate) completed_contract_ids: Vec<String>,
    #[serde(default)]
    pub(crate) active_contract_id: Option<String>,
}

impl Default for Progression {
    fn default() -> Self {
        Self {
            scrap: 100,
            hull_wear: 0,
            jump_count: 0,
            stored_components: starter_component_inventory(),
            known_station_ids: vec!["needle_rest".to_string()],
            unlocked_contact_ids: vec![
                "ivra_quell".to_string(),
                "sable_ren".to_string(),
                "peregrine_cho".to_string(),
            ],
            unlocked_lore_ids: vec![
                "needle_rest_foundation".to_string(),
                "null_swarms_brief".to_string(),
            ],
            completed_contract_ids: Vec::new(),
            active_contract_id: None,
        }
    }
}

impl Progression {
    #[allow(dead_code)]
    pub(crate) fn knows_station(&self, station_id: &str) -> bool {
        self.known_station_ids
            .iter()
            .any(|known| known == station_id)
    }

    #[allow(dead_code)]
    pub(crate) fn contact_unlocked(&self, contact_id: &str) -> bool {
        self.unlocked_contact_ids
            .iter()
            .any(|unlocked| unlocked == contact_id)
    }

    pub(crate) fn lore_unlocked(&self, lore_id: &str) -> bool {
        self.unlocked_lore_ids
            .iter()
            .any(|unlocked| unlocked == lore_id)
    }

    pub(crate) fn contract_completed(&self, contract_id: &str) -> bool {
        self.completed_contract_ids
            .iter()
            .any(|completed| completed == contract_id)
    }

    pub(crate) fn unlock_station(&mut self, station_id: impl Into<String>) {
        let station_id = station_id.into();
        if !self
            .known_station_ids
            .iter()
            .any(|known| known == &station_id)
        {
            self.known_station_ids.push(station_id);
        }
    }

    pub(crate) fn unlock_contact(&mut self, contact_id: impl Into<String>) {
        let contact_id = contact_id.into();
        if !self
            .unlocked_contact_ids
            .iter()
            .any(|unlocked| unlocked == &contact_id)
        {
            self.unlocked_contact_ids.push(contact_id);
        }
    }

    pub(crate) fn unlock_lore(&mut self, lore_id: impl Into<String>) {
        let lore_id = lore_id.into();
        if !self
            .unlocked_lore_ids
            .iter()
            .any(|unlocked| unlocked == &lore_id)
        {
            self.unlocked_lore_ids.push(lore_id);
        }
    }

    pub(crate) fn complete_contract(&mut self, contract_id: impl Into<String>) {
        let contract_id = contract_id.into();
        if !self
            .completed_contract_ids
            .iter()
            .any(|completed| completed == &contract_id)
        {
            self.completed_contract_ids.push(contract_id);
        }
    }

    pub(crate) fn ready_count(&self, kind: ModuleKind, variant: ModuleVariant) -> u32 {
        self.stored_components
            .iter()
            .find(|entry| entry.kind == kind && entry.variant == variant)
            .map_or(0, |entry| entry.ready)
    }

    pub(crate) fn damaged_count(&self, kind: ModuleKind, variant: ModuleVariant) -> u32 {
        self.stored_components
            .iter()
            .find(|entry| entry.kind == kind && entry.variant == variant)
            .map_or(0, |entry| entry.damaged)
    }

    pub(crate) fn add_ready_component(
        &mut self,
        kind: ModuleKind,
        variant: ModuleVariant,
        amount: u32,
    ) {
        if amount == 0 {
            return;
        }
        let entry = self.component_entry_mut(kind, variant);
        entry.ready += amount;
    }

    pub(crate) fn add_damaged_component(
        &mut self,
        kind: ModuleKind,
        variant: ModuleVariant,
        amount: u32,
    ) {
        if amount == 0 {
            return;
        }
        let entry = self.component_entry_mut(kind, variant);
        entry.damaged += amount;
    }

    pub(crate) fn try_consume_ready_component(
        &mut self,
        kind: ModuleKind,
        variant: ModuleVariant,
    ) -> bool {
        let entry = self.component_entry_mut(kind, variant);
        if entry.ready == 0 {
            return false;
        }
        entry.ready -= 1;
        true
    }

    pub(crate) fn try_repair_component(
        &mut self,
        kind: ModuleKind,
        variant: ModuleVariant,
    ) -> bool {
        let repair_cost = ModuleSpec::for_module(kind, variant).placement_cost.max(1);
        if self.scrap < repair_cost {
            return false;
        }
        let damaged_count = self.damaged_count(kind, variant);
        if damaged_count == 0 {
            return false;
        }
        self.scrap -= repair_cost;
        let entry = self.component_entry_mut(kind, variant);
        entry.damaged -= 1;
        entry.ready += 1;
        true
    }

    pub(crate) fn damaged_components(&self) -> impl Iterator<Item = &StoredComponentStack> {
        self.stored_components
            .iter()
            .filter(|entry| entry.damaged > 0)
    }

    fn component_entry_mut(
        &mut self,
        kind: ModuleKind,
        variant: ModuleVariant,
    ) -> &mut StoredComponentStack {
        if let Some(index) = self
            .stored_components
            .iter()
            .position(|entry| entry.kind == kind && entry.variant == variant)
        {
            return &mut self.stored_components[index];
        }
        self.stored_components.push(StoredComponentStack {
            kind,
            variant,
            ready: 0,
            damaged: 0,
        });
        self.stored_components.last_mut().unwrap()
    }
}

fn starter_component_inventory() -> Vec<StoredComponentStack> {
    vec![
        StoredComponentStack {
            kind: ModuleKind::Hull,
            variant: ModuleVariant::Standard,
            ready: 10,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Interior,
            variant: ModuleVariant::Standard,
            ready: 8,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Engine,
            variant: ModuleVariant::Standard,
            ready: 2,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Cargo,
            variant: ModuleVariant::GeneralCargo,
            ready: 2,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Battery,
            variant: ModuleVariant::BatteryCell,
            ready: 2,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Computer,
            variant: ModuleVariant::Standard,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Detector,
            variant: ModuleVariant::LifePulse,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Detector,
            variant: ModuleVariant::ShipPing,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Detector,
            variant: ModuleVariant::DamageAlarm,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Processor,
            variant: ModuleVariant::FabricatorSlow,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Airlock,
            variant: ModuleVariant::Standard,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Airlock,
            variant: ModuleVariant::DroneBay,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Turret,
            variant: ModuleVariant::LaserTurret,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Shield,
            variant: ModuleVariant::RadialShield,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Detector,
            variant: ModuleVariant::PowerMonitor,
            ready: 1,
            damaged: 0,
        },
        StoredComponentStack {
            kind: ModuleKind::Reactor,
            variant: ModuleVariant::Fission,
            ready: 1,
            damaged: 0,
        },
    ]
}

#[derive(Resource, Clone, Copy)]
pub(crate) struct SectorMapViewState {
    pub(crate) offset: Vec2,
    pub(crate) zoom: f32,
}

impl Default for SectorMapViewState {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub(crate) struct SectorMapPanState {
    pub(crate) last_cursor: Option<Vec2>,
}

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct LastMissionReport {
    pub(crate) headline: Option<String>,
    pub(crate) detail: Option<String>,
    pub(crate) scrap_awarded: u32,
    pub(crate) total_scrap: u32,
    pub(crate) hottest_module: Option<String>,
    pub(crate) first_disabled_module: Option<String>,
    pub(crate) repairs_performed: u32,
    pub(crate) stabilizations_performed: u32,
    pub(crate) automation_used: bool,
    pub(crate) automation_triggers: u32,
    pub(crate) redesign_hints: Vec<String>,
    pub(crate) recovered_raw_salvage: u32,
    pub(crate) processed_repair_charge: u32,
    pub(crate) consumed_repair_charge: u32,
    pub(crate) transfer_count: u32,
    pub(crate) processor_cycles: u32,
    pub(crate) logistics_bottleneck: Option<String>,
    pub(crate) logistics_automation_used: bool,
    pub(crate) arch_primary_program: Option<String>,
    pub(crate) arch_invalid_executions: u32,
    pub(crate) arch_recent_writes: Vec<String>,
    pub(crate) node_name: Option<String>,
    pub(crate) node_kind: Option<String>,
    pub(crate) travel_outcome: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SectorNodeStatus {
    #[default]
    Fresh,
    Completed,
    Exhausted,
    Failed,
}

impl SectorNodeStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "Fresh",
            Self::Completed => "Completed",
            Self::Exhausted => "Exhausted",
            Self::Failed => "Failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SectorNodeKind {
    HubStation,
    TestRange,
    SalvageField,
    HostileHold,
    UnstableDerelict,
}

impl SectorNodeKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::HubStation => "Hub Station",
            Self::TestRange => "Test Range",
            Self::SalvageField => "Salvage Field",
            Self::HostileHold => "Hostile Hold",
            Self::UnstableDerelict => "Unstable Derelict",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EncounterBackdrop {
    #[serde(default)]
    pub(crate) seed: u64,
    #[serde(default = "default_star_density")]
    pub(crate) star_density: u32,
    #[serde(default = "default_dust_density")]
    pub(crate) dust_density: u32,
    #[serde(default = "default_parallax_strength")]
    pub(crate) parallax_strength: f32,
    #[serde(default = "default_haze_tint")]
    pub(crate) haze_tint: [f32; 3],
    #[serde(default = "default_galaxy_tint")]
    pub(crate) galaxy_tint: [f32; 3],
    #[serde(default = "default_galaxy_arc_strength")]
    pub(crate) galaxy_arc_strength: f32,
}

impl Default for EncounterBackdrop {
    fn default() -> Self {
        Self {
            seed: 0,
            star_density: default_star_density(),
            dust_density: default_dust_density(),
            parallax_strength: default_parallax_strength(),
            haze_tint: default_haze_tint(),
            galaxy_tint: default_galaxy_tint(),
            galaxy_arc_strength: default_galaxy_arc_strength(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EncounterSpec {
    #[serde(default)]
    pub(crate) enemy_ship_ids: Vec<String>,
    pub(crate) hostile_count: u32,
    pub(crate) salvage_value: u32,
    pub(crate) ambient_heat_pressure: i32,
    pub(crate) ambient_electrical_pressure: i32,
    pub(crate) reward_multiplier: u32,
    pub(crate) arena_variant: String,
    #[serde(default)]
    pub(crate) backdrop: EncounterBackdrop,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TravelOutcome {
    pub(crate) node_id: u32,
    pub(crate) success: bool,
    pub(crate) failed: bool,
    pub(crate) scrap_awarded: u32,
    pub(crate) hull_wear_delta: u32,
    pub(crate) exhausted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SectorNode {
    pub(crate) id: u32,
    pub(crate) label: String,
    pub(crate) kind: SectorNodeKind,
    #[serde(default)]
    pub(crate) station_id: Option<String>,
    pub(crate) risk_tier: u8,
    pub(crate) reward_hint: String,
    pub(crate) neighbors: Vec<u32>,
    pub(crate) status: SectorNodeStatus,
    pub(crate) position: [f32; 2],
    pub(crate) encounter: EncounterSpec,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SectorLayoutConfig {
    seed: u64,
    current_node_id: u32,
    selected_node_id: Option<u32>,
    nodes: Vec<SectorNode>,
}

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SectorState {
    pub(crate) seed: u64,
    pub(crate) current_node_id: u32,
    pub(crate) selected_node_id: Option<u32>,
    pub(crate) active_encounter_node_id: Option<u32>,
    pub(crate) nodes: Vec<SectorNode>,
}

impl Default for SectorState {
    fn default() -> Self {
        Self::from_seed(0x10_4E6)
    }
}

impl SectorState {
    pub(crate) fn from_seed(seed: u64) -> Self {
        let layout = load_or_create_sector_layout(seed).unwrap_or_else(|error| {
            eprintln!("sector: failed to load sector layout, using built-in defaults: {error}");
            default_sector_layout(seed)
        });

        Self {
            seed: layout.seed,
            current_node_id: layout.current_node_id,
            selected_node_id: layout.selected_node_id,
            active_encounter_node_id: None,
            nodes: layout.nodes,
        }
    }

    pub(crate) fn ensure_latest_layout(&mut self) {
        let layout = load_or_create_sector_layout(self.seed).unwrap_or_else(|error| {
            eprintln!("sector: failed to refresh sector layout, using built-in defaults: {error}");
            default_sector_layout(self.seed)
        });

        let mut merged_nodes = Vec::with_capacity(layout.nodes.len());
        for layout_node in layout.nodes {
            if let Some(existing) = self.nodes.iter().find(|node| node.id == layout_node.id) {
                let mut node = layout_node;
                node.status = existing.status;
                merged_nodes.push(node);
            } else {
                merged_nodes.push(layout_node);
            }
        }

        self.nodes = merged_nodes;
        self.seed = layout.seed;
        if self.node(self.current_node_id).is_none() {
            self.current_node_id = layout.current_node_id;
        }
        if self
            .selected_node_id
            .is_none_or(|selected| self.node(selected).is_none())
        {
            self.selected_node_id = layout
                .selected_node_id
                .or_else(|| self.available_neighbors().first().map(|node| node.id));
        }
    }

    pub(crate) fn node(&self, node_id: u32) -> Option<&SectorNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub(crate) fn node_mut(&mut self, node_id: u32) -> Option<&mut SectorNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub(crate) fn current_node(&self) -> Option<&SectorNode> {
        self.node(self.current_node_id)
    }

    pub(crate) fn selected_node(&self) -> Option<&SectorNode> {
        self.selected_node_id.and_then(|node_id| self.node(node_id))
    }

    pub(crate) fn active_node(&self) -> Option<&SectorNode> {
        self.active_encounter_node_id
            .and_then(|node_id| self.node(node_id))
    }

    pub(crate) fn is_reachable(&self, node_id: u32) -> bool {
        self.current_node()
            .map(|node| node.neighbors.contains(&node_id))
            .unwrap_or(false)
    }

    pub(crate) fn available_neighbors(&self) -> Vec<&SectorNode> {
        let Some(current) = self.current_node() else {
            return Vec::new();
        };

        current
            .neighbors
            .iter()
            .filter_map(|neighbor_id| self.node(*neighbor_id))
            .collect()
    }
}

fn load_or_create_sector_layout(seed: u64) -> Result<SectorLayoutConfig, String> {
    let path = Path::new(DEFAULT_SECTOR_LAYOUT_PATH);
    if path.exists() {
        let encoded = fs::read_to_string(path)
            .map_err(|error| format!("failed to read sector layout {}: {error}", path.display()))?;
        let mut config: SectorLayoutConfig = serde_json::from_str(&encoded).map_err(|error| {
            format!("failed to decode sector layout {}: {error}", path.display())
        })?;
        backfill_default_station_reference(&mut config);
        return Ok(config);
    }

    let config = default_sector_layout(seed);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create sector layout directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("failed to encode sector layout {}: {error}", path.display()))?;
    fs::write(path, encoded)
        .map_err(|error| format!("failed to write sector layout {}: {error}", path.display()))?;
    Ok(config)
}

fn backfill_default_station_reference(config: &mut SectorLayoutConfig) {
    if config
        .nodes
        .iter()
        .any(|node| node.station_id.as_deref().is_some())
    {
        return;
    }
    if let Some(node) = config
        .nodes
        .iter_mut()
        .find(|node| node.kind == SectorNodeKind::HubStation)
    {
        node.station_id = Some("needle_rest".to_string());
    }
}

fn default_sector_layout(seed: u64) -> SectorLayoutConfig {
    SectorLayoutConfig {
        seed,
        current_node_id: 0,
        selected_node_id: Some(1),
        nodes: vec![
            SectorNode {
                id: 0,
                label: "Needle Rest".to_string(),
                kind: SectorNodeKind::HubStation,
                station_id: Some("needle_rest".to_string()),
                risk_tier: 0,
                reward_hint: "Safe dock, refit, relaunch".to_string(),
                neighbors: vec![1, 2, 3, 6],
                status: SectorNodeStatus::Fresh,
                position: [0.0, 0.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: Vec::new(),
                    hostile_count: 0,
                    salvage_value: 0,
                    ambient_heat_pressure: 0,
                    ambient_electrical_pressure: 0,
                    reward_multiplier: 1,
                    arena_variant: "station".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x1000,
                        star_density: 72,
                        dust_density: 18,
                        parallax_strength: 0.18,
                        haze_tint: [0.12, 0.17, 0.24],
                        galaxy_tint: [0.58, 0.72, 0.94],
                        galaxy_arc_strength: 0.22,
                    },
                },
            },
            SectorNode {
                id: 1,
                label: "Latchline Debris".to_string(),
                kind: SectorNodeKind::SalvageField,
                station_id: None,
                risk_tier: 1,
                reward_hint: "Low threat, strong salvage".to_string(),
                neighbors: vec![0, 4],
                status: SectorNodeStatus::Fresh,
                position: [-260.0, 150.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: vec!["raider_skiff".to_string()],
                    hostile_count: 1,
                    salvage_value: 10,
                    ambient_heat_pressure: 0,
                    ambient_electrical_pressure: 0,
                    reward_multiplier: 2,
                    arena_variant: "salvage".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x2000,
                        star_density: 132,
                        dust_density: 84,
                        parallax_strength: 0.42,
                        haze_tint: [0.18, 0.26, 0.18],
                        galaxy_tint: [0.88, 0.78, 0.54],
                        galaxy_arc_strength: 0.64,
                    },
                },
            },
            SectorNode {
                id: 6,
                label: "Calibration Ring".to_string(),
                kind: SectorNodeKind::TestRange,
                station_id: None,
                risk_tier: 0,
                reward_hint: "No hostiles, no salvage, pure ship testing".to_string(),
                neighbors: vec![0],
                status: SectorNodeStatus::Fresh,
                position: [320.0, 220.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: Vec::new(),
                    hostile_count: 0,
                    salvage_value: 1,
                    ambient_heat_pressure: 0,
                    ambient_electrical_pressure: 0,
                    reward_multiplier: 1,
                    arena_variant: "test".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x3000,
                        star_density: 64,
                        dust_density: 12,
                        parallax_strength: 0.14,
                        haze_tint: [0.10, 0.14, 0.22],
                        galaxy_tint: [0.60, 0.82, 1.00],
                        galaxy_arc_strength: 0.18,
                    },
                },
            },
            SectorNode {
                id: 2,
                label: "Gravehook Nest".to_string(),
                kind: SectorNodeKind::HostileHold,
                station_id: None,
                risk_tier: 3,
                reward_hint: "Heavy guns, middling haul".to_string(),
                neighbors: vec![0, 4, 5],
                status: SectorNodeStatus::Fresh,
                position: [0.0, 200.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: vec!["scrap_brigand".to_string()],
                    hostile_count: 3,
                    salvage_value: 8,
                    ambient_heat_pressure: 1,
                    ambient_electrical_pressure: 0,
                    reward_multiplier: 3,
                    arena_variant: "hostile".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x4000,
                        star_density: 88,
                        dust_density: 36,
                        parallax_strength: 0.28,
                        haze_tint: [0.28, 0.12, 0.14],
                        galaxy_tint: [0.94, 0.46, 0.34],
                        galaxy_arc_strength: 0.40,
                    },
                },
            },
            SectorNode {
                id: 3,
                label: "Blueglass Hush".to_string(),
                kind: SectorNodeKind::UnstableDerelict,
                station_id: None,
                risk_tier: 2,
                reward_hint: "System stress, moderate reward".to_string(),
                neighbors: vec![0, 5],
                status: SectorNodeStatus::Fresh,
                position: [250.0, 120.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: vec!["raider_skiff".to_string(), "raider_skiff".to_string()],
                    hostile_count: 2,
                    salvage_value: 9,
                    ambient_heat_pressure: 1,
                    ambient_electrical_pressure: 2,
                    reward_multiplier: 3,
                    arena_variant: "unstable".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x5000,
                        star_density: 110,
                        dust_density: 58,
                        parallax_strength: 0.38,
                        haze_tint: [0.14, 0.16, 0.34],
                        galaxy_tint: [0.56, 0.86, 1.00],
                        galaxy_arc_strength: 0.52,
                    },
                },
            },
            SectorNode {
                id: 4,
                label: "Forked Cache".to_string(),
                kind: SectorNodeKind::SalvageField,
                station_id: None,
                risk_tier: 2,
                reward_hint: "Branch route, better payout".to_string(),
                neighbors: vec![1, 2],
                status: SectorNodeStatus::Fresh,
                position: [-320.0, -70.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: vec!["raider_skiff".to_string(), "scrap_brigand".to_string()],
                    hostile_count: 2,
                    salvage_value: 12,
                    ambient_heat_pressure: 1,
                    ambient_electrical_pressure: 1,
                    reward_multiplier: 4,
                    arena_variant: "cache".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x6000,
                        star_density: 124,
                        dust_density: 62,
                        parallax_strength: 0.34,
                        haze_tint: [0.18, 0.20, 0.16],
                        galaxy_tint: [0.96, 0.84, 0.60],
                        galaxy_arc_strength: 0.58,
                    },
                },
            },
            SectorNode {
                id: 5,
                label: "Static Wake".to_string(),
                kind: SectorNodeKind::UnstableDerelict,
                station_id: None,
                risk_tier: 4,
                reward_hint: "Brutal branch, rich recovery".to_string(),
                neighbors: vec![2, 3],
                status: SectorNodeStatus::Fresh,
                position: [300.0, -90.0],
                encounter: EncounterSpec {
                    enemy_ship_ids: vec!["scrap_brigand".to_string(), "raider_skiff".to_string()],
                    hostile_count: 4,
                    salvage_value: 14,
                    ambient_heat_pressure: 2,
                    ambient_electrical_pressure: 3,
                    reward_multiplier: 5,
                    arena_variant: "storm".to_string(),
                    backdrop: EncounterBackdrop {
                        seed: seed ^ 0x7000,
                        star_density: 118,
                        dust_density: 76,
                        parallax_strength: 0.46,
                        haze_tint: [0.18, 0.14, 0.38],
                        galaxy_tint: [0.62, 0.70, 1.00],
                        galaxy_arc_strength: 0.72,
                    },
                },
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backfills_needle_rest_for_legacy_hub_without_station_id() {
        let mut config = default_sector_layout(1);
        for node in &mut config.nodes {
            node.station_id = None;
        }

        backfill_default_station_reference(&mut config);

        let hub = config
            .nodes
            .iter()
            .find(|node| node.kind == SectorNodeKind::HubStation)
            .unwrap();
        assert_eq!(hub.station_id.as_deref(), Some("needle_rest"));
    }

    #[test]
    fn backfill_keeps_existing_station_references() {
        let mut config = default_sector_layout(1);
        for node in &mut config.nodes {
            node.station_id = None;
        }
        config.nodes[1].station_id = Some("existing_station".to_string());

        backfill_default_station_reference(&mut config);

        assert_eq!(
            config.nodes[1].station_id.as_deref(),
            Some("existing_station")
        );
        assert!(
            config
                .nodes
                .iter()
                .find(|node| node.kind == SectorNodeKind::HubStation)
                .unwrap()
                .station_id
                .is_none()
        );
    }
}

#[derive(Component)]
pub(crate) struct SectorMapRoot;

#[derive(Component)]
pub(crate) struct SectorMapCanvas;

#[derive(Component)]
pub(crate) struct SectorMapStatusText;

#[derive(Component)]
pub(crate) struct SectorMapDetailText;

#[derive(Component)]
pub(crate) struct SectorNodeButton {
    pub(crate) node_id: u32,
}

#[derive(Component)]
pub(crate) struct SectorMapNodeBorder;

#[derive(Component)]
pub(crate) struct SectorMapLinkLayer;

#[derive(Component)]
pub(crate) struct SectorMapNodeLayer;

#[derive(Component)]
pub(crate) struct SectorMapLinkDash {
    pub(crate) target_node_id: u32,
    pub(crate) dash_index: u8,
    pub(crate) dash_count: u8,
}

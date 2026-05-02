use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct DemoProgression {
    pub(crate) scrap: u32,
    pub(crate) hull_wear: u32,
    pub(crate) jump_count: u32,
}

impl Default for DemoProgression {
    fn default() -> Self {
        Self {
            scrap: 100,
            hull_wear: 0,
            jump_count: 0,
        }
    }
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
pub(crate) struct EncounterSpec {
    #[serde(default)]
    pub(crate) enemy_ship_ids: Vec<String>,
    pub(crate) hostile_count: u32,
    pub(crate) salvage_value: u32,
    pub(crate) ambient_heat_pressure: i32,
    pub(crate) ambient_electrical_pressure: i32,
    pub(crate) reward_multiplier: u32,
    pub(crate) arena_variant: String,
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
    pub(crate) risk_tier: u8,
    pub(crate) reward_hint: String,
    pub(crate) neighbors: Vec<u32>,
    pub(crate) status: SectorNodeStatus,
    pub(crate) position: [f32; 2],
    pub(crate) encounter: EncounterSpec,
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
        let nodes = vec![
            SectorNode {
                id: 0,
                label: "Needle Rest".to_string(),
                kind: SectorNodeKind::HubStation,
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
                },
            },
            SectorNode {
                id: 1,
                label: "Latchline Debris".to_string(),
                kind: SectorNodeKind::SalvageField,
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
                },
            },
            SectorNode {
                id: 6,
                label: "Calibration Ring".to_string(),
                kind: SectorNodeKind::TestRange,
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
                },
            },
            SectorNode {
                id: 2,
                label: "Gravehook Nest".to_string(),
                kind: SectorNodeKind::HostileHold,
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
                },
            },
            SectorNode {
                id: 3,
                label: "Blueglass Hush".to_string(),
                kind: SectorNodeKind::UnstableDerelict,
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
                },
            },
            SectorNode {
                id: 4,
                label: "Forked Cache".to_string(),
                kind: SectorNodeKind::SalvageField,
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
                },
            },
            SectorNode {
                id: 5,
                label: "Static Wake".to_string(),
                kind: SectorNodeKind::UnstableDerelict,
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
                },
            },
        ];

        Self {
            seed,
            current_node_id: 0,
            selected_node_id: Some(1),
            active_encounter_node_id: None,
            nodes,
        }
    }

    pub(crate) fn ensure_latest_layout(&mut self) {
        let default_state = Self::from_seed(self.seed);

        for default_node in default_state.nodes {
            if self.nodes.iter().all(|node| node.id != default_node.id) {
                self.nodes.push(default_node);
            }
        }

        self.nodes.sort_by_key(|node| node.id);

        if let Some(hub) = self.node_mut(0)
            && !hub.neighbors.contains(&6)
        {
            hub.neighbors.push(6);
            hub.neighbors.sort_unstable();
        }

        if self.selected_node_id.is_none() {
            self.selected_node_id = Some(1);
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

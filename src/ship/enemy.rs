use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use super::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule};

pub const DEFAULT_ENEMY_SHIPS_PATH: &str = "saves/enemy_ships.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyShipEntry {
    pub id: String,
    pub display_name: String,
    pub threat_tier: u8,
    pub behavior_tag: String,
    pub ship: ShipDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnemyShipLibrary {
    pub entries: Vec<EnemyShipEntry>,
}

impl EnemyShipLibrary {
    pub fn seeded() -> Self {
        Self {
            entries: vec![raider_skiff(), scrap_brigand()],
        }
    }

    pub fn ensure_seeded(&mut self) {
        if self.entries.is_empty() {
            *self = Self::seeded();
        }
    }

    pub fn selected_or_first(&self, index: usize) -> Option<&EnemyShipEntry> {
        self.entries.get(index).or_else(|| self.entries.first())
    }

    pub fn selected_or_first_mut(&mut self, index: usize) -> Option<&mut EnemyShipEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let safe_index = index.min(self.entries.len() - 1);
        self.entries.get_mut(safe_index)
    }

    pub fn find_by_id(&self, id: &str) -> Option<&EnemyShipEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    pub fn add_blank_entry(&mut self) -> usize {
        let next_index = self.entries.len() + 1;
        let id = format!("enemy_{next_index}");
        self.entries.push(EnemyShipEntry {
            id: id.clone(),
            display_name: format!("Enemy {next_index}"),
            threat_tier: 1,
            behavior_tag: "aggressive".to_string(),
            ship: ShipDefinition {
                name: format!("Enemy {next_index}"),
                modules: vec![
                    ShipModule::new(1, ModuleKind::Core, 0, 0, 0),
                    ShipModule::new(2, ModuleKind::Cockpit, 0, 1, 0),
                    ShipModule::new(3, ModuleKind::Engine, 0, -1, 2),
                    ShipModule::new(4, ModuleKind::Turret, 1, 0, 1),
                    ShipModule::new(5, ModuleKind::Hull, -1, 0, 3),
                ],
            },
        });
        self.entries.len() - 1
    }
}

pub fn load_default_enemy_library() -> Result<Option<EnemyShipLibrary>, String> {
    load_enemy_library_from_path(Path::new(DEFAULT_ENEMY_SHIPS_PATH))
}

pub fn save_default_enemy_library(library: &EnemyShipLibrary) -> Result<(), String> {
    save_enemy_library_to_path(Path::new(DEFAULT_ENEMY_SHIPS_PATH), library)
}

fn load_enemy_library_from_path(path: &Path) -> Result<Option<EnemyShipLibrary>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let encoded = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read enemy ship library {}: {error}",
            path.display()
        )
    })?;
    let library = serde_json::from_str(&encoded).map_err(|error| {
        format!(
            "failed to decode enemy ship library {}: {error}",
            path.display()
        )
    })?;
    Ok(Some(library))
}

fn save_enemy_library_to_path(path: &Path, library: &EnemyShipLibrary) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create enemy ship library directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let encoded = serde_json::to_string_pretty(library).map_err(|error| {
        format!(
            "failed to encode enemy ship library {}: {error}",
            path.display()
        )
    })?;
    fs::write(path, encoded).map_err(|error| {
        format!(
            "failed to write enemy ship library {}: {error}",
            path.display()
        )
    })
}

fn raider_skiff() -> EnemyShipEntry {
    let mut modules = vec![
        ShipModule::new(1, ModuleKind::Core, 0, 0, 0),
        ShipModule::new(2, ModuleKind::Cockpit, 0, 1, 0),
        ShipModule::new(3, ModuleKind::Reactor, -1, 0, 3),
        ShipModule::new(4, ModuleKind::Engine, 0, -1, 2),
        ShipModule::new(5, ModuleKind::Engine, 1, -1, 2),
        ShipModule::new(6, ModuleKind::Turret, 1, 0, 1),
        ShipModule::new(7, ModuleKind::Hull, -1, 1, 3),
        ShipModule::new(8, ModuleKind::Hull, 1, 1, 1),
        ShipModule::new(9, ModuleKind::Cargo, -2, 0, 3),
    ];
    modules[1].variant = ModuleVariant::AdvancedHelm;
    modules[2].variant = ModuleVariant::Fission;
    modules[5].variant = ModuleVariant::LaserTurret;
    modules[8].variant = ModuleVariant::AmmoRack;
    EnemyShipEntry {
        id: "raider_skiff".to_string(),
        display_name: "Raider Skiff".to_string(),
        threat_tier: 1,
        behavior_tag: "skirmisher".to_string(),
        ship: ShipDefinition {
            name: "Raider Skiff".to_string(),
            modules,
        },
    }
}

fn scrap_brigand() -> EnemyShipEntry {
    let mut modules = vec![
        ShipModule::new(1, ModuleKind::Core, 0, 0, 0),
        ShipModule::new(2, ModuleKind::Cockpit, 0, 1, 0),
        ShipModule::new(3, ModuleKind::Reactor, -1, 0, 3),
        ShipModule::new(4, ModuleKind::Battery, -2, 0, 3),
        ShipModule::new(5, ModuleKind::Engine, 0, -1, 2),
        ShipModule::new(6, ModuleKind::Engine, 1, -1, 2),
        ShipModule::new(7, ModuleKind::Turret, 1, 0, 1),
        ShipModule::new(8, ModuleKind::Turret, 2, 0, 1),
        ShipModule::new(9, ModuleKind::Cargo, -1, 1, 0),
        ShipModule::new(10, ModuleKind::Hull, -2, 1, 3),
        ShipModule::new(11, ModuleKind::Hull, 2, 1, 1),
        ShipModule::new(12, ModuleKind::HullInnerCorner, 2, -1, 2),
        ShipModule::new(13, ModuleKind::Shield, -2, -1, 0),
    ];
    modules[0].variant = ModuleVariant::ExpandedCore;
    modules[2].variant = ModuleVariant::Fusion;
    modules[3].variant = ModuleVariant::Capacitor;
    modules[6].variant = ModuleVariant::BallisticTurret;
    modules[7].variant = ModuleVariant::LaserTurret;
    modules[8].variant = ModuleVariant::FuelTank;
    modules[12].variant = ModuleVariant::DirectionalShield;
    EnemyShipEntry {
        id: "scrap_brigand".to_string(),
        display_name: "Scrap Brigand".to_string(),
        threat_tier: 3,
        behavior_tag: "brawler".to_string(),
        ship: ShipDefinition {
            name: "Scrap Brigand".to_string(),
            modules,
        },
    }
}

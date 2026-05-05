use std::{collections::HashMap, fs, path::Path};

use bevy::log;
use serde::{Deserialize, Serialize};

use super::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule};
use crate::stations::FactionId;

pub const DEFAULT_ENEMY_SHIPS_PATH: &str = "saves/enemy_ships.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyShipEntryValidationStatus {
    Valid,
    RepairedInMemory,
    Invalid,
}

#[derive(Debug, Clone)]
pub struct ValidatedEnemyLibrary {
    pub library: EnemyShipLibrary,
    pub statuses: HashMap<String, EnemyShipEntryValidationStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyShipEntry {
    pub id: String,
    pub display_name: String,
    pub threat_tier: u8,
    pub behavior_tag: String,
    #[serde(default = "default_enemy_faction")]
    pub faction_id: FactionId,
    #[serde(default)]
    pub ship_name: Option<String>,
    #[serde(default)]
    pub captain_name: Option<String>,
    #[serde(default)]
    pub comms_intro: Option<String>,
    #[serde(default)]
    pub comms_outro: Option<String>,
    #[serde(default)]
    pub is_crewed: bool,
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
            faction_id: FactionId::RogueContinuants,
            ship_name: Some(format!("Enemy {next_index}")),
            captain_name: Some("Unassigned Captain".to_string()),
            comms_intro: Some(
                "You are entering a live claim. Turn off your lamps and drift away.".to_string(),
            ),
            comms_outro: Some(
                "This route is spoken for. Remember that the next time you cut into our wrecks."
                    .to_string(),
            ),
            is_crewed: true,
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

    pub fn validate_and_repair_in_memory(
        &mut self,
    ) -> HashMap<String, EnemyShipEntryValidationStatus> {
        let mut statuses = HashMap::default();
        for entry in &mut self.entries {
            let status = match validate_enemy_ship_entry(entry) {
                Ok(()) => EnemyShipEntryValidationStatus::Valid,
                Err(error) => {
                    if let Some(seeded) = seeded_enemy_by_id(&entry.id) {
                        *entry = seeded;
                        log::warn!(
                            "Enemy ship entry '{}' was invalid and has been repaired in memory from seeded defaults: {}",
                            entry.id,
                            error
                        );
                        EnemyShipEntryValidationStatus::RepairedInMemory
                    } else {
                        log::warn!(
                            "Enemy ship entry '{}' is invalid and cannot be repaired automatically: {}",
                            entry.id,
                            error
                        );
                        EnemyShipEntryValidationStatus::Invalid
                    }
                }
            };
            statuses.insert(entry.id.clone(), status);
        }
        statuses
    }
}

pub fn load_default_enemy_library() -> Result<Option<EnemyShipLibrary>, String> {
    load_validated_default_enemy_library().map(|library| library.map(|validated| validated.library))
}

pub fn load_validated_default_enemy_library() -> Result<Option<ValidatedEnemyLibrary>, String> {
    load_enemy_library_from_path(Path::new(DEFAULT_ENEMY_SHIPS_PATH)).map(|library| {
        library.map(|mut library| {
            let statuses = library.validate_and_repair_in_memory();
            ValidatedEnemyLibrary { library, statuses }
        })
    })
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

pub fn validate_enemy_ship_entry(entry: &EnemyShipEntry) -> Result<(), String> {
    validate_enemy_ship_definition(&entry.ship)
}

pub fn validate_enemy_ship_definition(ship: &ShipDefinition) -> Result<(), String> {
    if ship.modules.is_empty() {
        return Err("ship has no modules".to_string());
    }
    if !ship
        .modules
        .iter()
        .any(|module| module.kind == ModuleKind::Core)
    {
        return Err("ship is missing a core".to_string());
    }
    if !ship
        .modules
        .iter()
        .any(|module| module.kind == ModuleKind::Cockpit)
    {
        return Err("ship is missing a cockpit".to_string());
    }
    Ok(())
}

fn seeded_enemy_by_id(id: &str) -> Option<EnemyShipEntry> {
    EnemyShipLibrary::seeded()
        .entries
        .into_iter()
        .find(|entry| entry.id == id)
}

fn default_enemy_faction() -> FactionId {
    FactionId::RogueContinuants
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
        faction_id: FactionId::RogueContinuants,
        ship_name: Some("Cutter Ashfall".to_string()),
        captain_name: Some("Captain Ilex Marr".to_string()),
        comms_intro: Some(
            "Cutter Ashfall to unknown crew: this lane is under private recovery. Break away or be broken.".to_string(),
        ),
        comms_outro: Some(
            "You got your warning. Needle Rest keeps sending crews into other people's futures.".to_string(),
        ),
        is_crewed: true,
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
        faction_id: FactionId::RogueContinuants,
        ship_name: Some("Saint-of-Cinders".to_string()),
        captain_name: Some("Cell Lead Orin Vey".to_string()),
        comms_intro: Some(
            "Saint-of-Cinders hails: leave your cargo and your route marks, and we might let the station keep your names.".to_string(),
        ),
        comms_outro: Some(
            "Needle Rest doesn't own this corridor. People like us survived the Quiet by learning that first.".to_string(),
        ),
        is_crewed: true,
        ship: ShipDefinition {
            name: "Scrap Brigand".to_string(),
            modules,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_entry(id: &str) -> EnemyShipEntry {
        seeded_enemy_by_id(id).unwrap()
    }

    #[test]
    fn validation_rejects_empty_ship() {
        let mut entry = valid_entry("raider_skiff");
        entry.ship.modules.clear();
        assert!(validate_enemy_ship_entry(&entry).is_err());
    }

    #[test]
    fn validation_rejects_ship_without_core() {
        let mut entry = valid_entry("raider_skiff");
        entry
            .ship
            .modules
            .retain(|module| module.kind != ModuleKind::Core);
        assert!(validate_enemy_ship_entry(&entry).is_err());
    }

    #[test]
    fn validation_rejects_ship_without_cockpit() {
        let mut entry = valid_entry("raider_skiff");
        entry
            .ship
            .modules
            .retain(|module| module.kind != ModuleKind::Cockpit);
        assert!(validate_enemy_ship_entry(&entry).is_err());
    }

    #[test]
    fn invalid_seeded_entry_is_repaired_in_memory() {
        let mut library = EnemyShipLibrary {
            entries: vec![EnemyShipEntry {
                id: "raider_skiff".to_string(),
                display_name: "Broken Raider".to_string(),
                threat_tier: 1,
                behavior_tag: "skirmisher".to_string(),
                faction_id: FactionId::RogueContinuants,
                ship_name: Some("Broken Raider".to_string()),
                captain_name: Some("Captain Placeholder".to_string()),
                comms_intro: None,
                comms_outro: None,
                is_crewed: true,
                ship: ShipDefinition::empty("Broken Raider"),
            }],
        };
        let statuses = library.validate_and_repair_in_memory();
        assert_eq!(
            statuses.get("raider_skiff"),
            Some(&EnemyShipEntryValidationStatus::RepairedInMemory)
        );
        assert!(!library.entries[0].ship.modules.is_empty());
    }

    #[test]
    fn invalid_unknown_entry_stays_invalid() {
        let mut library = EnemyShipLibrary {
            entries: vec![EnemyShipEntry {
                id: "custom_unknown".to_string(),
                display_name: "Broken Custom".to_string(),
                threat_tier: 1,
                behavior_tag: "aggressive".to_string(),
                faction_id: FactionId::RogueContinuants,
                ship_name: Some("Broken Custom".to_string()),
                captain_name: Some("Captain Placeholder".to_string()),
                comms_intro: None,
                comms_outro: None,
                is_crewed: true,
                ship: ShipDefinition::empty("Broken Custom"),
            }],
        };
        let statuses = library.validate_and_repair_in_memory();
        assert_eq!(
            statuses.get("custom_unknown"),
            Some(&EnemyShipEntryValidationStatus::Invalid)
        );
        assert!(library.entries[0].ship.modules.is_empty());
    }
}

use std::{fs, path::Path};

use super::ShipDefinition;

pub const DEFAULT_EDITOR_SHIP_PATH: &str = "saves/editor_ship.json";

pub fn load_default_ship() -> Result<Option<ShipDefinition>, String> {
    load_ship_from_path(Path::new(DEFAULT_EDITOR_SHIP_PATH))
}

pub fn save_default_ship(ship: &ShipDefinition) -> Result<(), String> {
    save_ship_to_path(Path::new(DEFAULT_EDITOR_SHIP_PATH), ship)
}

pub fn load_ship_from_path(path: &Path) -> Result<Option<ShipDefinition>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let encoded = fs::read_to_string(path)
        .map_err(|error| format!("failed to read ship file {}: {error}", path.display()))?;
    let ship = serde_json::from_str(&encoded)
        .map_err(|error| format!("failed to decode ship file {}: {error}", path.display()))?;
    Ok(Some(ship))
}

pub fn save_ship_to_path(path: &Path, ship: &ShipDefinition) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create ship save directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let encoded = serde_json::to_string_pretty(ship).map_err(|error| {
        format!(
            "failed to encode ship definition for {}: {error}",
            path.display()
        )
    })?;
    fs::write(path, encoded)
        .map_err(|error| format!("failed to write ship file {}: {error}", path.display()))
}

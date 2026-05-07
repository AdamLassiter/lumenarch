use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use super::state::{LastMissionReport, Progression, SectorState};

const DEFAULT_CAMPAIGN_PATH: &str = "saves/campaign_state.json";

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct CampaignSave {
    pub(crate) progression: Progression,
    pub(crate) sector: SectorState,
    pub(crate) last_mission_report: LastMissionReport,
}

pub(crate) fn load_campaign() -> Result<Option<CampaignSave>, String> {
    load_campaign_from_path(Path::new(DEFAULT_CAMPAIGN_PATH))
}

pub(crate) fn save_campaign(save: &CampaignSave) -> Result<(), String> {
    save_campaign_to_path(Path::new(DEFAULT_CAMPAIGN_PATH), save)
}

fn load_campaign_from_path(path: &Path) -> Result<Option<CampaignSave>, String> {
    if !path.exists() {
        return Ok(None);
    }

    let encoded = fs::read_to_string(path)
        .map_err(|error| format!("failed to read campaign file {}: {error}", path.display()))?;
    let save = serde_json::from_str(&encoded)
        .map_err(|error| format!("failed to decode campaign file {}: {error}", path.display()))?;
    Ok(Some(save))
}

fn save_campaign_to_path(path: &Path, save: &CampaignSave) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create campaign save directory {}: {error}",
                parent.display()
            )
        })?;
    }

    let encoded = serde_json::to_string_pretty(save)
        .map_err(|error| format!("failed to encode campaign save {}: {error}", path.display()))?;
    fs::write(path, encoded)
        .map_err(|error| format!("failed to write campaign file {}: {error}", path.display()))
}

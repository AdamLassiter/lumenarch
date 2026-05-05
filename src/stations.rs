use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

const DEFAULT_STATIONS_PATH: &str = "saves/stations.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum FactionId {
    ContinuantGuild,
    RogueContinuants,
    NullSwarms,
}

impl FactionId {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::ContinuantGuild => "Continuant Guild",
            Self::RogueContinuants => "Rogue Continuants",
            Self::NullSwarms => "Null Swarms",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StationContact {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) role: String,
    pub(crate) bio: String,
    pub(crate) brief: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum StationService {
    Shipyard,
    Quartermaster,
    Contracts,
    Archives,
}

impl StationService {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Shipyard => "Shipyard",
            Self::Quartermaster => "Quartermaster",
            Self::Contracts => "Contract Board",
            Self::Archives => "Archives",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum StationContractKind {
    SalvageRecovery,
    Calibration,
    HostileCleanup,
    Retrieval,
}

impl StationContractKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::SalvageRecovery => "Salvage Recovery",
            Self::Calibration => "Calibration",
            Self::HostileCleanup => "Hostile Cleanup",
            Self::Retrieval => "Retrieval",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StationContract {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) contact_id: String,
    pub(crate) target_node_id: u32,
    pub(crate) kind: StationContractKind,
    pub(crate) briefing: String,
    pub(crate) launch_blurb: String,
    pub(crate) success_debrief: String,
    pub(crate) failure_debrief: String,
    #[serde(default)]
    pub(crate) reward_bonus_scrap: u32,
    #[serde(default)]
    pub(crate) lore_unlock_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct LoreEntry {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StationDefinition {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) faction: FactionId,
    pub(crate) flavor: String,
    pub(crate) contacts: Vec<StationContact>,
    pub(crate) services: Vec<StationService>,
    pub(crate) contracts: Vec<StationContract>,
    pub(crate) lore_entries: Vec<LoreEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StationCatalog {
    pub(crate) stations: Vec<StationDefinition>,
}

impl StationCatalog {
    pub(crate) fn station(&self, station_id: &str) -> Option<&StationDefinition> {
        self.stations
            .iter()
            .find(|station| station.id == station_id)
    }

    pub(crate) fn contract(
        &self,
        contract_id: &str,
    ) -> Option<(&StationDefinition, &StationContract)> {
        self.stations.iter().find_map(|station| {
            station
                .contracts
                .iter()
                .find(|contract| contract.id == contract_id)
                .map(|contract| (station, contract))
        })
    }

    pub(crate) fn contract_by_index(
        &self,
        station_id: &str,
        contract_index: usize,
    ) -> Option<(&StationDefinition, &StationContract)> {
        let station = self.station(station_id)?;
        let contract = station.contracts.get(contract_index)?;
        Some((station, contract))
    }

    pub(crate) fn contact<'a>(
        &'a self,
        station_id: &str,
        contact_id: &str,
    ) -> Option<&'a StationContact> {
        self.station(station_id)?
            .contacts
            .iter()
            .find(|contact| contact.id == contact_id)
    }

    pub(crate) fn lore<'a>(&'a self, station_id: &str, lore_id: &str) -> Option<&'a LoreEntry> {
        self.station(station_id)?
            .lore_entries
            .iter()
            .find(|entry| entry.id == lore_id)
    }
}

pub(crate) fn current_station_id(sector: &crate::state::SectorState) -> Option<&str> {
    sector.current_node()?.station_id.as_deref()
}

pub(crate) fn current_station<'a>(
    catalog: &'a StationCatalog,
    sector: &crate::state::SectorState,
) -> Option<&'a StationDefinition> {
    catalog.station(current_station_id(sector)?)
}

#[derive(bevy::prelude::Resource, Clone, Debug, Default)]
pub(crate) struct StationCatalogResource(pub(crate) StationCatalog);

impl StationCatalogResource {
    pub(crate) fn load_or_default() -> Self {
        match load_or_create_default_stations() {
            Ok(catalog) => Self(catalog),
            Err(error) => {
                eprintln!("stations: failed to load station catalog, using defaults: {error}");
                Self(default_station_catalog())
            }
        }
    }
}

pub(crate) fn load_or_create_default_stations() -> Result<StationCatalog, String> {
    let path = Path::new(DEFAULT_STATIONS_PATH);
    if path.exists() {
        let encoded = fs::read_to_string(path).map_err(|error| {
            format!("failed to read station catalog {}: {error}", path.display())
        })?;
        let catalog = serde_json::from_str(&encoded).map_err(|error| {
            format!(
                "failed to decode station catalog {}: {error}",
                path.display()
            )
        })?;
        return Ok(catalog);
    }

    let catalog = default_station_catalog();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create station catalog directory {}: {error}",
                parent.display()
            )
        })?;
    }
    let encoded = serde_json::to_string_pretty(&catalog).map_err(|error| {
        format!(
            "failed to encode station catalog {}: {error}",
            path.display()
        )
    })?;
    fs::write(path, encoded).map_err(|error| {
        format!(
            "failed to write station catalog {}: {error}",
            path.display()
        )
    })?;
    Ok(catalog)
}

pub(crate) fn default_station_catalog() -> StationCatalog {
    StationCatalog {
        stations: vec![StationDefinition {
            id: "needle_rest".to_string(),
            name: "Needle Rest".to_string(),
            faction: FactionId::ContinuantGuild,
            flavor: "A Continuant splice-station threaded through a dead shipping spar. Needle Rest survives by salvaging the Quiet After one careful sortie at a time.".to_string(),
            contacts: vec![
                StationContact {
                    id: "ivra_quell".to_string(),
                    name: "Ivra Quell".to_string(),
                    role: "Harbormaster".to_string(),
                    bio: "Ivra keeps the slips alive, knows every hull scar in the yard, and speaks like every minute dockside is borrowed from a worse emergency.".to_string(),
                    brief: "Keep the ship flying, keep the crew breathing, and don't waste a clean launch window.".to_string(),
                },
                StationContact {
                    id: "sable_ren".to_string(),
                    name: "Sable Ren".to_string(),
                    role: "Contract Broker".to_string(),
                    bio: "Sable turns station rumors into paid routes. She cares less about glory than about what can be carried home and who will still trade with Needle Rest tomorrow.".to_string(),
                    brief: "Take the jobs that keep us useful. Leave the heroic nonsense to dead captains.".to_string(),
                },
                StationContact {
                    id: "peregrine_cho".to_string(),
                    name: "Peregrine Cho".to_string(),
                    role: "Archive Custodian".to_string(),
                    bio: "Cho curates logs, fragments, and oral accounts from the Quiet After. He treats every recovered scrap as an argument against forgetting.".to_string(),
                    brief: "History here survives in fragments. Bring anything that still speaks.".to_string(),
                },
            ],
            services: vec![
                StationService::Shipyard,
                StationService::Quartermaster,
                StationService::Contracts,
                StationService::Archives,
            ],
            contracts: vec![
                StationContract {
                    id: "needle_calibration_ring".to_string(),
                    title: "Calibration Ring Shakeout".to_string(),
                    contact_id: "ivra_quell".to_string(),
                    target_node_id: 6,
                    kind: StationContractKind::Calibration,
                    briefing: "Take the fresh refit through the Calibration Ring. We need a clean systems shakedown before riskier work leaves the dock.".to_string(),
                    launch_blurb: "Quell patches you into the Ring telemetry feed. 'Clean lap, clean readback, then come home.'".to_string(),
                    success_debrief: "Calibration pass logged. The yard signs off on the current configuration and updates its trust in your hull math.".to_string(),
                    failure_debrief: "The shakedown came back dirty. Needle Rest logs the faults and Quell wants the next launch treated as remedial, not routine.".to_string(),
                    reward_bonus_scrap: 2,
                    lore_unlock_ids: vec!["needle_rest_flight_discipline".to_string()],
                },
                StationContract {
                    id: "needle_latchline_recovery".to_string(),
                    title: "Latchline Recovery Sweep".to_string(),
                    contact_id: "sable_ren".to_string(),
                    target_node_id: 1,
                    kind: StationContractKind::SalvageRecovery,
                    briefing: "Latchline Debris is still coughing up useful structure. Bring back what you can before raiders strip the route bare.".to_string(),
                    launch_blurb: "Ren forwards a salvage manifest with half the items already crossed out by competitors.".to_string(),
                    success_debrief: "The sweep paid out. Ren posts the haul under Needle Rest colors and opens a few more quiet favors on the local exchange.".to_string(),
                    failure_debrief: "The route went wrong and the station gets less for showing its badge on that wreck field than Ren promised.".to_string(),
                    reward_bonus_scrap: 5,
                    lore_unlock_ids: vec!["quiet_after_salvage_custom".to_string()],
                },
                StationContract {
                    id: "needle_gravehook_cleanup".to_string(),
                    title: "Gravehook Interdiction".to_string(),
                    contact_id: "sable_ren".to_string(),
                    target_node_id: 2,
                    kind: StationContractKind::HostileCleanup,
                    briefing: "A rogue Continuant cell has been leaning on traffic near Gravehook Nest. Break their hold long enough for station scavengers to work again.".to_string(),
                    launch_blurb: "Ren's brief ends with a captain name underlined twice: 'Expect a thinking crew, not a blind machine.'".to_string(),
                    success_debrief: "Traffic opens again and Ren quietly admits the station needed this win more than the ledger suggests.".to_string(),
                    failure_debrief: "The rogue cell keeps its grip on Gravehook. Needle Rest loses face and another route turns expensive.".to_string(),
                    reward_bonus_scrap: 8,
                    lore_unlock_ids: vec!["rogue_continuants_field_note".to_string()],
                },
                StationContract {
                    id: "needle_blueglass_retrieval".to_string(),
                    title: "Blueglass Archive Pull".to_string(),
                    contact_id: "peregrine_cho".to_string(),
                    target_node_id: 3,
                    kind: StationContractKind::Retrieval,
                    briefing: "Cho wants surviving records from Blueglass Hush. The node is unstable, but the archive fragments there may predate the Quiet After.".to_string(),
                    launch_blurb: "Cho sends a tag-list of dead names and lost facilities, then quietly asks you not to leave the logs behind this time.".to_string(),
                    success_debrief: "Recovered archive traces fold into Cho's growing map of the Quiet After. He treats the return like a funeral conducted correctly.".to_string(),
                    failure_debrief: "The archive pull failed. Cho says little, but the silence feels heavier than an argument.".to_string(),
                    reward_bonus_scrap: 6,
                    lore_unlock_ids: vec!["continuant_memory_practice".to_string()],
                },
            ],
            lore_entries: vec![
                LoreEntry {
                    id: "needle_rest_foundation".to_string(),
                    title: "Needle Rest".to_string(),
                    body: "Needle Rest was assembled after the Quiet After from a transit spar, a broken spindle dock, and whatever hullwork the early crews could keep pressurized. Its people believe a place becomes real when enough crews can return there alive.".to_string(),
                },
                LoreEntry {
                    id: "quiet_after_salvage_custom".to_string(),
                    title: "Quiet After Salvage Law".to_string(),
                    body: "Out here, salvage rights are mostly the rights you can enforce and survive. Stations like Needle Rest survive by making those rights social before they have to be violent.".to_string(),
                },
                LoreEntry {
                    id: "continuant_memory_practice".to_string(),
                    title: "Continuant Memory Practice".to_string(),
                    body: "Continuant stations keep layered records: machine logs, witness accounts, route songs, and repair marks cut into hull braces. If one memory format dies, another may still carry the shape of what mattered.".to_string(),
                },
                LoreEntry {
                    id: "needle_rest_flight_discipline".to_string(),
                    title: "Flight Discipline".to_string(),
                    body: "Every launch from Needle Rest is tracked like a wound under observation. The station remembers which crews came back, which hulls failed warm, and which routes punish carelessness faster than fear can teach it.".to_string(),
                },
                LoreEntry {
                    id: "rogue_continuants_field_note".to_string(),
                    title: "Rogue Continuants".to_string(),
                    body: "Not every Continuant crew stayed tied to a station. Some turned their ships into roaming cells, trading obligation for autonomy and often mistaking that freedom for permission.".to_string(),
                },
                LoreEntry {
                    id: "null_swarms_brief".to_string(),
                    title: "Null Swarms".to_string(),
                    body: "Null Swarms are machine clusters that survived the Quiet After without anyone left to contextualize their directives. Some hold routes, some strip wrecks, and some simply continue patterns no one living can explain.".to_string(),
                },
            ],
        }],
    }
}

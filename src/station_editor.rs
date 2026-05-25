use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::{
    HOVERED_BUTTON,
    PRESSED_BUTTON,
    netcode,
    ship::{ModuleKind, ModuleVariant, ShipDefinition, ShipModule},
    state::{
        EditorMode,
        EditorSessionState,
        EditorShip,
        FocusedTextBox,
        SectorState,
        StationEditorTool,
        StationNextButton,
        StationPrevButton,
    },
    stations::{
        StationCatalogResource,
        StationDefinition,
        StationNpcServiceAction,
        load_or_create_default_stations,
        save_station_catalog,
    },
};

#[derive(Resource, Clone, Debug)]
pub(crate) struct StationEditorState {
    pub(crate) selected_station_index: usize,
    pub(crate) selected_tool: Option<StationEditorTool>,
    pub(crate) status: String,
}

impl Default for StationEditorState {
    fn default() -> Self {
        Self {
            selected_station_index: 0,
            selected_tool: None,
            status: "F5 saves stations. F9 reloads stations.".to_string(),
        }
    }
}

pub(crate) fn station_tool_sprite_path(tool: StationEditorTool) -> &'static str {
    match tool {
        StationEditorTool::EngineerNpc => StationNpcServiceAction::Engineer.sprite_path(),
        StationEditorTool::ContractsNpc => StationNpcServiceAction::Contracts.sprite_path(),
        StationEditorTool::ArchivesNpc => StationNpcServiceAction::Archives.sprite_path(),
    }
}

pub(crate) fn station_npc_sprite_path_for_module(module: &ShipModule) -> Option<&'static str> {
    station_npc_service_action_for_module(module).map(StationNpcServiceAction::sprite_path)
}

pub(crate) fn sync_editor_ship_from_station(
    stations: &StationCatalogResource,
    editor_state: &StationEditorState,
    editor_ship: &mut EditorShip,
) {
    editor_ship.ship = stations
        .0
        .stations
        .get(editor_state.selected_station_index)
        .map(|station| station.ship.clone())
        .unwrap_or_else(|| ShipDefinition::empty("Untitled Station"));
}

pub(crate) fn referenced_station_ids(sector_state: &SectorState) -> Vec<String> {
    let mut ids = Vec::new();
    for node in &sector_state.nodes {
        if let Some(station_id) = node.station_id.as_ref()
            && !ids.iter().any(|existing| existing == station_id)
        {
            ids.push(station_id.clone());
        }
    }
    ids
}

pub(crate) fn ensure_selected_station_reference(
    sector_state: &SectorState,
    stations: &mut StationCatalogResource,
    editor_state: &mut StationEditorState,
) {
    let referenced = referenced_station_ids(sector_state);
    if referenced.is_empty() {
        editor_state.selected_station_index = editor_state
            .selected_station_index
            .min(stations.0.stations.len().saturating_sub(1));
        return;
    }
    let current_id = stations
        .0
        .stations
        .get(editor_state.selected_station_index)
        .map(|station| station.id.as_str());
    if current_id.is_some_and(|id| referenced.iter().any(|reference| reference == id)) {
        return;
    }
    editor_state.selected_station_index = ensure_station_for_id(stations, &referenced[0]);
}

pub(crate) fn reload_station_catalog(
    stations: &mut StationCatalogResource,
    editor_state: &mut StationEditorState,
    editor_ship: &mut EditorShip,
) -> Result<(), String> {
    stations.0 = load_or_create_default_stations()?;
    editor_state.selected_station_index = editor_state
        .selected_station_index
        .min(stations.0.stations.len().saturating_sub(1));
    sync_editor_ship_from_station(stations, editor_state, editor_ship);
    editor_state.status = "Reloaded station catalog.".to_string();
    Ok(())
}

pub(crate) fn save_station_editor_catalog(
    stations: &mut StationCatalogResource,
    editor_state: &StationEditorState,
    editor_ship: &EditorShip,
) -> Result<(), String> {
    if editor_ship.ship.foundation_tiles.is_empty()
        && editor_ship.ship.hull_tiles.is_empty()
        && editor_ship.ship.modules.is_empty()
    {
        return Err("refusing to save an empty station layout".to_string());
    }
    sync_selected_station_from_editor(stations, editor_state, editor_ship);
    save_station_catalog(&stations.0)
}

pub(crate) fn sync_selected_station_from_editor(
    stations: &mut StationCatalogResource,
    editor_state: &StationEditorState,
    editor_ship: &EditorShip,
) {
    sync_station_from_editor(stations, editor_state, editor_ship);
}

pub(crate) fn apply_station_editor_build_action(
    editor_state: &mut StationEditorState,
    editor_ship: &mut EditorShip,
    grid_x: i32,
    grid_y: i32,
    rotation_quadrants: u8,
    erase: bool,
) -> bool {
    let Some(tool) = editor_state.selected_tool else {
        return false;
    };

    if erase {
        erase_station_extra(editor_ship, grid_x, grid_y, tool);
        editor_state.status = format!("Erased {} at ({grid_x}, {grid_y}).", tool.label());
        return true;
    }

    match tool {
        StationEditorTool::EngineerNpc => {
            upsert_station_npc_module(
                &mut editor_ship.ship,
                grid_x,
                grid_y,
                rotation_quadrants,
                StationNpcServiceAction::Engineer,
            );
        }
        StationEditorTool::ContractsNpc => {
            upsert_station_npc_module(
                &mut editor_ship.ship,
                grid_x,
                grid_y,
                rotation_quadrants,
                StationNpcServiceAction::Contracts,
            );
        }
        StationEditorTool::ArchivesNpc => {
            upsert_station_npc_module(
                &mut editor_ship.ship,
                grid_x,
                grid_y,
                rotation_quadrants,
                StationNpcServiceAction::Archives,
            );
        }
    }

    editor_state.status = format!("{} at ({grid_x}, {grid_y}).", tool.label());
    true
}

fn sync_station_from_editor(
    stations: &mut StationCatalogResource,
    editor_state: &StationEditorState,
    editor_ship: &EditorShip,
) {
    if let Some(station) = stations
        .0
        .stations
        .get_mut(editor_state.selected_station_index)
    {
        station.name = editor_ship.ship.name.clone();
        station.ship = editor_ship.ship.clone();
    }
}

/// Handles station catalog browser buttons so authors can step between referenced station entries.
pub(crate) fn station_library_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&StationPrevButton>,
            Option<&StationNextButton>,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<StationPrevButton>, With<StationNextButton>)>,
        ),
    >,
    editor_session: Res<EditorSessionState>,
    mut stations: ResMut<StationCatalogResource>,
    mut station_editor_state: ResMut<StationEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    sector_state: Res<SectorState>,
) {
    if !netcode::is_host_authority(&status) || editor_session.mode != EditorMode::Station {
        return;
    }

    for (interaction, mut background, prev, next) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(PRESSED_BUTTON);
                sync_selected_station_from_editor(
                    &mut stations,
                    &station_editor_state,
                    &editor_ship,
                );
                if prev.is_some() {
                    cycle_referenced_station(
                        &sector_state,
                        &mut stations,
                        &mut station_editor_state,
                        -1,
                    );
                } else if next.is_some() {
                    cycle_referenced_station(
                        &sector_state,
                        &mut stations,
                        &mut station_editor_state,
                        1,
                    );
                }
                sync_editor_ship_from_station(&stations, &station_editor_state, &mut editor_ship);
                if let Err(error) = save_station_catalog(&stations.0) {
                    station_editor_state.status = format!("Station catalog save failed: {error}");
                } else {
                    station_editor_state.status = "Station catalog saved.".to_string();
                }
            }
            Interaction::Hovered => {
                *background = BackgroundColor(HOVERED_BUTTON);
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgb(0.24, 0.32, 0.48));
            }
        }
    }
}

/// Mirrors station catalog navigation onto the keyboard so station iteration matches enemy editing.
pub(crate) fn station_library_keyboard_shortcuts(
    keys: Res<ButtonInput<KeyCode>>,
    focused_textbox: Res<FocusedTextBox>,
    editor_session: Res<EditorSessionState>,
    mut stations: ResMut<StationCatalogResource>,
    mut station_editor_state: ResMut<StationEditorState>,
    mut editor_ship: ResMut<EditorShip>,
    status: Res<netcode::SessionStatus>,
    sector_state: Res<SectorState>,
) {
    if focused_textbox.field.is_some()
        || !netcode::is_host_authority(&status)
        || editor_session.mode != EditorMode::Station
    {
        return;
    }

    let mut changed = false;
    if keys.just_pressed(KeyCode::BracketLeft) {
        sync_selected_station_from_editor(&mut stations, &station_editor_state, &editor_ship);
        cycle_referenced_station(&sector_state, &mut stations, &mut station_editor_state, -1);
        changed = true;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        sync_selected_station_from_editor(&mut stations, &station_editor_state, &editor_ship);
        cycle_referenced_station(&sector_state, &mut stations, &mut station_editor_state, 1);
        changed = true;
    }

    if changed {
        sync_editor_ship_from_station(&stations, &station_editor_state, &mut editor_ship);
        if let Err(error) = save_station_catalog(&stations.0) {
            station_editor_state.status = format!("Station catalog save failed: {error}");
        } else {
            station_editor_state.status = "Station catalog saved.".to_string();
        }
    }
}

pub(crate) fn station_entry_label(
    editor_session: &EditorSessionState,
    stations: &StationCatalogResource,
    station_editor_state: &StationEditorState,
) -> String {
    if editor_session.mode != EditorMode::Station {
        return "Player Ship".to_string();
    }
    let Some(station) = stations
        .0
        .stations
        .get(station_editor_state.selected_station_index)
    else {
        return "No Station Entry".to_string();
    };
    format!("{} [{}]", station.name, station.id)
}

pub(crate) fn station_config_references_text(
    sector_state: &SectorState,
    stations: &StationCatalogResource,
) -> String {
    let mut references: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in &sector_state.nodes {
        if let Some(station_id) = node.station_id.as_ref() {
            references
                .entry(station_id.clone())
                .or_default()
                .push(format!("{} {}", node.id, node.label));
        }
    }

    let mut lines = vec!["Config Station References".to_string()];
    if references.is_empty() {
        lines.push("No station ids referenced by sector config.".to_string());
    }
    let mut referenced_ids = Vec::new();
    for (station_id, nodes) in references {
        referenced_ids.push(station_id.clone());
        let station_label = stations
            .0
            .station(&station_id)
            .map(|station| station.name.as_str())
            .unwrap_or("missing from catalog");
        let status = if stations.0.station(&station_id).is_some() {
            "valid"
        } else {
            "missing"
        };
        lines.push(format!(
            "{station_id} - {station_label} [{status}]\n  used by: {}",
            nodes.join(", ")
        ));
    }
    for station in &stations.0.stations {
        if !referenced_ids.iter().any(|id| id == &station.id) {
            lines.push(format!("{} - {} [unreferenced]", station.id, station.name));
        }
    }
    lines.join("\n")
}

fn cycle_referenced_station(
    sector_state: &SectorState,
    stations: &mut StationCatalogResource,
    editor_state: &mut StationEditorState,
    direction: i32,
) {
    let referenced = referenced_station_ids(sector_state);
    if referenced.is_empty() {
        let len = stations.0.stations.len();
        if len > 0 {
            editor_state.selected_station_index = (editor_state.selected_station_index as i32
                + direction)
                .rem_euclid(len as i32) as usize;
        }
        return;
    }
    let current_id = stations
        .0
        .stations
        .get(editor_state.selected_station_index)
        .map(|station| station.id.as_str());
    let current_reference_index = current_id
        .and_then(|id| referenced.iter().position(|reference| reference == id))
        .unwrap_or(0);
    let next_reference_index =
        (current_reference_index as i32 + direction).rem_euclid(referenced.len() as i32) as usize;
    editor_state.selected_station_index =
        ensure_station_for_id(stations, &referenced[next_reference_index]);
}

fn ensure_station_for_id(stations: &mut StationCatalogResource, station_id: &str) -> usize {
    if let Some(index) = stations
        .0
        .stations
        .iter()
        .position(|station| station.id == station_id)
    {
        return index;
    }
    stations.0.stations.push(StationDefinition {
        id: station_id.to_string(),
        name: station_id.to_string(),
        faction: crate::stations::FactionId::ContinuantGuild,
        flavor: "A referenced station awaiting authoring.".to_string(),
        ship: ShipDefinition::empty(station_id),
        contacts: Vec::new(),
        services: Vec::new(),
        contracts: Vec::new(),
        lore_entries: Vec::new(),
    });
    stations.0.stations.len() - 1
}

fn erase_station_extra(
    editor_ship: &mut EditorShip,
    grid_x: i32,
    grid_y: i32,
    tool: StationEditorTool,
) {
    match tool {
        StationEditorTool::EngineerNpc
        | StationEditorTool::ContractsNpc
        | StationEditorTool::ArchivesNpc => {
            editor_ship.ship.modules.retain(|module| {
                !(module.grid_x == grid_x
                    && module.grid_y == grid_y
                    && station_npc_service_action_for_module(module).is_some())
            });
        }
    }
}

fn upsert_station_npc_module(
    ship: &mut ShipDefinition,
    grid_x: i32,
    grid_y: i32,
    rotation_quadrants: u8,
    service_action: StationNpcServiceAction,
) {
    let kind = station_npc_module_kind(Some(service_action));
    let variant = station_npc_module_variant(Some(service_action));
    ship.modules.retain(|module| {
        !(module.grid_x == grid_x
            && module.grid_y == grid_y
            && station_npc_service_action_for_module(module).is_some())
    });
    let mut module = ShipModule::new(
        ship.next_module_id(),
        kind,
        grid_x,
        grid_y,
        rotation_quadrants % 4,
    );
    module.variant = variant;
    ship.modules.push(module);
}

fn station_npc_module_kind(service_action: Option<StationNpcServiceAction>) -> ModuleKind {
    match service_action {
        Some(_) => ModuleKind::Interior,
        None => ModuleKind::Interior,
    }
}

fn station_npc_module_variant(service_action: Option<StationNpcServiceAction>) -> ModuleVariant {
    match service_action {
        Some(StationNpcServiceAction::Engineer) => ModuleVariant::StationEngineerNpc,
        Some(StationNpcServiceAction::Contracts) => ModuleVariant::StationContractsNpc,
        Some(StationNpcServiceAction::Archives) => ModuleVariant::StationArchivesNpc,
        None => ModuleVariant::Standard,
    }
}

pub(crate) fn station_npc_service_action_for_module(
    module: &ShipModule,
) -> Option<StationNpcServiceAction> {
    match (module.kind, module.variant) {
        (ModuleKind::Interior, ModuleVariant::StationEngineerNpc) => {
            Some(StationNpcServiceAction::Engineer)
        }
        (ModuleKind::Interior, ModuleVariant::StationContractsNpc) => {
            Some(StationNpcServiceAction::Contracts)
        }
        (ModuleKind::Interior, ModuleVariant::StationArchivesNpc) => {
            Some(StationNpcServiceAction::Archives)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state::{EncounterBackdrop, EncounterSpec, SectorNode, SectorNodeKind, SectorNodeStatus},
        stations::{FactionId, StationCatalog},
    };

    fn sector_with_station_refs(station_refs: &[Option<&str>]) -> SectorState {
        SectorState {
            seed: 1,
            current_node_id: 0,
            selected_node_id: None,
            active_encounter_node_id: None,
            nodes: station_refs
                .iter()
                .enumerate()
                .map(|(index, station_id)| SectorNode {
                    id: index as u32,
                    label: format!("Node {index}"),
                    kind: SectorNodeKind::HubStation,
                    station_id: station_id.map(str::to_string),
                    risk_tier: 0,
                    reward_hint: String::new(),
                    neighbors: Vec::new(),
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
                        backdrop: EncounterBackdrop::default(),
                    },
                })
                .collect(),
        }
    }

    fn station_definition(id: &str) -> StationDefinition {
        StationDefinition {
            id: id.to_string(),
            name: format!("{id} display"),
            faction: FactionId::ContinuantGuild,
            flavor: String::new(),
            ship: ShipDefinition::empty(id),
            contacts: Vec::new(),
            services: Vec::new(),
            contracts: Vec::new(),
            lore_entries: Vec::new(),
        }
    }

    #[test]
    fn station_references_define_navigation_order_without_duplicates() {
        let sector = sector_with_station_refs(&[
            Some("needle_rest"),
            Some("outer_lock"),
            Some("needle_rest"),
        ]);

        assert_eq!(
            referenced_station_ids(&sector),
            vec!["needle_rest".to_string(), "outer_lock".to_string()]
        );
    }

    #[test]
    fn missing_referenced_station_creates_fixed_id_entry() {
        let sector = sector_with_station_refs(&[Some("missing_station")]);
        let mut stations = StationCatalogResource(StationCatalog {
            stations: vec![station_definition("unreferenced")],
        });
        let mut editor_state = StationEditorState::default();

        ensure_selected_station_reference(&sector, &mut stations, &mut editor_state);

        let selected = &stations.0.stations[editor_state.selected_station_index];
        assert_eq!(selected.id, "missing_station");
        assert_eq!(selected.name, "missing_station");
    }

    #[test]
    fn unreferenced_station_entries_are_not_navigable() {
        let sector = sector_with_station_refs(&[Some("needle_rest"), Some("outer_lock")]);
        let mut stations = StationCatalogResource(StationCatalog {
            stations: vec![
                station_definition("unreferenced"),
                station_definition("needle_rest"),
                station_definition("outer_lock"),
            ],
        });
        let mut editor_state = StationEditorState::default();

        ensure_selected_station_reference(&sector, &mut stations, &mut editor_state);
        assert_eq!(
            stations.0.stations[editor_state.selected_station_index].id,
            "needle_rest"
        );

        cycle_referenced_station(&sector, &mut stations, &mut editor_state, 1);
        assert_eq!(
            stations.0.stations[editor_state.selected_station_index].id,
            "outer_lock"
        );

        cycle_referenced_station(&sector, &mut stations, &mut editor_state, 1);
        assert_eq!(
            stations.0.stations[editor_state.selected_station_index].id,
            "needle_rest"
        );
    }

    #[test]
    fn station_reference_text_reports_unreferenced_catalog_entries() {
        let sector = sector_with_station_refs(&[Some("needle_rest")]);
        let stations = StationCatalogResource(StationCatalog {
            stations: vec![
                station_definition("needle_rest"),
                station_definition("unused_station"),
            ],
        });

        let text = station_config_references_text(&sector, &stations);

        assert!(text.contains("needle_rest"));
        assert!(text.contains("unused_station - unused_station display [unreferenced]"));
    }

    #[test]
    fn station_tools_place_npc_spawnpoint_components() {
        let mut editor_state = StationEditorState {
            selected_tool: Some(StationEditorTool::EngineerNpc),
            ..Default::default()
        };
        let mut editor_ship = EditorShip {
            ship: ShipDefinition::empty("Station"),
        };

        assert!(apply_station_editor_build_action(
            &mut editor_state,
            &mut editor_ship,
            2,
            3,
            1,
            false,
        ));

        let module = editor_ship.ship.module_at(2, 3).unwrap();
        assert_eq!(module.kind, ModuleKind::Interior);
        assert_eq!(module.variant, ModuleVariant::StationEngineerNpc);
        assert_eq!(module.rotation_quadrants, 1);
        assert_eq!(
            station_npc_sprite_path_for_module(module),
            Some("actors/station_engineer.png")
        );
    }

    #[test]
    fn station_tool_sprites_use_unique_npc_assets() {
        assert_eq!(
            station_tool_sprite_path(StationEditorTool::EngineerNpc),
            "actors/station_engineer.png"
        );
        assert_eq!(
            station_tool_sprite_path(StationEditorTool::ContractsNpc),
            "actors/station_contracts.png"
        );
        assert_eq!(
            station_tool_sprite_path(StationEditorTool::ArchivesNpc),
            "actors/station_archives.png"
        );
    }
}

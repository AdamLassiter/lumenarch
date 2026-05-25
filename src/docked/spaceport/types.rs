use super::*;

#[derive(Component)]
pub(crate) struct DockedSpaceportRoot;

#[derive(Component)]
pub(crate) struct DockedLocalAvatar;

#[derive(Component)]
pub(crate) struct DockedFocusMarker;

#[derive(Component)]
pub(super) struct DockedNpcMarker;

#[derive(Component)]
pub(super) struct DockedShipTileMarker;

#[derive(Component)]
pub(crate) struct DockedDialogueRoot;

#[derive(Component)]
pub(crate) struct DockedDialogueCloseButton;

#[derive(Component)]
pub(crate) struct DockedDialogueRepairButton;

#[derive(Component)]
pub(crate) struct DockedDialogueRefitButton;

#[derive(Component)]
pub(crate) struct DockedDialogueContractsButton;

#[derive(Component)]
pub(crate) struct DockedDialogueArchivesButton;

#[derive(Component)]
pub(crate) struct DockedDialogueYarnOptionButton {
    pub(super) option_id: OptionId,
    pub(super) label: String,
}

#[derive(Component)]
pub(crate) struct DockedYarnRunner;

#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub(crate) struct DockedAvatarMemory {
    pub(super) by_handle: BTreeMap<PlayerHandle, DockedAvatarState>,
}

impl DockedAvatarMemory {
    pub(super) fn restore_for(
        &self,
        local_handle: Option<&LocalPlayerHandle>,
    ) -> Option<DockedAvatarState> {
        self.by_handle
            .get(&docked_avatar_memory_handle(local_handle))
            .copied()
    }

    pub(super) fn save_for(
        &mut self,
        local_handle: Option<&LocalPlayerHandle>,
        avatar: DockedAvatarState,
    ) {
        self.by_handle
            .insert(docked_avatar_memory_handle(local_handle), avatar);
    }
}

#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct DockedDialogueState {
    pub(crate) active_npc_id: Option<String>,
    pub(crate) selected_surface_hint: Option<StationNpcServiceAction>,
    pub(crate) active_yarn_node: Option<String>,
    pub(crate) runner_entity: Option<Entity>,
    pub(crate) yarn_speaker: Option<String>,
    pub(crate) yarn_line: Option<String>,
    pub(crate) yarn_options: Vec<DockedYarnOption>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DockedYarnOption {
    pub(super) option_id: OptionId,
    pub(super) label: String,
    pub(super) is_available: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DockedAvatarState {
    pub(super) grid_x: i32,
    pub(super) grid_y: i32,
    pub(super) facing_x: i32,
    pub(super) facing_y: i32,
    pub(super) local_position: Vec2,
    pub(super) local_velocity: Vec2,
    pub(super) facing_radians: f32,
}

#[derive(Clone, Debug)]
pub(super) struct DockedStationNpc {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) role: String,
    pub(super) grid_x: i32,
    pub(super) grid_y: i32,
    pub(super) dialogue_start_node: String,
    pub(super) service_action: StationNpcServiceAction,
}

#[derive(Resource, Clone, Debug, PartialEq)]
pub(crate) struct DockedSpaceportState {
    pub(super) avatar: DockedAvatarState,
    pub(super) ship_offset_x: i32,
    pub(super) ship_offset_y: i32,
    pub(super) ship_walkable_tiles: Vec<(i32, i32)>,
    pub(super) cockpit_tile: Option<(i32, i32)>,
}

impl Default for DockedSpaceportState {
    fn default() -> Self {
        Self {
            avatar: DockedAvatarState {
                grid_x: -4,
                grid_y: 0,
                facing_x: 1,
                facing_y: 0,
                local_position: grid_position(-4, 0),
                local_velocity: Vec2::ZERO,
                facing_radians: 0.0,
            },
            ship_offset_x: 0,
            ship_offset_y: 0,
            ship_walkable_tiles: Vec::new(),
            cockpit_tile: None,
        }
    }
}

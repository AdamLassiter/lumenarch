use super::*;

#[derive(Clone)]
pub(super) struct ModuleSnapshot {
    pub(super) module_id: u64,
    pub(super) kind: ModuleKind,
    pub(super) grid_x: i32,
    pub(super) grid_y: i32,
    pub(super) power_draw: Option<i32>,
    pub(super) producer_output: Option<i32>,
    pub(super) reactor_output: Option<crate::helpers::Fx>,
    pub(super) storage: Option<StorageSnapshot>,
    pub(super) processor: bool,
    pub(super) weapon_requires_ammo: bool,
    pub(super) destroyed: bool,
    pub(super) junction_open: Option<bool>,
    pub(super) valve_open: Option<bool>,
}

#[derive(Clone, Copy)]
pub(super) struct AttachedNetwork {
    pub(super) id: u32,
    pub(super) service_coord: (i32, i32),
}

#[derive(Clone, Copy)]
pub(super) struct StorageSnapshot {
    pub(super) accepts_fuel: bool,
    pub(super) accepts_ammunition: bool,
    pub(super) accepts_general: bool,
    pub(super) accepts_oxygen: bool,
    pub(super) fuel: u32,
    pub(super) ammunition: u32,
    pub(super) raw_salvage: u32,
    pub(super) repair_charge: u32,
    pub(super) oxygen: u32,
}

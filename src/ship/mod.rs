pub mod arch;
pub mod enemy;
pub mod storage;

use std::fmt;

use serde::{Deserialize, Serialize};

use self::arch::{ArchProgram, ArchProgramTemplate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleKind {
    Core,
    Interior,
    Cockpit,
    Computer,
    Processor,
    Reactor,
    Engine,
    Cargo,
    Battery,
    Airlock,
    Turret,
    Hull,
    HullInnerCorner,
    HullOuterCorner,
}

impl ModuleKind {
    pub const ALL: [Self; 14] = [
        Self::Core,
        Self::Interior,
        Self::Cockpit,
        Self::Computer,
        Self::Processor,
        Self::Reactor,
        Self::Engine,
        Self::Cargo,
        Self::Battery,
        Self::Airlock,
        Self::Turret,
        Self::Hull,
        Self::HullInnerCorner,
        Self::HullOuterCorner,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Interior => "interior",
            Self::Cockpit => "cockpit",
            Self::Computer => "computer",
            Self::Processor => "processor",
            Self::Reactor => "reactor",
            Self::Engine => "engine",
            Self::Cargo => "cargo",
            Self::Battery => "battery",
            Self::Airlock => "airlock",
            Self::Turret => "turret",
            Self::Hull => "hull",
            Self::HullInnerCorner => "hull_inner_corner",
            Self::HullOuterCorner => "hull_outer_corner",
        }
    }
}

impl fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipModule {
    pub id: u64,
    pub kind: ModuleKind,
    pub grid_x: i32,
    pub grid_y: i32,
    pub rotation_quadrants: u8,
    #[serde(default)]
    pub arch_program: Option<ArchProgram>,
}

impl ShipModule {
    pub fn new(
        id: u64,
        kind: ModuleKind,
        grid_x: i32,
        grid_y: i32,
        rotation_quadrants: u8,
    ) -> Self {
        Self {
            id,
            kind,
            grid_x,
            grid_y,
            rotation_quadrants: rotation_quadrants % 4,
            arch_program: (kind == ModuleKind::Computer)
                .then(|| ArchProgram::from_template(ArchProgramTemplate::BalancedOps)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipDefinition {
    pub name: String,
    pub modules: Vec<ShipModule>,
}

#[allow(dead_code)]
impl ShipDefinition {
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            modules: Vec::new(),
        }
    }

    pub fn module_at(&self, grid_x: i32, grid_y: i32) -> Option<&ShipModule> {
        self.modules
            .iter()
            .find(|module| module.grid_x == grid_x && module.grid_y == grid_y)
    }

    pub fn module_at_mut(&mut self, grid_x: i32, grid_y: i32) -> Option<&mut ShipModule> {
        self.modules
            .iter_mut()
            .find(|module| module.grid_x == grid_x && module.grid_y == grid_y)
    }

    pub fn replace_module(&mut self, module: ShipModule) {
        if let Some(existing) = self.module_at_mut(module.grid_x, module.grid_y) {
            *existing = module;
        } else {
            self.modules.push(module);
        }
    }

    pub fn remove_module_at(&mut self, grid_x: i32, grid_y: i32) {
        self.modules
            .retain(|module| !(module.grid_x == grid_x && module.grid_y == grid_y));
    }

    pub fn bounds(&self) -> Option<(i32, i32, i32, i32)> {
        let mut modules = self.modules.iter();
        let first = modules.next()?;

        let mut min_x = first.grid_x;
        let mut max_x = first.grid_x;
        let mut min_y = first.grid_y;
        let mut max_y = first.grid_y;

        for module in modules {
            min_x = min_x.min(module.grid_x);
            max_x = max_x.max(module.grid_x);
            min_y = min_y.min(module.grid_y);
            max_y = max_y.max(module.grid_y);
        }

        Some((min_x, max_x, min_y, max_y))
    }

    pub fn next_module_id(&self) -> u64 {
        self.modules
            .iter()
            .map(|module| module.id)
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn has_module_kind(&self, kind: ModuleKind) -> bool {
        self.modules.iter().any(|module| module.kind == kind)
    }

    pub fn validate_required_modules(&self) -> bool {
        self.has_module_kind(ModuleKind::Core) && self.has_module_kind(ModuleKind::Cockpit)
    }
}

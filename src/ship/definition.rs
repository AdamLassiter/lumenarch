use serde::{Deserialize, Serialize};

use super::{
    ArchProgram,
    ArchProgramTemplate,
    LumenProgram,
    LumenProgramTemplate,
    ModuleDefaultState,
    ModuleKind,
    ModuleSpec,
    ModuleVariant,
    ShipFoundationKind,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipModule {
    pub id: u64,
    pub kind: ModuleKind,
    #[serde(default)]
    pub variant: ModuleVariant,
    pub grid_x: i32,
    pub grid_y: i32,
    pub rotation_quadrants: u8,
    #[serde(default)]
    pub channel: u8,
    #[serde(default)]
    pub defaults: ModuleDefaultState,
    #[serde(default)]
    pub arch_program: Option<ArchProgram>,
    #[serde(default)]
    pub lumen_program: Option<LumenProgram>,
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
            variant: ModuleVariant::default_for_kind(kind),
            grid_x,
            grid_y,
            rotation_quadrants: rotation_quadrants % 4,
            channel: 0,
            defaults: ModuleDefaultState::default(),
            arch_program: (kind == ModuleKind::Computer)
                .then(|| ArchProgram::from_template(ArchProgramTemplate::BalancedOps)),
            lumen_program: (kind == ModuleKind::Computer)
                .then(|| LumenProgram::from_template(LumenProgramTemplate::BalancedSupervision)),
        }
    }

    pub fn display_name(&self) -> String {
        let family = self.kind.as_str().replace('_', " ");
        let family = family
            .split(' ')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        if self.variant == ModuleVariant::default_for_kind(self.kind)
            && ModuleVariant::supported_for_kind(self.kind).len() == 1
        {
            family
        } else {
            format!("{family} / {}", self.variant.display_name())
        }
    }

    pub fn effective_channel(&self) -> u8 {
        if self.kind.supports_channel() {
            self.channel % 10
        } else {
            0
        }
    }

    pub fn clamped_defaults(&mut self) {
        self.defaults.reaction_rate_milli = self.defaults.reaction_rate_milli.min(1000);
        self.defaults.turbine_load_milli = self.defaults.turbine_load_milli.min(1000);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShipFoundationTile {
    pub id: u64,
    pub kind: ShipFoundationKind,
    pub grid_x: i32,
    pub grid_y: i32,
    pub rotation_quadrants: u8,
}

impl ShipFoundationTile {
    pub fn new(
        id: u64,
        kind: ShipFoundationKind,
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
        }
    }

    pub fn display_name(&self) -> &'static str {
        self.kind.display_name()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShipDefinition {
    pub name: String,
    #[serde(default)]
    pub foundation_tiles: Vec<ShipFoundationTile>,
    #[serde(default)]
    pub hull_tiles: Vec<ShipFoundationTile>,
    pub modules: Vec<ShipModule>,
}

#[allow(dead_code)]
impl ShipDefinition {
    pub fn empty(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            foundation_tiles: Vec::new(),
            hull_tiles: Vec::new(),
            modules: Vec::new(),
        }
    }

    pub fn core_only(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            foundation_tiles: vec![ShipFoundationTile::new(
                1,
                ShipFoundationKind::Floor,
                0,
                0,
                0,
            )],
            hull_tiles: Vec::new(),
            modules: vec![ShipModule::new(1, ModuleKind::Core, 0, 0, 0)],
        }
    }

    pub fn logistics_at(&self, grid_x: i32, grid_y: i32) -> Option<&ShipFoundationTile> {
        self.foundation_tiles
            .iter()
            .find(|tile| tile.grid_x == grid_x && tile.grid_y == grid_y)
    }

    pub fn logistics_at_mut(
        &mut self,
        grid_x: i32,
        grid_y: i32,
    ) -> Option<&mut ShipFoundationTile> {
        self.foundation_tiles
            .iter_mut()
            .find(|tile| tile.grid_x == grid_x && tile.grid_y == grid_y)
    }

    pub fn replace_logistics_tile(&mut self, tile: ShipFoundationTile) {
        if let Some(existing) = self.logistics_at_mut(tile.grid_x, tile.grid_y) {
            *existing = tile;
        } else {
            self.foundation_tiles.push(tile);
        }
    }

    pub fn remove_logistics_at(&mut self, grid_x: i32, grid_y: i32) {
        self.foundation_tiles
            .retain(|tile| !(tile.grid_x == grid_x && tile.grid_y == grid_y));
    }

    pub fn hull_at(&self, grid_x: i32, grid_y: i32) -> Option<&ShipFoundationTile> {
        self.hull_tiles
            .iter()
            .find(|tile| tile.grid_x == grid_x && tile.grid_y == grid_y)
    }

    pub fn hull_at_mut(&mut self, grid_x: i32, grid_y: i32) -> Option<&mut ShipFoundationTile> {
        self.hull_tiles
            .iter_mut()
            .find(|tile| tile.grid_x == grid_x && tile.grid_y == grid_y)
    }

    pub fn replace_hull_tile(&mut self, tile: ShipFoundationTile) {
        if let Some(existing) = self.hull_at_mut(tile.grid_x, tile.grid_y) {
            *existing = tile;
        } else {
            self.hull_tiles.push(tile);
        }
    }

    pub fn remove_hull_at(&mut self, grid_x: i32, grid_y: i32) {
        self.hull_tiles
            .retain(|tile| !(tile.grid_x == grid_x && tile.grid_y == grid_y));
    }

    pub fn foundation_at(&self, grid_x: i32, grid_y: i32) -> Option<&ShipFoundationTile> {
        self.logistics_at(grid_x, grid_y)
    }

    pub fn foundation_at_mut(
        &mut self,
        grid_x: i32,
        grid_y: i32,
    ) -> Option<&mut ShipFoundationTile> {
        self.logistics_at_mut(grid_x, grid_y)
    }

    pub fn replace_foundation_tile(&mut self, tile: ShipFoundationTile) {
        self.replace_logistics_tile(tile);
    }

    pub fn remove_foundation_at(&mut self, grid_x: i32, grid_y: i32) {
        self.remove_logistics_at(grid_x, grid_y);
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
        let mut points = self
            .modules
            .iter()
            .map(|module| (module.grid_x, module.grid_y))
            .chain(
                self.foundation_tiles
                    .iter()
                    .map(|tile| (tile.grid_x, tile.grid_y)),
            )
            .chain(
                self.hull_tiles
                    .iter()
                    .map(|tile| (tile.grid_x, tile.grid_y)),
            );
        let first = points.next()?;

        let mut min_x = first.0;
        let mut max_x = first.0;
        let mut min_y = first.1;
        let mut max_y = first.1;

        for (grid_x, grid_y) in points {
            min_x = min_x.min(grid_x);
            max_x = max_x.max(grid_x);
            min_y = min_y.min(grid_y);
            max_y = max_y.max(grid_y);
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

    pub fn next_foundation_id(&self) -> u64 {
        self.foundation_tiles
            .iter()
            .chain(self.hull_tiles.iter())
            .map(|tile| tile.id)
            .max()
            .unwrap_or(0)
            + 1
    }

    pub fn has_module_kind(&self, kind: ModuleKind) -> bool {
        self.modules.iter().any(|module| module.kind == kind)
    }

    pub fn module_channel(&self, module_id: u64) -> Option<u8> {
        self.modules
            .iter()
            .find(|module| module.id == module_id)
            .map(ShipModule::effective_channel)
    }

    pub fn set_module_channel(&mut self, module_id: u64, channel: u8) -> bool {
        let Some(module) = self
            .modules
            .iter_mut()
            .find(|module| module.id == module_id)
        else {
            return false;
        };
        if !module.kind.supports_channel() {
            return false;
        }
        module.channel = channel % 10;
        true
    }

    pub fn modules_by_channel(
        &self,
        kind: ModuleKind,
        channel: u8,
    ) -> impl Iterator<Item = &ShipModule> {
        let channel = channel % 10;
        self.modules.iter().filter(move |module| {
            module.kind == kind
                && module.kind.supports_channel()
                && module.effective_channel() == channel
        })
    }

    pub fn normalize_variants(&mut self) {
        for module in &mut self.modules {
            module.variant = module.variant.normalize_for_kind(module.kind);
            module.channel = module.effective_channel();
            module.clamped_defaults();
            if let Some(program) = &mut module.arch_program
                && program.source_text.trim().is_empty()
            {
                program.refresh_source_text();
            }
            if let Some(program) = &mut module.lumen_program
                && program.source_text.trim().is_empty()
            {
                program.refresh_source_text();
            }
        }
    }

    pub fn core_module_capacity(&self) -> u32 {
        self.modules
            .iter()
            .filter(|module| module.kind == ModuleKind::Core)
            .map(|module| ModuleSpec::for_module(module.kind, module.variant).core_capacity_modules)
            .max()
            .unwrap_or(
                ModuleSpec::for_module(ModuleKind::Core, ModuleVariant::BasicCore)
                    .core_capacity_modules,
            )
    }

    pub fn fits_core_capacity(&self) -> bool {
        self.modules.len() as u32 <= self.core_module_capacity()
    }

    pub fn validate_required_modules(&self) -> bool {
        self.has_module_kind(ModuleKind::Core)
            && self.has_module_kind(ModuleKind::Airlock)
            && self.fits_core_capacity()
    }

    pub fn has_docking_airlock(&self) -> bool {
        self.has_module_kind(ModuleKind::Airlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_modules_include_airlock_for_docking() {
        let mut ship = ShipDefinition::core_only("No Dock");
        assert!(!ship.validate_required_modules());

        ship.replace_module(ShipModule::new(2, ModuleKind::Airlock, 1, 0, 0));
        assert!(ship.validate_required_modules());
    }
}

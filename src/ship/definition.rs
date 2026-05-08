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
            && self.has_module_kind(ModuleKind::Cockpit)
            && self.fits_core_capacity()
    }
}

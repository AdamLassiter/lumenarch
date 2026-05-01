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
    Shield,
    Hull,
    HullInnerCorner,
    HullOuterCorner,
}

impl ModuleKind {
    pub const ALL: [Self; 15] = [
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
        Self::Shield,
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
            Self::Shield => "shield",
            Self::Hull => "hull",
            Self::HullInnerCorner => "hull_inner_corner",
            Self::HullOuterCorner => "hull_outer_corner",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ModuleVariant {
    #[default]
    Standard,
    AdvancedHelm,
    Fission,
    Fusion,
    GeneralCargo,
    FuelTank,
    AmmoRack,
    BatteryCell,
    Capacitor,
    LaserTurret,
    BallisticTurret,
    BasicCore,
    ExpandedCore,
    FabricatorSlow,
    FabricatorFast,
    RadialShield,
    DirectionalShield,
}

impl ModuleVariant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::AdvancedHelm => "advanced_helm",
            Self::Fission => "fission",
            Self::Fusion => "fusion",
            Self::GeneralCargo => "general_cargo",
            Self::FuelTank => "fuel_tank",
            Self::AmmoRack => "ammo_rack",
            Self::BatteryCell => "battery_cell",
            Self::Capacitor => "capacitor",
            Self::LaserTurret => "laser_turret",
            Self::BallisticTurret => "ballistic_turret",
            Self::BasicCore => "basic_core",
            Self::ExpandedCore => "expanded_core",
            Self::FabricatorSlow => "fabricator_slow",
            Self::FabricatorFast => "fabricator_fast",
            Self::RadialShield => "radial_shield",
            Self::DirectionalShield => "directional_shield",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Standard => "Standard",
            Self::AdvancedHelm => "Advanced Helm",
            Self::Fission => "Fission",
            Self::Fusion => "Fusion",
            Self::GeneralCargo => "General Cargo",
            Self::FuelTank => "Fuel Tank",
            Self::AmmoRack => "Ammo Rack",
            Self::BatteryCell => "Battery Cell",
            Self::Capacitor => "Capacitor",
            Self::LaserTurret => "Laser Turret",
            Self::BallisticTurret => "Ballistic Turret",
            Self::BasicCore => "Basic Core",
            Self::ExpandedCore => "Expanded Core",
            Self::FabricatorSlow => "Slow Fabricator",
            Self::FabricatorFast => "Fast Fabricator",
            Self::RadialShield => "Radial Shield",
            Self::DirectionalShield => "Directional Shield",
        }
    }

    pub fn default_for_kind(kind: ModuleKind) -> Self {
        match kind {
            ModuleKind::Core => Self::BasicCore,
            ModuleKind::Cockpit => Self::Standard,
            ModuleKind::Processor => Self::FabricatorSlow,
            ModuleKind::Reactor => Self::Fission,
            ModuleKind::Cargo => Self::GeneralCargo,
            ModuleKind::Battery => Self::BatteryCell,
            ModuleKind::Turret => Self::LaserTurret,
            ModuleKind::Shield => Self::RadialShield,
            _ => Self::Standard,
        }
    }

    pub fn supported_for_kind(kind: ModuleKind) -> &'static [Self] {
        use ModuleVariant::*;
        match kind {
            ModuleKind::Core => &[BasicCore, ExpandedCore],
            ModuleKind::Cockpit => &[Standard, AdvancedHelm],
            ModuleKind::Processor => &[FabricatorSlow, FabricatorFast],
            ModuleKind::Reactor => &[Fission, Fusion],
            ModuleKind::Cargo => &[GeneralCargo, FuelTank, AmmoRack],
            ModuleKind::Battery => &[BatteryCell, Capacitor],
            ModuleKind::Turret => &[LaserTurret, BallisticTurret],
            ModuleKind::Shield => &[RadialShield, DirectionalShield],
            _ => &[Standard],
        }
    }

    pub fn normalize_for_kind(self, kind: ModuleKind) -> Self {
        if Self::supported_for_kind(kind).contains(&self) {
            self
        } else {
            Self::default_for_kind(kind)
        }
    }

    pub fn cycle_for_kind(self, kind: ModuleKind, direction: i32) -> Self {
        let variants = Self::supported_for_kind(kind);
        let index = variants
            .iter()
            .position(|variant| *variant == self)
            .unwrap_or(0);
        let next = ((index as i32 + direction).rem_euclid(variants.len() as i32)) as usize;
        variants[next]
    }
}

impl fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModuleSpec {
    pub integrity: i32,
    pub placement_cost: u32,
    pub storage_capacity: u32,
    pub engine_multiplier: f32,
    pub helm_multiplier: f32,
    pub reactor_output_multiplier: f32,
    pub battery_capacity_multiplier: f32,
    pub battery_flow_multiplier: f32,
    pub core_capacity_modules: u32,
    pub inertia_multiplier: f32,
    pub processor_speed_multiplier: f32,
    pub ammo_per_shot: u32,
    pub projectile_damage: i32,
    pub projectile_speed_multiplier: f32,
    pub weapon_cooldown_multiplier: f32,
    pub fuel_demand_multiplier: f32,
    pub reactor_heat_multiplier: f32,
    pub shield_capacity: f32,
    pub shield_arc_degrees: f32,
    pub shield_regen: f32,
}

impl ModuleSpec {
    pub fn for_module(kind: ModuleKind, variant: ModuleVariant) -> Self {
        let variant = variant.normalize_for_kind(kind);
        let mut spec = match kind {
            ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner => Self {
                integrity: 12,
                placement_cost: 1,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Core => Self {
                integrity: 20,
                placement_cost: 6,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 24,
                inertia_multiplier: 1.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Cockpit => Self {
                integrity: 10,
                placement_cost: 4,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 1.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Computer => Self {
                integrity: 8,
                placement_cost: 4,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Processor => Self {
                integrity: 8,
                placement_cost: 4,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 1.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Reactor => Self {
                integrity: 14,
                placement_cost: 5,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 1.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 1.0,
                reactor_heat_multiplier: 1.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Engine => Self {
                integrity: 10,
                placement_cost: 3,
                storage_capacity: 0,
                engine_multiplier: 1.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Cargo => Self {
                integrity: 10,
                placement_cost: 2,
                storage_capacity: 8,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Battery => Self {
                integrity: 8,
                placement_cost: 2,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 1.0,
                battery_flow_multiplier: 1.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Airlock => Self {
                integrity: 8,
                placement_cost: 2,
                storage_capacity: 4,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Turret => Self {
                integrity: 8,
                placement_cost: 4,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 3,
                projectile_speed_multiplier: 1.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
            ModuleKind::Shield => Self {
                integrity: 10,
                placement_cost: 5,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 12.0,
                shield_arc_degrees: 360.0,
                shield_regen: 1.4,
            },
            ModuleKind::Interior => Self {
                integrity: 6,
                placement_cost: 0,
                storage_capacity: 0,
                engine_multiplier: 0.0,
                helm_multiplier: 0.0,
                reactor_output_multiplier: 0.0,
                battery_capacity_multiplier: 0.0,
                battery_flow_multiplier: 0.0,
                core_capacity_modules: 0,
                inertia_multiplier: 0.0,
                processor_speed_multiplier: 0.0,
                ammo_per_shot: 0,
                projectile_damage: 0,
                projectile_speed_multiplier: 0.0,
                weapon_cooldown_multiplier: 1.0,
                fuel_demand_multiplier: 0.0,
                reactor_heat_multiplier: 0.0,
                shield_capacity: 0.0,
                shield_arc_degrees: 0.0,
                shield_regen: 0.0,
            },
        };

        match variant {
            ModuleVariant::AdvancedHelm => {
                spec.placement_cost = 6;
                spec.integrity = 12;
                spec.helm_multiplier = 1.3;
            }
            ModuleVariant::Fusion => {
                spec.placement_cost = 7;
                spec.integrity = 16;
                spec.reactor_output_multiplier = 1.6;
                spec.fuel_demand_multiplier = 1.4;
                spec.reactor_heat_multiplier = 1.35;
            }
            ModuleVariant::FuelTank => {
                spec.placement_cost = 3;
                spec.storage_capacity = 10;
            }
            ModuleVariant::AmmoRack => {
                spec.placement_cost = 3;
                spec.storage_capacity = 8;
            }
            ModuleVariant::Capacitor => {
                spec.placement_cost = 3;
                spec.integrity = 6;
                spec.battery_capacity_multiplier = 0.45;
                spec.battery_flow_multiplier = 3.0;
            }
            ModuleVariant::BallisticTurret => {
                spec.placement_cost = 6;
                spec.projectile_damage = 7;
                spec.projectile_speed_multiplier = 0.8;
                spec.weapon_cooldown_multiplier = 1.7;
                spec.ammo_per_shot = 1;
            }
            ModuleVariant::ExpandedCore => {
                spec.placement_cost = 9;
                spec.integrity = 26;
                spec.core_capacity_modules = 48;
                spec.inertia_multiplier = 1.35;
            }
            ModuleVariant::FabricatorFast => {
                spec.placement_cost = 6;
                spec.processor_speed_multiplier = 1.75;
            }
            ModuleVariant::RadialShield => {
                spec.shield_capacity = 12.0;
                spec.shield_arc_degrees = 360.0;
                spec.shield_regen = 1.1;
            }
            ModuleVariant::DirectionalShield => {
                spec.placement_cost = 7;
                spec.shield_capacity = 18.0;
                spec.shield_arc_degrees = 120.0;
                spec.shield_regen = 0.9;
            }
            _ => {}
        }

        spec
    }
}

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
            variant: ModuleVariant::default_for_kind(kind),
            grid_x,
            grid_y,
            rotation_quadrants: rotation_quadrants % 4,
            arch_program: (kind == ModuleKind::Computer)
                .then(|| ArchProgram::from_template(ArchProgramTemplate::BalancedOps)),
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

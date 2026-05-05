pub mod arch;
pub mod enemy;
pub mod lumen;
pub mod storage;

use std::fmt;

use serde::{Deserialize, Serialize};

use self::{
    arch::{ArchProgram, ArchProgramTemplate},
    lumen::{LumenProgram, LumenProgramTemplate},
};

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

    pub fn supports_channel(self) -> bool {
        matches!(
            self,
            Self::Cockpit
                | Self::Computer
                | Self::Processor
                | Self::Reactor
                | Self::Engine
                | Self::Cargo
                | Self::Airlock
                | Self::Turret
                | Self::Shield
        )
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StoredResourceKind {
    #[default]
    RawSalvage,
    RepairCharge,
    Fuel,
    Ammunition,
}

impl StoredResourceKind {
    pub fn next(self) -> Self {
        match self {
            Self::RawSalvage => Self::RepairCharge,
            Self::RepairCharge => Self::Fuel,
            Self::Fuel => Self::Ammunition,
            Self::Ammunition => Self::RawSalvage,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RawSalvage => "Raw Salvage",
            Self::RepairCharge => "Repair Charge",
            Self::Fuel => "Fuel",
            Self::Ammunition => "Ammunition",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StoredProcessorRecipe {
    #[default]
    RepairCharge,
    Ammunition,
    Fuel,
}

impl StoredProcessorRecipe {
    pub fn next(self) -> Self {
        match self {
            Self::RepairCharge => Self::Ammunition,
            Self::Ammunition => Self::Fuel,
            Self::Fuel => Self::RepairCharge,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::RepairCharge => "Repair Charge",
            Self::Ammunition => "Ammunition",
            Self::Fuel => "Fuel",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDefaultState {
    #[serde(default = "default_reaction_rate_milli")]
    pub reaction_rate_milli: u16,
    #[serde(default = "default_turbine_load_milli")]
    pub turbine_load_milli: u16,
    #[serde(default = "default_true")]
    pub storage_allow_intake: bool,
    #[serde(default)]
    pub airlock_open: bool,
    #[serde(default)]
    pub manipulator_manual_mode: bool,
    #[serde(default)]
    pub manipulator_transfer_enabled: bool,
    #[serde(default)]
    pub manipulator_resource_kind: StoredResourceKind,
    #[serde(default)]
    pub processor_recipe: StoredProcessorRecipe,
    #[serde(default = "default_true")]
    pub processor_enabled: bool,
    #[serde(default = "default_true")]
    pub computer_enabled: bool,
    #[serde(default)]
    pub turret_fire_intent: bool,
}

impl Default for ModuleDefaultState {
    fn default() -> Self {
        Self {
            reaction_rate_milli: default_reaction_rate_milli(),
            turbine_load_milli: default_turbine_load_milli(),
            storage_allow_intake: true,
            airlock_open: false,
            manipulator_manual_mode: false,
            manipulator_transfer_enabled: false,
            manipulator_resource_kind: StoredResourceKind::RawSalvage,
            processor_recipe: StoredProcessorRecipe::RepairCharge,
            processor_enabled: true,
            computer_enabled: true,
            turret_fire_intent: false,
        }
    }
}

const fn default_reaction_rate_milli() -> u16 {
    350
}

const fn default_turbine_load_milli() -> u16 {
    300
}

const fn default_true() -> bool {
    true
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
                spec.helm_multiplier = 1.25;
            }
            ModuleVariant::Fusion => {
                spec.placement_cost = 8;
                spec.integrity = 16;
                spec.reactor_output_multiplier = 1.45;
                spec.fuel_demand_multiplier = 1.75;
                spec.reactor_heat_multiplier = 1.55;
            }
            ModuleVariant::FuelTank => {
                spec.placement_cost = 4;
                spec.storage_capacity = 14;
            }
            ModuleVariant::AmmoRack => {
                spec.placement_cost = 4;
                spec.storage_capacity = 6;
            }
            ModuleVariant::Capacitor => {
                spec.placement_cost = 4;
                spec.integrity = 5;
                spec.battery_capacity_multiplier = 0.35;
                spec.battery_flow_multiplier = 4.0;
            }
            ModuleVariant::BallisticTurret => {
                spec.placement_cost = 7;
                spec.projectile_damage = 8;
                spec.projectile_speed_multiplier = 0.78;
                spec.weapon_cooldown_multiplier = 1.9;
                spec.ammo_per_shot = 2;
            }
            ModuleVariant::ExpandedCore => {
                spec.placement_cost = 10;
                spec.integrity = 26;
                spec.core_capacity_modules = 44;
                spec.inertia_multiplier = 1.25;
            }
            ModuleVariant::FabricatorFast => {
                spec.placement_cost = 7;
                spec.processor_speed_multiplier = 1.55;
            }
            ModuleVariant::RadialShield => {
                spec.placement_cost = 5;
                spec.shield_capacity = 10.5;
                spec.shield_arc_degrees = 360.0;
                spec.shield_regen = 1.0;
            }
            ModuleVariant::DirectionalShield => {
                spec.placement_cost = 8;
                spec.shield_capacity = 20.0;
                spec.shield_arc_degrees = 110.0;
                spec.shield_regen = 0.75;
            }
            _ => {}
        }

        spec
    }
}

#[cfg(test)]
mod tests {
    use super::{ModuleKind, ModuleSpec, ModuleVariant};

    #[test]
    fn upgraded_variants_keep_clear_tradeoffs() {
        let basic_core = ModuleSpec::for_module(ModuleKind::Core, ModuleVariant::BasicCore);
        let expanded_core = ModuleSpec::for_module(ModuleKind::Core, ModuleVariant::ExpandedCore);
        assert!(expanded_core.core_capacity_modules > basic_core.core_capacity_modules);
        assert!(expanded_core.placement_cost > basic_core.placement_cost);

        let fission = ModuleSpec::for_module(ModuleKind::Reactor, ModuleVariant::Fission);
        let fusion = ModuleSpec::for_module(ModuleKind::Reactor, ModuleVariant::Fusion);
        assert!(fusion.reactor_output_multiplier > fission.reactor_output_multiplier);
        assert!(fusion.fuel_demand_multiplier > fission.fuel_demand_multiplier);
        assert!(fusion.reactor_heat_multiplier > fission.reactor_heat_multiplier);

        let battery = ModuleSpec::for_module(ModuleKind::Battery, ModuleVariant::BatteryCell);
        let capacitor = ModuleSpec::for_module(ModuleKind::Battery, ModuleVariant::Capacitor);
        assert!(capacitor.battery_capacity_multiplier < battery.battery_capacity_multiplier);
        assert!(capacitor.battery_flow_multiplier > battery.battery_flow_multiplier);

        let laser = ModuleSpec::for_module(ModuleKind::Turret, ModuleVariant::LaserTurret);
        let ballistic = ModuleSpec::for_module(ModuleKind::Turret, ModuleVariant::BallisticTurret);
        assert!(ballistic.projectile_damage > laser.projectile_damage);
        assert!(ballistic.ammo_per_shot > laser.ammo_per_shot);
        assert!(ballistic.weapon_cooldown_multiplier > laser.weapon_cooldown_multiplier);

        let radial = ModuleSpec::for_module(ModuleKind::Shield, ModuleVariant::RadialShield);
        let directional =
            ModuleSpec::for_module(ModuleKind::Shield, ModuleVariant::DirectionalShield);
        assert!(directional.shield_capacity > radial.shield_capacity);
        assert!(directional.shield_arc_degrees < radial.shield_arc_degrees);
        assert!(directional.shield_regen < radial.shield_regen);
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

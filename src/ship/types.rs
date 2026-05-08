use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModuleKind {
    Core,
    Interior,
    Cockpit,
    Computer,
    Detector,
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
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Core => "core",
            Self::Interior => "interior",
            Self::Cockpit => "cockpit",
            Self::Computer => "computer",
            Self::Detector => "detector",
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
                | Self::Detector
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

impl fmt::Display for ModuleKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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
    LifePulse,
    LifeSweep,
    LifeSurvey,
    ShipPing,
    ShipVector,
    ShipSurvey,
    DamageAlarm,
    DamageArray,
    StructuralSurveyor,
    PowerMonitor,
    HeatMonitor,
    LogisticsBeacon,
    RadialShield,
    DirectionalShield,
    DroneBay,
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
            Self::LifePulse => "Life Pulse",
            Self::LifeSweep => "Life Sweep",
            Self::LifeSurvey => "Life Survey",
            Self::ShipPing => "Ship Ping",
            Self::ShipVector => "Ship Vector",
            Self::ShipSurvey => "Ship Survey",
            Self::DamageAlarm => "Damage Alarm",
            Self::DamageArray => "Damage Array",
            Self::StructuralSurveyor => "Structural Surveyor",
            Self::PowerMonitor => "Power Monitor",
            Self::HeatMonitor => "Heat Monitor",
            Self::LogisticsBeacon => "Logistics Beacon",
            Self::RadialShield => "Radial Shield",
            Self::DirectionalShield => "Directional Shield",
            Self::DroneBay => "Drone Bay",
        }
    }

    pub fn default_for_kind(kind: ModuleKind) -> Self {
        match kind {
            ModuleKind::Core => Self::BasicCore,
            ModuleKind::Cockpit => Self::Standard,
            ModuleKind::Detector => Self::LifePulse,
            ModuleKind::Processor => Self::FabricatorSlow,
            ModuleKind::Reactor => Self::Fission,
            ModuleKind::Cargo => Self::GeneralCargo,
            ModuleKind::Battery => Self::BatteryCell,
            ModuleKind::Turret => Self::LaserTurret,
            ModuleKind::Shield => Self::RadialShield,
            ModuleKind::Airlock => Self::Standard,
            _ => Self::Standard,
        }
    }

    pub fn supported_for_kind(kind: ModuleKind) -> &'static [Self] {
        use ModuleVariant::*;
        match kind {
            ModuleKind::Core => &[BasicCore, ExpandedCore],
            ModuleKind::Cockpit => &[Standard, AdvancedHelm],
            ModuleKind::Detector => &[
                LifePulse,
                LifeSweep,
                LifeSurvey,
                ShipPing,
                ShipVector,
                ShipSurvey,
                DamageAlarm,
                DamageArray,
                StructuralSurveyor,
                PowerMonitor,
                HeatMonitor,
                LogisticsBeacon,
            ],
            ModuleKind::Processor => &[FabricatorSlow, FabricatorFast],
            ModuleKind::Reactor => &[Fission, Fusion],
            ModuleKind::Cargo => &[GeneralCargo, FuelTank, AmmoRack],
            ModuleKind::Battery => &[BatteryCell, Capacitor],
            ModuleKind::Turret => &[LaserTurret, BallisticTurret],
            ModuleKind::Shield => &[RadialShield, DirectionalShield],
            ModuleKind::Airlock => &[Standard, DroneBay],
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

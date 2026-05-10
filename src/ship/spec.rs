use super::{ModuleKind, ModuleVariant};

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
            ModuleKind::Detector => Self {
                integrity: 7,
                placement_cost: 3,
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
            ModuleKind::Interior
            | ModuleKind::InteriorWall
            | ModuleKind::JunctionBox
            | ModuleKind::Valve
            | ModuleKind::O2Generator => Self {
                integrity: 6,
                placement_cost: if matches!(kind, ModuleKind::Interior) {
                    0
                } else {
                    2
                },
                storage_capacity: if matches!(kind, ModuleKind::Valve | ModuleKind::O2Generator) {
                    4
                } else {
                    0
                },
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
            ModuleVariant::RawSalvageCrate | ModuleVariant::RepairChargeRack => {
                spec.placement_cost = 3;
                spec.storage_capacity = 10;
            }
            ModuleVariant::O2Canister => {
                spec.placement_cost = 4;
                spec.storage_capacity = 12;
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
            ModuleVariant::LifeSweep | ModuleVariant::ShipVector | ModuleVariant::DamageArray => {
                spec.placement_cost = 5;
                spec.integrity = 8;
            }
            ModuleVariant::LifeSurvey
            | ModuleVariant::ShipSurvey
            | ModuleVariant::StructuralSurveyor => {
                spec.placement_cost = 7;
                spec.integrity = 9;
            }
            ModuleVariant::PowerMonitor
            | ModuleVariant::HeatMonitor
            | ModuleVariant::LogisticsBeacon => {
                spec.placement_cost = 4;
                spec.integrity = 8;
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
            ModuleVariant::DroneBay => {
                spec.placement_cost = 6;
                spec.integrity = 10;
                spec.storage_capacity = 6;
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

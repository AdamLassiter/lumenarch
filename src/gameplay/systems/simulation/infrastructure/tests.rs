use std::collections::BTreeMap;

use super::{ModuleSnapshot, StorageSnapshot, build_infrastructure_state};
use crate::{
    gameplay::components::{InfrastructureRouteKind, ResourceKind},
    helpers::Fx,
    ship::ModuleKind,
};

fn module(module_id: u64, kind: ModuleKind, grid_x: i32, grid_y: i32) -> ModuleSnapshot {
    ModuleSnapshot {
        module_id,
        kind,
        grid_x,
        grid_y,
        power_draw: None,
        producer_output: None,
        reactor_output: None,
        storage: None,
        processor: false,
        weapon_requires_ammo: false,
        destroyed: false,
        junction_open: None,
        valve_open: None,
    }
}

fn powered_turret(grid_x: i32, grid_y: i32) -> ModuleSnapshot {
    ModuleSnapshot {
        power_draw: Some(2),
        weapon_requires_ammo: true,
        ..module(1, ModuleKind::Turret, grid_x, grid_y)
    }
}

fn reactor(module_id: u64, grid_x: i32, grid_y: i32) -> ModuleSnapshot {
    ModuleSnapshot {
        reactor_output: Some(Fx::from_num(10)),
        ..module(module_id, ModuleKind::Reactor, grid_x, grid_y)
    }
}

fn ammunition_storage(module_id: u64, grid_x: i32, grid_y: i32, ammunition: u32) -> ModuleSnapshot {
    ModuleSnapshot {
        storage: Some(StorageSnapshot {
            accepts_fuel: false,
            accepts_ammunition: true,
            accepts_general: false,
            accepts_oxygen: false,
            fuel: 0,
            ammunition,
            raw_salvage: 0,
            repair_charge: 0,
            oxygen: 0,
        }),
        ..module(module_id, ModuleKind::Cargo, grid_x, grid_y)
    }
}

fn fuel_storage(module_id: u64, grid_x: i32, grid_y: i32, fuel: u32) -> ModuleSnapshot {
    ModuleSnapshot {
        storage: Some(StorageSnapshot {
            accepts_fuel: true,
            accepts_ammunition: false,
            accepts_general: false,
            accepts_oxygen: false,
            fuel,
            ammunition: 0,
            raw_salvage: 0,
            repair_charge: 0,
            oxygen: 0,
        }),
        ..module(module_id, ModuleKind::Cargo, grid_x, grid_y)
    }
}

fn state(
    routes: &[((i32, i32), InfrastructureRouteKind)],
    modules: Vec<ModuleSnapshot>,
) -> crate::gameplay::components::ShipInfrastructureState {
    build_infrastructure_state(
        routes.iter().copied().collect::<BTreeMap<_, _>>(),
        modules,
        true,
        &BTreeMap::new(),
        Fx::from_num(0),
    )
}

#[test]
fn turret_connects_to_power_under_and_ammunition_adjacent() {
    let infrastructure = state(
        &[
            ((0, 0), InfrastructureRouteKind::Power),
            (
                (1, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, -1, 0),
            ammunition_storage(3, 2, 0, 4),
        ],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(turret.powered);
    assert!(turret.power_network.is_some());
    assert_eq!(
        turret
            .service_statuses
            .iter()
            .find(|service| service.route_kind == InfrastructureRouteKind::Power)
            .and_then(|service| service.service_coord),
        Some((0, 0))
    );
    assert!(
        turret
            .network_for_resource(ResourceKind::Ammunition)
            .is_some()
    );
    assert_eq!(
        turret
            .service_statuses
            .iter()
            .find(|service| {
                service.route_kind == InfrastructureRouteKind::Resource(ResourceKind::Ammunition)
            })
            .and_then(|service| service.service_coord),
        Some((1, 0))
    );
    assert!(turret.blocked_reason.is_none());
}

#[test]
fn turret_connects_to_ammunition_under_and_power_adjacent() {
    let infrastructure = state(
        &[
            (
                (0, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
            ((1, 0), InfrastructureRouteKind::Power),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, 2, 0),
            ammunition_storage(3, -1, 0, 4),
        ],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(turret.powered);
    assert!(turret.power_network.is_some());
    assert_eq!(
        turret
            .service_statuses
            .iter()
            .find(|service| service.route_kind == InfrastructureRouteKind::Power)
            .and_then(|service| service.service_coord),
        Some((1, 0))
    );
    assert!(
        turret
            .network_for_resource(ResourceKind::Ammunition)
            .is_some()
    );
    assert_eq!(
        turret
            .service_statuses
            .iter()
            .find(|service| {
                service.route_kind == InfrastructureRouteKind::Resource(ResourceKind::Ammunition)
            })
            .and_then(|service| service.service_coord),
        Some((0, 0))
    );
    assert!(turret.blocked_reason.is_none());
}

#[test]
fn turret_with_power_but_no_ammunition_reports_missing_resource() {
    let infrastructure = state(
        &[((0, 0), InfrastructureRouteKind::Power)],
        vec![powered_turret(0, 0), reactor(2, 1, 0)],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(turret.powered);
    assert_eq!(
        turret.blocked_reason.as_deref(),
        Some("no compatible resource")
    );
}

#[test]
fn turret_with_empty_ammunition_network_reports_insufficient_supply() {
    let infrastructure = state(
        &[
            ((0, 0), InfrastructureRouteKind::Power),
            (
                (1, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, -1, 0),
            ammunition_storage(3, 2, 0, 0),
        ],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(turret.powered);
    assert_eq!(
        turret.blocked_reason.as_deref(),
        Some("insufficient supply")
    );
}

#[test]
fn turret_with_ammunition_but_no_power_reports_no_wired_power() {
    let infrastructure = state(
        &[(
            (1, 0),
            InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
        )],
        vec![powered_turret(0, 0), ammunition_storage(2, 2, 0, 4)],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(!turret.powered);
    assert_eq!(turret.blocked_reason.as_deref(), Some("no wired power"));
}

#[test]
fn diagonal_routes_do_not_attach_to_component_service_ports() {
    let infrastructure = state(
        &[
            ((1, 1), InfrastructureRouteKind::Power),
            (
                (-1, -1),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, 1, 2),
            ammunition_storage(3, -1, -2, 4),
        ],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(turret.power_network.is_none());
    assert!(
        turret
            .service_statuses
            .iter()
            .all(|service| service.service_coord.is_none())
    );
    assert!(
        turret
            .network_for_resource(ResourceKind::Ammunition)
            .is_none()
    );
}

#[test]
fn different_route_kinds_do_not_merge_in_adjacent_service_area() {
    let infrastructure = state(
        &[
            ((0, 0), InfrastructureRouteKind::Power),
            (
                (1, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, -1, 0),
            ammunition_storage(3, 2, 0, 4),
        ],
    );

    let power_network = infrastructure
        .networks
        .iter()
        .find(|network| network.kind == Some(InfrastructureRouteKind::Power))
        .unwrap();
    let ammo_network = infrastructure
        .networks
        .iter()
        .find(|network| {
            network.kind == Some(InfrastructureRouteKind::Resource(ResourceKind::Ammunition))
        })
        .unwrap();
    assert_ne!(power_network.id, ammo_network.id);
    assert_eq!(power_network.tile_count, 1);
    assert_eq!(ammo_network.tile_count, 1);
}

#[test]
fn reactor_can_inject_power_and_consume_adjacent_fuel() {
    let infrastructure = state(
        &[
            ((0, 0), InfrastructureRouteKind::Power),
            (
                (1, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Fuel),
            ),
        ],
        vec![reactor(1, 0, 0), fuel_storage(2, 2, 0, 3)],
    );

    let reactor = infrastructure.status_for_module(1).unwrap();
    assert!(reactor.power_network.is_some());
    assert!(reactor.network_for_resource(ResourceKind::Fuel).is_some());
    assert!(reactor.blocked_reason.is_none());
}

#[test]
fn closed_adjacent_junction_breaks_power_without_breaking_ammunition() {
    let mut junction = module(4, ModuleKind::JunctionBox, 1, 0);
    junction.junction_open = Some(false);
    let infrastructure = state(
        &[
            ((1, 0), InfrastructureRouteKind::Power),
            (
                (0, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, 2, 0),
            ammunition_storage(3, -1, 0, 4),
            junction,
        ],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert_eq!(turret.blocked_reason.as_deref(), Some("closed junction"));
    assert!(
        turret
            .network_for_resource(ResourceKind::Ammunition)
            .is_some()
    );
}

#[test]
fn closed_adjacent_valve_breaks_ammunition_without_breaking_power() {
    let mut valve = module(4, ModuleKind::Valve, 1, 0);
    valve.valve_open = Some(false);
    let infrastructure = state(
        &[
            ((0, 0), InfrastructureRouteKind::Power),
            (
                (1, 0),
                InfrastructureRouteKind::Resource(ResourceKind::Ammunition),
            ),
        ],
        vec![
            powered_turret(0, 0),
            reactor(2, -1, 0),
            ammunition_storage(3, 2, 0, 4),
            valve,
        ],
    );

    let turret = infrastructure.status_for_module(1).unwrap();
    assert!(turret.powered);
    assert_eq!(turret.blocked_reason.as_deref(), Some("closed valve"));
}

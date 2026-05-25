use bevy::prelude::*;

use super::*;
use crate::{
    gameplay::{
        components::{
            DetectorKind,
            HostileShip,
            PlayerMotionState,
            PlayerReferenceFrame,
            SimPosition,
            SimRotation,
        },
        helpers::FixedVec2,
        update_detector_modules,
    },
    ship::ModuleVariant,
};

fn runtime_module(kind: ModuleKind, variant: ModuleVariant) -> RuntimeShipModule {
    RuntimeShipModule {
        module_id: 1,
        kind,
        variant,
        channel: 0,
        grid_x: 0,
        grid_y: 0,
        rotation_quadrants: 0,
        local_position: FixedVec2::zero(),
    }
}

fn power_state() -> ShipPowerState {
    ShipPowerState {
        stored_energy: Fx::from_num(6),
        generation: Fx::from_num(4),
        draw: Fx::from_num(3),
        surplus: Fx::from_num(1),
        engine_power_ratio: Fx::from_num(1),
        weapons_powered: true,
        engines_powered: true,
    }
}

#[test]
fn life_pulse_reports_presence_without_vector_detail() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, update_detector_modules);

    app.world_mut().spawn((
        PlayerShip,
        ShipRoot,
        SimPosition {
            value: FixedVec2::zero(),
        },
        SimRotation {
            radians: Fx::from_num(0),
        },
        power_state(),
        ShipDamageSensorState::default(),
    ));
    app.world_mut().spawn((
        runtime_module(ModuleKind::Detector, ModuleVariant::LifePulse),
        DetectorModule {
            kind: DetectorKind::LifeSign,
            tier: 1,
            range: Fx::from_num(200),
            detected: false,
            secondary_detected: false,
            direction: FixedVec2::zero(),
            distance: Fx::from_num(0),
            magnitude: Fx::from_num(0),
            critical: false,
        },
    ));
    app.world_mut().spawn(PlayerMotionState {
        frame: PlayerReferenceFrame::World,
        world_position: FixedVec2::from_num(48, 12),
        world_velocity: FixedVec2::zero(),
        local_position: FixedVec2::zero(),
        local_velocity: FixedVec2::zero(),
        facing_radians: Fx::from_num(0),
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&DetectorModule>();
    let detector = query.single(world).expect("detector present");
    assert!(detector.detected);
    assert!(!detector.secondary_detected);
    assert_eq!(detector.direction.x, Fx::from_num(0));
    assert_eq!(detector.direction.y, Fx::from_num(0));
    assert_eq!(detector.distance, Fx::from_num(0));
}

#[test]
fn ship_survey_reports_hostile_direction_and_distance() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, update_detector_modules);

    app.world_mut().spawn((
        PlayerShip,
        ShipRoot,
        SimPosition {
            value: FixedVec2::zero(),
        },
        SimRotation {
            radians: Fx::from_num(0),
        },
        power_state(),
        ShipDamageSensorState::default(),
    ));
    app.world_mut().spawn((
        HostileShip,
        ShipRoot,
        SimPosition {
            value: FixedVec2::from_num(120, -40),
        },
    ));
    app.world_mut().spawn((
        runtime_module(ModuleKind::Detector, ModuleVariant::ShipSurvey),
        DetectorModule {
            kind: DetectorKind::Ship,
            tier: 3,
            range: Fx::from_num(500),
            detected: false,
            secondary_detected: false,
            direction: FixedVec2::zero(),
            distance: Fx::from_num(0),
            magnitude: Fx::from_num(0),
            critical: false,
        },
    ));

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&DetectorModule>();
    let detector = query.single(world).expect("detector present");
    assert!(detector.detected);
    assert!(detector.secondary_detected);
    assert!(detector.distance > Fx::from_num(0));
    assert!(detector.direction.x > Fx::from_num(0));
    assert!(detector.direction.y < Fx::from_num(0));
}

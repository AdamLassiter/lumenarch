use super::{apply_auto_hull_to_ship, apply_build_action};
use crate::{
    ship::{
        ModuleKind,
        ModuleVariant,
        ShipDefinition,
        ShipFoundationKind,
        ShipFoundationTile,
        ShipModule,
    },
    state::{EditorLayer, EditorMode, EditorToolState, Progression},
};

fn tool_state_for_components(kind: ModuleKind) -> EditorToolState {
    EditorToolState {
        active_layer: EditorLayer::Components,
        selected_kind: kind,
        selected_variant: ModuleVariant::default_for_kind(kind),
        ..Default::default()
    }
}

fn tool_state_for_hull_fixture(kind: ModuleKind) -> EditorToolState {
    EditorToolState {
        active_layer: EditorLayer::Hull,
        selected_kind: kind,
        selected_variant: ModuleVariant::default_for_kind(kind),
        ..Default::default()
    }
}

#[test]
fn components_require_logistics_support() {
    let mut ship = ShipDefinition::empty("Support Test");
    let mut progression = Progression::default();
    let placed = apply_build_action(
        &mut ship,
        &mut progression,
        EditorMode::Enemy,
        &tool_state_for_components(ModuleKind::Core),
        0,
        0,
        false,
    );
    assert!(!placed);

    ship.replace_foundation_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    let placed = apply_build_action(
        &mut ship,
        &mut progression,
        EditorMode::Enemy,
        &tool_state_for_components(ModuleKind::Core),
        0,
        0,
        false,
    );
    assert!(placed);
}

#[test]
fn exterior_modules_require_hull_foundation() {
    let mut ship = ShipDefinition::empty("Exterior Test");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    let mut progression = Progression::default();
    let placed = apply_build_action(
        &mut ship,
        &mut progression,
        EditorMode::Enemy,
        &tool_state_for_hull_fixture(ModuleKind::Turret),
        0,
        0,
        false,
    );
    assert!(!placed);

    ship.replace_hull_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Hull,
        0,
        0,
        0,
    ));
    let placed = apply_build_action(
        &mut ship,
        &mut progression,
        EditorMode::Enemy,
        &tool_state_for_hull_fixture(ModuleKind::Turret),
        0,
        0,
        false,
    );
    assert!(placed);
}

#[test]
fn hull_layer_airlock_selection_places_module_on_hull() {
    let mut ship = ShipDefinition::empty("Airlock Hull Test");
    ship.replace_hull_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Hull,
        0,
        0,
        0,
    ));
    let mut progression = Progression::default();
    let placed = apply_build_action(
        &mut ship,
        &mut progression,
        EditorMode::Enemy,
        &EditorToolState {
            active_layer: EditorLayer::Hull,
            selected_kind: ModuleKind::Airlock,
            selected_variant: ModuleVariant::Standard,
            ..Default::default()
        },
        0,
        0,
        false,
    );
    assert!(placed);
    assert_eq!(
        ship.module_at(0, 0).map(|module| module.kind),
        Some(ModuleKind::Airlock)
    );
}

#[test]
fn auto_hull_writes_foundation_hull_tiles() {
    let mut ship = ShipDefinition::empty("Auto Hull Foundations");
    ship.replace_logistics_tile(ShipFoundationTile::new(
        1,
        ShipFoundationKind::Floor,
        0,
        0,
        0,
    ));
    ship.replace_module(ShipModule::new(1, ModuleKind::Core, 0, 0, 0));

    let changed = apply_auto_hull_to_ship(&mut ship);

    assert!(changed);
    assert!(
        ship.hull_tiles
            .iter()
            .any(|tile| matches!(tile.kind, ShipFoundationKind::HullOuterCorner))
    );
    assert!(ship.modules.iter().all(|module| !matches!(
        module.kind,
        ModuleKind::Hull | ModuleKind::HullInnerCorner | ModuleKind::HullOuterCorner
    )));
}

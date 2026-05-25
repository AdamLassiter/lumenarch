use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    NORMAL_BUTTON,
    SELECTED_BUTTON,
    UI_BUTTON_RADIUS,
    editor::{SELECTED_UNAFFORDABLE_BUTTON, UNAFFORDABLE_BUTTON},
    helpers::{editor::sprite_path_for_kind, sprite_path_for_foundation, variant_inventory_label},
    ship::{ModuleKind, ModuleVariant, ShipFoundationKind},
    state::{
        EditorLayer,
        EditorLayerButton,
        EditorLayerButtonText,
        EditorMode,
        EditorToolMode,
        EditorToolModeButton,
        EditorToolModeButtonText,
        Progression,
        StationEditorTool,
        StationToolboxButton,
        ToolboxFoundationButton,
        ToolboxFoundationButtonText,
        ToolboxVariantButton,
        ToolboxVariantButtonText,
    },
};

const TOOLBOX_GROUP_FOUNDATION: &[ShipFoundationKind] = &[ShipFoundationKind::Floor];
const TOOLBOX_GROUP_HULL: &[ShipFoundationKind] = &[
    ShipFoundationKind::Hull,
    ShipFoundationKind::HullInnerCorner,
    ShipFoundationKind::HullOuterCorner,
];
const TOOLBOX_GROUP_ROUTES: &[ShipFoundationKind] = &[
    ShipFoundationKind::Wire,
    ShipFoundationKind::OxygenDuct,
    ShipFoundationKind::PipeRawSalvage,
    ShipFoundationKind::PipeRepairCharge,
    ShipFoundationKind::PipeFuel,
    ShipFoundationKind::PipeAmmunition,
    ShipFoundationKind::PipeOxygen,
];
const TOOLBOX_GROUP_HULL_FIXTURES: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Airlock, ModuleVariant::Standard),
    (ModuleKind::Airlock, ModuleVariant::DroneBay),
    (ModuleKind::Engine, ModuleVariant::Standard),
    (ModuleKind::Turret, ModuleVariant::LaserTurret),
    (ModuleKind::Turret, ModuleVariant::BallisticTurret),
];
const TOOLBOX_GROUP_COMMAND: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Core, ModuleVariant::BasicCore),
    (ModuleKind::Core, ModuleVariant::ExpandedCore),
    (ModuleKind::Cockpit, ModuleVariant::Standard),
    (ModuleKind::Cockpit, ModuleVariant::AdvancedHelm),
    (ModuleKind::Computer, ModuleVariant::Standard),
];
const TOOLBOX_GROUP_LEGACY_STRUCTURE: &[(ModuleKind, ModuleVariant)] =
    &[(ModuleKind::Interior, ModuleVariant::Standard)];
const TOOLBOX_GROUP_POWER: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Reactor, ModuleVariant::Fission),
    (ModuleKind::Reactor, ModuleVariant::Fusion),
    (ModuleKind::Battery, ModuleVariant::BatteryCell),
    (ModuleKind::Battery, ModuleVariant::Capacitor),
];
const TOOLBOX_GROUP_LOGISTICS: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Processor, ModuleVariant::FabricatorSlow),
    (ModuleKind::Processor, ModuleVariant::FabricatorFast),
    (ModuleKind::Cargo, ModuleVariant::GeneralCargo),
    (ModuleKind::Cargo, ModuleVariant::RawSalvageCrate),
    (ModuleKind::Cargo, ModuleVariant::RepairChargeRack),
    (ModuleKind::Cargo, ModuleVariant::FuelTank),
    (ModuleKind::Cargo, ModuleVariant::AmmoRack),
    (ModuleKind::Cargo, ModuleVariant::O2Canister),
];
const TOOLBOX_GROUP_ENGINEERING: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::JunctionBox, ModuleVariant::Standard),
    (ModuleKind::Valve, ModuleVariant::Standard),
    (ModuleKind::O2Generator, ModuleVariant::Standard),
    (ModuleKind::InteriorWall, ModuleVariant::Standard),
];
const TOOLBOX_GROUP_AUTOMATION: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Detector, ModuleVariant::LifePulse),
    (ModuleKind::Detector, ModuleVariant::LifeSweep),
    (ModuleKind::Detector, ModuleVariant::LifeSurvey),
    (ModuleKind::Detector, ModuleVariant::ShipPing),
    (ModuleKind::Detector, ModuleVariant::ShipVector),
    (ModuleKind::Detector, ModuleVariant::ShipSurvey),
    (ModuleKind::Detector, ModuleVariant::DamageAlarm),
    (ModuleKind::Detector, ModuleVariant::DamageArray),
    (ModuleKind::Detector, ModuleVariant::StructuralSurveyor),
    (ModuleKind::Detector, ModuleVariant::PowerMonitor),
    (ModuleKind::Detector, ModuleVariant::HeatMonitor),
    (ModuleKind::Detector, ModuleVariant::LogisticsBeacon),
];
const TOOLBOX_GROUP_COMBAT: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Shield, ModuleVariant::RadialShield),
    (ModuleKind::Shield, ModuleVariant::DirectionalShield),
];

pub(super) fn component_toolbox_groups()
-> [(&'static str, &'static [(ModuleKind, ModuleVariant)]); 7] {
    [
        ("Command", TOOLBOX_GROUP_COMMAND),
        ("Legacy Structure", TOOLBOX_GROUP_LEGACY_STRUCTURE),
        ("Power", TOOLBOX_GROUP_POWER),
        ("Combat", TOOLBOX_GROUP_COMBAT),
        ("Logistics", TOOLBOX_GROUP_LOGISTICS),
        ("Engineering", TOOLBOX_GROUP_ENGINEERING),
        ("Automation", TOOLBOX_GROUP_AUTOMATION),
    ]
}

pub(super) fn logistics_toolbox_groups() -> [(&'static str, &'static [ShipFoundationKind]); 2] {
    [
        ("Deck", TOOLBOX_GROUP_FOUNDATION),
        ("Routes", TOOLBOX_GROUP_ROUTES),
    ]
}

pub(super) fn hull_toolbox_groups() -> [(&'static str, HullToolboxGroup); 2] {
    [
        ("Hull", HullToolboxGroup::Foundations(TOOLBOX_GROUP_HULL)),
        (
            "Exterior Fixtures",
            HullToolboxGroup::Modules(TOOLBOX_GROUP_HULL_FIXTURES),
        ),
    ]
}

pub(super) enum HullToolboxGroup {
    Foundations(&'static [ShipFoundationKind]),
    Modules(&'static [(ModuleKind, ModuleVariant)]),
}

pub(super) fn spawn_station_tool_button_grid(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    selected_tool: Option<StationEditorTool>,
) {
    for row in StationEditorTool::ALL.chunks(3) {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row_parent| {
                for tool in row {
                    row_parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(74.0),
                                height: Val::Px(62.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(if Some(*tool) == selected_tool {
                                SELECTED_BUTTON
                            } else {
                                NORMAL_BUTTON
                            }),
                            StationToolboxButton { tool: *tool },
                        ))
                        .with_child((
                            Text::new(tool.label()),
                            TextFont {
                                font: font.clone(),
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                            TextLayout::new_with_justify(Justify::Center),
                        ));
                }
            });
    }
}

pub(super) fn spawn_layer_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    layer: EditorLayer,
    selected_layer: EditorLayer,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(33.33),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(if layer == selected_layer {
                SELECTED_BUTTON
            } else {
                NORMAL_BUTTON
            }),
            EditorLayerButton { layer },
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
            EditorLayerButtonText,
        ));
}

pub(super) fn spawn_foundation_button_grid(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    font: &Handle<Font>,
    selected_kind: ShipFoundationKind,
    entries: &[ShipFoundationKind],
) {
    for row in entries.chunks(3) {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row_parent| {
                for kind in row {
                    row_parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(74.0),
                                height: Val::Px(82.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(6.0),
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(if *kind == selected_kind {
                                SELECTED_BUTTON
                            } else {
                                NORMAL_BUTTON
                            }),
                            ToolboxFoundationButton { kind: *kind },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                ImageNode::new(
                                    asset_server.load(sprite_path_for_foundation(*kind)),
                                ),
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                            ));
                            button.spawn((
                                Text::new(kind.display_name()),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 10.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                TextLayout::new_with_justify(Justify::Center),
                                ToolboxFoundationButtonText { kind: *kind },
                            ));
                        });
                }
            });
    }
}

pub(super) fn spawn_tool_mode_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    mode: EditorToolMode,
    selected_mode: EditorToolMode,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(50.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(if mode == selected_mode {
                SELECTED_BUTTON
            } else {
                NORMAL_BUTTON
            }),
            EditorToolModeButton { mode },
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 15.0,
                ..default()
            },
            TextColor(Color::WHITE),
            EditorToolModeButtonText,
        ));
}

pub(super) fn spawn_variant_button_grid(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    font: &Handle<Font>,
    mode: EditorMode,
    progression: &Progression,
    ignore_component_limits: bool,
    selected_kind: ModuleKind,
    selected_variant: ModuleVariant,
    entries: &[(ModuleKind, ModuleVariant)],
) {
    for row in entries.chunks(3) {
        parent
            .spawn(Node {
                width: Val::Percent(100.0),
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row_parent| {
                for (kind, variant) in row {
                    let available = ignore_component_limits
                        || mode == EditorMode::Enemy
                        || mode == EditorMode::Station
                        || progression.ready_count(*kind, *variant) > 0
                        || progression.damaged_count(*kind, *variant) > 0;
                    row_parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(74.0),
                                height: Val::Px(92.0),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(6.0),
                                padding: UiRect::all(Val::Px(6.0)),
                                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                                ..default()
                            },
                            BackgroundColor(toolbox_variant_color(
                                available,
                                *kind == selected_kind && *variant == selected_variant,
                            )),
                            ToolboxVariantButton {
                                kind: *kind,
                                variant: *variant,
                            },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                ImageNode::new(
                                    asset_server.load(sprite_path_for_kind(kind, *variant)),
                                ),
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(32.0),
                                    ..default()
                                },
                            ));
                            button.spawn((
                                Text::new(format!(
                                    "{}\n{}",
                                    variant.display_name(),
                                    if ignore_component_limits {
                                        "limits ignored".to_string()
                                    } else {
                                        variant_inventory_label(mode, progression, *kind, *variant)
                                    }
                                )),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                TextLayout::new_with_justify(Justify::Center),
                                ToolboxVariantButtonText {
                                    kind: *kind,
                                    variant: *variant,
                                },
                            ));
                        });
                }
            });
    }
}

pub(super) fn spawn_select_action_button<T: Bundle + 'static>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    color: Color,
    marker: T,
    font: &Handle<Font>,
) {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(UI_BUTTON_RADIUS)),
                ..default()
            },
            BackgroundColor(color),
            marker,
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font: font.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
}

pub(super) fn toolbox_variant_color(available: bool, selected: bool) -> Color {
    if selected {
        if available {
            SELECTED_BUTTON
        } else {
            SELECTED_UNAFFORDABLE_BUTTON
        }
    } else if available {
        NORMAL_BUTTON
    } else {
        UNAFFORDABLE_BUTTON
    }
}

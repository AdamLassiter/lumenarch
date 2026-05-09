use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    NORMAL_BUTTON,
    SELECTED_BUTTON,
    UI_BODY_FONT_SIZE,
    UI_BUTTON_RADIUS,
    editor::{
        SELECTED_UNAFFORDABLE_BUTTON,
        UNAFFORDABLE_BUTTON,
        helpers::{sprite_path_for_kind, variant_inventory_label},
    },
    ship::{ModuleKind, ModuleVariant},
    state::{
        EditorMode,
        EditorToolMode,
        EditorToolModeButton,
        EditorToolModeButtonText,
        Progression,
        ToolboxVariantButton,
        ToolboxVariantButtonText,
    },
};

const TOOLBOX_GROUP_STRUCTURE: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Hull, ModuleVariant::Standard),
    (ModuleKind::HullInnerCorner, ModuleVariant::Standard),
    (ModuleKind::HullOuterCorner, ModuleVariant::Standard),
    (ModuleKind::Interior, ModuleVariant::Standard),
];
const TOOLBOX_GROUP_COMMAND: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Core, ModuleVariant::BasicCore),
    (ModuleKind::Core, ModuleVariant::ExpandedCore),
    (ModuleKind::Cockpit, ModuleVariant::Standard),
    (ModuleKind::Cockpit, ModuleVariant::AdvancedHelm),
    (ModuleKind::Computer, ModuleVariant::Standard),
];
const TOOLBOX_GROUP_POWER: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Reactor, ModuleVariant::Fission),
    (ModuleKind::Reactor, ModuleVariant::Fusion),
    (ModuleKind::Battery, ModuleVariant::BatteryCell),
    (ModuleKind::Battery, ModuleVariant::Capacitor),
    (ModuleKind::Engine, ModuleVariant::Standard),
];
const TOOLBOX_GROUP_LOGISTICS: &[(ModuleKind, ModuleVariant)] = &[
    (ModuleKind::Processor, ModuleVariant::FabricatorSlow),
    (ModuleKind::Processor, ModuleVariant::FabricatorFast),
    (ModuleKind::Cargo, ModuleVariant::GeneralCargo),
    (ModuleKind::Cargo, ModuleVariant::FuelTank),
    (ModuleKind::Cargo, ModuleVariant::AmmoRack),
    (ModuleKind::Airlock, ModuleVariant::Standard),
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
    (ModuleKind::Turret, ModuleVariant::LaserTurret),
    (ModuleKind::Turret, ModuleVariant::BallisticTurret),
    (ModuleKind::Shield, ModuleVariant::RadialShield),
    (ModuleKind::Shield, ModuleVariant::DirectionalShield),
];

pub(super) fn toolbox_groups() -> [(&'static str, &'static [(ModuleKind, ModuleVariant)]); 6] {
    [
        ("Structure", TOOLBOX_GROUP_STRUCTURE),
        ("Command", TOOLBOX_GROUP_COMMAND),
        ("Power", TOOLBOX_GROUP_POWER),
        ("Logistics", TOOLBOX_GROUP_LOGISTICS),
        ("Automation", TOOLBOX_GROUP_AUTOMATION),
        ("Combat", TOOLBOX_GROUP_COMBAT),
    ]
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
                    let available = mode == EditorMode::Enemy
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
                                    variant_inventory_label(mode, progression, *kind, *variant)
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

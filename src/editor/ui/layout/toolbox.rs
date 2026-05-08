use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

use crate::{
    UI_BODY_FONT_SIZE, UI_BUTTON_RADIUS,
    state::{
        EditorMode, EditorToolMode, EditorToolModeButton, EditorToolModeButtonText, Progression,
        ToolboxVariantButton, ToolboxVariantButtonText,
    },
};

const TOOLBOX_GROUP_STRUCTURE: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)] = &[
    (
        crate::ship::ModuleKind::Hull,
        crate::ship::ModuleVariant::Standard,
    ),
    (
        crate::ship::ModuleKind::HullInnerCorner,
        crate::ship::ModuleVariant::Standard,
    ),
    (
        crate::ship::ModuleKind::HullOuterCorner,
        crate::ship::ModuleVariant::Standard,
    ),
    (
        crate::ship::ModuleKind::Interior,
        crate::ship::ModuleVariant::Standard,
    ),
];
const TOOLBOX_GROUP_COMMAND: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)] = &[
    (
        crate::ship::ModuleKind::Core,
        crate::ship::ModuleVariant::BasicCore,
    ),
    (
        crate::ship::ModuleKind::Core,
        crate::ship::ModuleVariant::ExpandedCore,
    ),
    (
        crate::ship::ModuleKind::Cockpit,
        crate::ship::ModuleVariant::Standard,
    ),
    (
        crate::ship::ModuleKind::Cockpit,
        crate::ship::ModuleVariant::AdvancedHelm,
    ),
    (
        crate::ship::ModuleKind::Computer,
        crate::ship::ModuleVariant::Standard,
    ),
];
const TOOLBOX_GROUP_POWER: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)] = &[
    (
        crate::ship::ModuleKind::Reactor,
        crate::ship::ModuleVariant::Fission,
    ),
    (
        crate::ship::ModuleKind::Reactor,
        crate::ship::ModuleVariant::Fusion,
    ),
    (
        crate::ship::ModuleKind::Battery,
        crate::ship::ModuleVariant::BatteryCell,
    ),
    (
        crate::ship::ModuleKind::Battery,
        crate::ship::ModuleVariant::Capacitor,
    ),
    (
        crate::ship::ModuleKind::Engine,
        crate::ship::ModuleVariant::Standard,
    ),
];
const TOOLBOX_GROUP_LOGISTICS: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)] = &[
    (
        crate::ship::ModuleKind::Processor,
        crate::ship::ModuleVariant::FabricatorSlow,
    ),
    (
        crate::ship::ModuleKind::Processor,
        crate::ship::ModuleVariant::FabricatorFast,
    ),
    (
        crate::ship::ModuleKind::Cargo,
        crate::ship::ModuleVariant::GeneralCargo,
    ),
    (
        crate::ship::ModuleKind::Cargo,
        crate::ship::ModuleVariant::FuelTank,
    ),
    (
        crate::ship::ModuleKind::Cargo,
        crate::ship::ModuleVariant::AmmoRack,
    ),
    (
        crate::ship::ModuleKind::Airlock,
        crate::ship::ModuleVariant::Standard,
    ),
];
const TOOLBOX_GROUP_AUTOMATION: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)] = &[
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::LifePulse,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::LifeSweep,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::LifeSurvey,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::ShipPing,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::ShipVector,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::ShipSurvey,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::DamageAlarm,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::DamageArray,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::StructuralSurveyor,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::PowerMonitor,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::HeatMonitor,
    ),
    (
        crate::ship::ModuleKind::Detector,
        crate::ship::ModuleVariant::LogisticsBeacon,
    ),
];
const TOOLBOX_GROUP_COMBAT: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)] = &[
    (
        crate::ship::ModuleKind::Turret,
        crate::ship::ModuleVariant::LaserTurret,
    ),
    (
        crate::ship::ModuleKind::Turret,
        crate::ship::ModuleVariant::BallisticTurret,
    ),
    (
        crate::ship::ModuleKind::Shield,
        crate::ship::ModuleVariant::RadialShield,
    ),
    (
        crate::ship::ModuleKind::Shield,
        crate::ship::ModuleVariant::DirectionalShield,
    ),
];

pub(super) fn toolbox_groups() -> [(
    &'static str,
    &'static [(crate::ship::ModuleKind, crate::ship::ModuleVariant)],
); 6] {
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
                crate::SELECTED_BUTTON
            } else {
                crate::NORMAL_BUTTON
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
    selected_kind: crate::ship::ModuleKind,
    selected_variant: crate::ship::ModuleVariant,
    entries: &[(crate::ship::ModuleKind, crate::ship::ModuleVariant)],
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
                                ImageNode::new(asset_server.load(
                                    super::super::helpers::sprite_path_for_kind(kind, *variant),
                                )),
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
            crate::SELECTED_BUTTON
        } else {
            crate::editor::SELECTED_UNAFFORDABLE_BUTTON
        }
    } else if available {
        crate::NORMAL_BUTTON
    } else {
        crate::editor::UNAFFORDABLE_BUTTON
    }
}

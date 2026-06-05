use bevy::prelude::*;

use crate::{
    gameplay::{
        components::{
            CarriedItemKind,
            LooseCargo,
            ResourceKind,
            SalvagePickup,
            SalvageWreck,
            SimPosition,
        },
        helpers::{FixedVec2, render_translation},
    },
    state::{MissionArtifactKind, PlayingCleanup},
};

pub(crate) fn spawn_salvage_wreck(commands: &mut Commands, salvage_value: u32) {
    if salvage_value == 0 {
        return;
    }
    let position = FixedVec2::from_num(270.0, -120.0);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.96, 0.86, 0.34), Vec2::new(28.0, 24.0)),
        Transform::from_translation(render_translation(position, 3.0)),
        SimPosition { value: position },
        SalvagePickup {
            scrap_value: salvage_value,
        },
        LooseCargo {
            kind: CarriedItemKind::Resource(ResourceKind::RawSalvage),
            amount: salvage_value,
        },
        SalvageWreck,
        PlayingCleanup,
    ));
}

pub(crate) fn spawn_mission_artifact(commands: &mut Commands, artifact: MissionArtifactKind) {
    let position = match artifact {
        MissionArtifactKind::BlueglassArchiveShard => FixedVec2::from_num(190.0, -165.0),
        MissionArtifactKind::NullSwarmTelemetry => FixedVec2::from_num(235.0, 80.0),
        MissionArtifactKind::ContinuantLedger => FixedVec2::from_num(-180.0, -130.0),
    };
    commands.spawn((
        Sprite::from_color(
            CarriedItemKind::Artifact(artifact).color(),
            Vec2::new(22.0, 18.0),
        ),
        Transform::from_translation(render_translation(position, 3.2)),
        SimPosition { value: position },
        LooseCargo {
            kind: CarriedItemKind::Artifact(artifact),
            amount: 1,
        },
        PlayingCleanup,
    ));
}

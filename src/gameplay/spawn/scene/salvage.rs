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
    state::PlayingCleanup,
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

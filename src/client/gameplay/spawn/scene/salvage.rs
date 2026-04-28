use bevy::prelude::*;

use crate::client::gameplay::{
    components::{SalvagePickup, SalvageWreck, SimPosition},
    helpers::{render_translation, FixedVec2},
};
use crate::client::state::PlayingCleanup;

pub(super) fn spawn_salvage_wreck(commands: &mut Commands) {
    let position = FixedVec2::from_num(270.0, -120.0);
    commands.spawn((
        Sprite::from_color(Color::srgb(0.96, 0.86, 0.34), Vec2::new(28.0, 24.0)),
        Transform::from_translation(render_translation(position, 3.0)),
        SimPosition { value: position },
        SalvagePickup { scrap_value: 6 },
        SalvageWreck,
        PlayingCleanup,
    ));
}

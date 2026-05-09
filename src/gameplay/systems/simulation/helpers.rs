use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    gameplay::{
        components::{
            CarriedItemKind,
            HostileShipModule,
            LooseCargo,
            Projectile,
            ResourceKind,
            RuntimeShipModule,
            SalvagePickup,
            SalvageWreck,
            ShieldCommandState,
            SimPosition,
            StorageModule,
        },
        helpers::{FixedVec2, Fx, angle_from_vector, render_translation, wrap_radians},
    },
    state::PlayingCleanup,
};

pub(crate) type SimVec = FixedVec2;

pub(crate) fn spawn_hostile_salvage(
    commands: &mut Commands,
    position: SimVec,
    salvage_reward: u32,
) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.90, 0.72, 0.28), Vec2::new(30.0, 26.0)),
        Transform::from_translation(render_translation(position, 3.0)),
        SimPosition { value: position },
        SalvagePickup {
            scrap_value: salvage_reward,
        },
        LooseCargo {
            kind: CarriedItemKind::Resource(ResourceKind::RawSalvage),
            amount: salvage_reward,
        },
        SalvageWreck,
        PlayingCleanup,
    ));
}

pub(crate) fn consume_ship_resource(
    storage_query: &mut Query<(&ChildOf, &mut StorageModule)>,
    children: &Children,
    resource_kind: ResourceKind,
    amount: u32,
) -> bool {
    for child in children.iter() {
        let Ok((_parent, mut storage)) = storage_query.get_mut(child) else {
            continue;
        };
        if !storage.accepts(resource_kind) {
            continue;
        }
        if storage.inventory.remove(resource_kind, amount) >= amount {
            return true;
        }
    }
    false
}

pub(crate) fn absorb_hostile_shield_hit(
    root_entity: Entity,
    ship_rotation: Fx,
    projectile: &Projectile,
    shield_query: &mut Query<(&ChildOf, &mut ShieldCommandState), With<HostileShipModule>>,
) -> bool {
    for (parent, mut shield) in shield_query.iter_mut() {
        if parent.get() != root_entity || shield.strength <= Fx::from_num(0) {
            continue;
        }
        if !shield_covers_direction(&shield, ship_rotation, projectile.velocity) {
            continue;
        }
        shield.strength =
            (shield.strength - Fx::from_num(projectile.damage.max(1))).max(Fx::from_num(0));
        return true;
    }
    false
}

pub(crate) fn absorb_player_shield_hit(
    ship_rotation: Fx,
    projectile: &Projectile,
    shield_query: &mut Query<
        (&RuntimeShipModule, &mut ShieldCommandState),
        (
            With<RuntimeShipModule>,
            Without<HostileShipModule>,
            With<ShieldCommandState>,
        ),
    >,
) -> bool {
    for (_runtime_module, mut shield) in shield_query.iter_mut() {
        if shield.strength <= Fx::from_num(0) {
            continue;
        }
        if !shield_covers_direction(&shield, ship_rotation, projectile.velocity) {
            continue;
        }
        shield.strength =
            (shield.strength - Fx::from_num(projectile.damage.max(1))).max(Fx::from_num(0));
        return true;
    }
    false
}

fn shield_covers_direction(
    shield: &ShieldCommandState,
    ship_rotation: Fx,
    velocity: SimVec,
) -> bool {
    if !shield.directional {
        return true;
    }
    let incoming_angle =
        wrap_radians(angle_from_vector(velocity * Fx::from_num(-1)) - ship_rotation);
    let desired = wrap_radians(shield.desired_angle);
    let half_arc = Fx::from_num(shield.width.to_num::<f32>().to_radians()) * Fx::from_num(0.5);
    wrap_radians(incoming_angle - desired).abs() <= half_arc
}

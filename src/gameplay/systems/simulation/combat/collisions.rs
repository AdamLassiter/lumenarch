use std::collections::{BTreeMap, BTreeSet};

use super::*;
use crate::gameplay::helpers::*;

#[derive(Clone, Copy)]
enum CollisionShipKind {
    Player,
    Hostile { salvage_reward: u32 },
}

#[derive(Clone, Copy)]
enum ShipColliderKind {
    Component,
    Shield {
        directional: bool,
        desired_angle: Fx,
        arc_degrees: Fx,
        strength: Fx,
    },
}

#[derive(Clone, Copy)]
struct ShipCollider {
    entity: Entity,
    module_kind: ModuleKind,
    world_position: FixedVec2,
    radius: Fx,
    kind: ShipColliderKind,
}

#[derive(Clone)]
struct ShipCollisionSnapshot {
    entity: Entity,
    children: Vec<Entity>,
    position: FixedVec2,
    velocity: FixedVec2,
    rotation: Fx,
    broad_radius: Fx,
    mass: Fx,
    kind: CollisionShipKind,
    colliders: Vec<ShipCollider>,
}

#[derive(Default, Clone, Copy)]
struct ShipCollisionAdjustment {
    position_delta: FixedVec2,
    velocity_delta: FixedVec2,
}

#[derive(Default, Clone, Copy)]
struct ModuleCollisionDamage {
    damage: i32,
    highest_impact_energy: WideFx,
}

#[derive(Default, Clone, Copy)]
struct ShieldCollisionDamage {
    damage: Fx,
}

pub(crate) fn handle_ship_collisions(
    mut commands: Commands,
    balance: Res<BalanceConfig>,
    mut ship_queries: ParamSet<(
        Query<
            (
                Entity,
                &Children,
                &SimPosition,
                &SimRotation,
                &LinearVelocity,
                Option<&PlayerShip>,
                Option<&HostileShipAi>,
            ),
            With<ShipRoot>,
        >,
        Query<(&mut Transform, &mut SimPosition, &mut LinearVelocity), With<ShipRoot>>,
    )>,
    mut module_queries: ParamSet<(
        Query<
            (
                &RuntimeShipModule,
                &Integrity,
                &ModuleRuntimeState,
                Option<&ShieldCommandState>,
                Option<&DestroyedModule>,
            ),
            With<RuntimeShipModule>,
        >,
        Query<
            (
                &RuntimeShipModule,
                &mut Integrity,
                &mut ModuleRuntimeState,
                Option<&mut ShieldCommandState>,
                Option<&DestroyedModule>,
            ),
            With<RuntimeShipModule>,
        >,
    )>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
) {
    let module_read_query = module_queries.p0();
    let mut ships: Vec<_> = ship_queries
        .p0()
        .iter()
        .map(
            |(entity, children, position, rotation, velocity, player_ship, hostile_ai)| {
                build_ship_snapshot(
                    entity,
                    children,
                    position.value,
                    velocity.value,
                    rotation.radians,
                    player_ship.is_some(),
                    hostile_ai,
                    &balance,
                    &module_read_query,
                )
            },
        )
        .collect();
    ships.sort_by_key(|ship| ship.entity.index());

    let mut ship_adjustments = BTreeMap::<Entity, ShipCollisionAdjustment>::new();
    let mut module_damage = BTreeMap::<Entity, ModuleCollisionDamage>::new();
    let mut shield_damage = BTreeMap::<Entity, ShieldCollisionDamage>::new();

    for index in 0..ships.len() {
        for other_index in (index + 1)..ships.len() {
            let ship_a = &ships[index];
            let ship_b = &ships[other_index];
            let ship_delta = ship_b.position - ship_a.position;
            let ship_distance = ship_delta.length();
            if ship_distance > ship_a.broad_radius + ship_b.broad_radius {
                continue;
            }

            let relative_velocity = ship_b.velocity - ship_a.velocity;
            let default_normal = if ship_distance > Fx::from_num(0.001) {
                ship_delta * (Fx::from_num(1) / ship_distance)
            } else if !relative_velocity.is_near_zero() {
                relative_velocity.normalized_or_zero()
            } else {
                FixedVec2::new(Fx::from_num(1), Fx::from_num(0))
            };

            for collider_a in &ship_a.colliders {
                for collider_b in &ship_b.colliders {
                    if !colliders_can_contact(ship_a, collider_a, ship_b, collider_b) {
                        continue;
                    }

                    let delta = collider_b.world_position - collider_a.world_position;
                    let distance = delta.length();
                    let overlap = collider_a.radius + collider_b.radius - distance;
                    if overlap <= Fx::from_num(0) {
                        continue;
                    }

                    let normal = if distance > Fx::from_num(0.001) {
                        delta * (Fx::from_num(1) / distance)
                    } else {
                        default_normal
                    };
                    let closing_speed = clamp_non_negative(
                        (-dot(relative_velocity, normal)).max(Fx::from_num(0)),
                        Fx::from_num(balance.combat.collision_max_effective_speed),
                    );
                    let collision_energy = collision_energy_wide(
                        ship_a.mass,
                        ship_b.mass,
                        closing_speed,
                        Fx::from_num(balance.combat.collision_max_effective_mass),
                        Fx::from_num(balance.combat.collision_max_effective_speed),
                    );

                    apply_overlap_separation(
                        ship_a,
                        ship_b,
                        normal,
                        overlap,
                        &balance,
                        &mut ship_adjustments,
                    );
                    apply_collision_impulse(
                        ship_a,
                        ship_b,
                        normal,
                        closing_speed,
                        &balance,
                        &mut ship_adjustments,
                    );

                    let damage = collision_damage_from_energy(
                        collision_energy,
                        WideFx::from_num(balance.combat.collision_damage_energy_threshold),
                        WideFx::from_num(balance.combat.collision_damage_energy_divisor),
                    );
                    if damage <= 0 {
                        continue;
                    }

                    apply_collider_damage(
                        collider_a,
                        damage,
                        collision_energy,
                        &mut module_damage,
                        &mut shield_damage,
                    );
                    apply_collider_damage(
                        collider_b,
                        damage,
                        collision_energy,
                        &mut module_damage,
                        &mut shield_damage,
                    );
                }
            }
        }
    }

    for (entity, adjustment) in ship_adjustments {
        let mut ship_state_query = ship_queries.p1();
        let Ok((mut transform, mut position, mut velocity)) = ship_state_query.get_mut(entity)
        else {
            continue;
        };
        position.value += adjustment.position_delta;
        velocity.value += adjustment.velocity_delta;
        transform.translation = render_translation(position.value, transform.translation.z);
    }

    let mut mission_state = mission_query.into_inner();
    let ship_kinds: BTreeMap<_, _> = ships.iter().map(|ship| (ship.entity, ship.kind)).collect();
    let mut roots_to_despawn = BTreeSet::new();

    for (module_entity, shield_loss) in shield_damage {
        let mut module_apply_query = module_queries.p1();
        let Ok((_, _, mut runtime_state, shield_state, destroyed)) =
            module_apply_query.get_mut(module_entity)
        else {
            continue;
        };
        if destroyed.is_some() {
            continue;
        }
        let Some(mut shield_state) = shield_state else {
            continue;
        };
        shield_state.strength = (shield_state.strength - shield_loss.damage).max(Fx::from_num(0));
        runtime_state.current_heat += shield_loss.damage * Fx::from_num(0.08);
        runtime_state.needs_attention = true;
    }

    for ship in &ships {
        for module_entity in &ship.children {
            let Some(damage) = module_damage.get(module_entity).copied() else {
                continue;
            };
            let mut module_apply_query = module_queries.p1();
            let Ok((runtime_module, mut integrity, mut runtime_state, _, destroyed)) =
                module_apply_query.get_mut(*module_entity)
            else {
                continue;
            };
            if destroyed.is_some() || integrity.current <= 0 {
                continue;
            }

            integrity.current = (integrity.current - damage.damage).max(0);
            runtime_state.current_heat +=
                narrow_wide_clamped(safe_sqrt_wide(damage.highest_impact_energy))
                    * Fx::from_num(balance.combat.collision_heat_from_damage);
            runtime_state.needs_attention = true;

            if integrity.current > 0 || destroyed.is_some() {
                continue;
            }

            commands.entity(*module_entity).insert(DestroyedModule);
            match ship_kinds.get(&ship.entity).copied() {
                Some(CollisionShipKind::Player) if runtime_module.kind == ModuleKind::Core => {
                    mission_state.failed = true;
                    mission_state.failure_reason = Some("Core destroyed".to_string());
                    mission_state.encounter_cleared = false;
                    mission_state.completed = false;
                    mission_state.completion_reason = None;
                    mission_state
                        .return_delay_remaining
                        .get_or_insert(Fx::from_num(balance.mission.return_delay_seconds));
                }
                Some(CollisionShipKind::Hostile { salvage_reward })
                    if runtime_module.kind == ModuleKind::Core =>
                {
                    helpers::spawn_hostile_salvage(&mut commands, ship.position, salvage_reward);
                    roots_to_despawn.insert(ship.entity);
                }
                _ => {}
            }
        }
    }

    for root_entity in roots_to_despawn {
        commands.entity(root_entity).despawn();
    }
}

fn build_ship_snapshot(
    entity: Entity,
    children: &Children,
    position: FixedVec2,
    velocity: FixedVec2,
    rotation: Fx,
    is_player: bool,
    hostile_ai: Option<&HostileShipAi>,
    balance: &BalanceConfig,
    module_query: &Query<
        (
            &RuntimeShipModule,
            &Integrity,
            &ModuleRuntimeState,
            Option<&ShieldCommandState>,
            Option<&DestroyedModule>,
        ),
        With<RuntimeShipModule>,
    >,
) -> ShipCollisionSnapshot {
    let component_collider_radius = Fx::from_num(balance.combat.component_collider_radius);
    let shield_collider_radius = Fx::from_num(balance.combat.shield_collider_radius);
    let mut colliders = Vec::new();
    let mut broad_radius = component_collider_radius;
    let mut live_module_count = 0usize;

    for child in children.iter() {
        let Ok((runtime_module, integrity, runtime_state, shield_state, destroyed)) =
            module_query.get(child)
        else {
            continue;
        };
        if destroyed.is_some() || integrity.current <= 0 || runtime_state.extracted {
            continue;
        }
        live_module_count += 1;
        let world_position = position + runtime_module.local_position.rotate(rotation);

        if runtime_module.kind == ModuleKind::Shield
            && let Some(shield_state) = shield_state
            && shield_state.strength > Fx::from_num(0)
        {
            let radius = shield_collider_radius;
            broad_radius = broad_radius.max((world_position - position).length() + radius);
            colliders.push(ShipCollider {
                entity: child,
                module_kind: runtime_module.kind,
                world_position,
                radius,
                kind: ShipColliderKind::Shield {
                    directional: shield_state.directional,
                    desired_angle: shield_state.desired_angle,
                    arc_degrees: shield_state.width,
                    strength: shield_state.strength,
                },
            });
            continue;
        }

        let radius = component_collider_radius;
        broad_radius = broad_radius.max((world_position - position).length() + radius);
        colliders.push(ShipCollider {
            entity: child,
            module_kind: runtime_module.kind,
            world_position,
            radius,
            kind: ShipColliderKind::Component,
        });
    }

    ShipCollisionSnapshot {
        entity,
        children: children.iter().collect(),
        position,
        velocity,
        rotation,
        broad_radius,
        mass: Fx::from_num(live_module_count.max(1) as i32),
        kind: if is_player {
            CollisionShipKind::Player
        } else {
            CollisionShipKind::Hostile {
                salvage_reward: hostile_ai.map_or(0, |ai| ai.salvage_reward),
            }
        },
        colliders,
    }
}

fn colliders_can_contact(
    ship_a: &ShipCollisionSnapshot,
    collider_a: &ShipCollider,
    ship_b: &ShipCollisionSnapshot,
    collider_b: &ShipCollider,
) -> bool {
    match (collider_a.kind, collider_b.kind) {
        (
            ShipColliderKind::Shield {
                directional,
                desired_angle,
                arc_degrees,
                strength,
            },
            _,
        ) if strength > Fx::from_num(0) => shield_accepts_contact(
            ship_a.rotation,
            collider_a.world_position,
            collider_b.world_position,
            directional,
            desired_angle,
            arc_degrees,
            angle_from_vector,
        ),
        (
            _,
            ShipColliderKind::Shield {
                directional,
                desired_angle,
                arc_degrees,
                strength,
            },
        ) if strength > Fx::from_num(0) => shield_accepts_contact(
            ship_b.rotation,
            collider_b.world_position,
            collider_a.world_position,
            directional,
            desired_angle,
            arc_degrees,
            angle_from_vector,
        ),
        _ => true,
    }
}

fn apply_overlap_separation(
    ship_a: &ShipCollisionSnapshot,
    ship_b: &ShipCollisionSnapshot,
    normal: FixedVec2,
    overlap: Fx,
    balance: &BalanceConfig,
    ship_adjustments: &mut BTreeMap<Entity, ShipCollisionAdjustment>,
) {
    let inv_mass_a = Fx::from_num(1) / ship_a.mass.max(Fx::from_num(1));
    let inv_mass_b = Fx::from_num(1) / ship_b.mass.max(Fx::from_num(1));
    let inv_mass_sum = (inv_mass_a + inv_mass_b).max(Fx::from_num(0.001));
    let correction = normal * overlap * Fx::from_num(balance.combat.collision_push_stiffness);
    let share_a = inv_mass_a / inv_mass_sum;
    let share_b = inv_mass_b / inv_mass_sum;

    ship_adjustments
        .entry(ship_a.entity)
        .or_default()
        .position_delta -= correction * share_a;
    ship_adjustments
        .entry(ship_b.entity)
        .or_default()
        .position_delta += correction * share_b;
}

fn apply_collision_impulse(
    ship_a: &ShipCollisionSnapshot,
    ship_b: &ShipCollisionSnapshot,
    normal: FixedVec2,
    closing_speed: Fx,
    balance: &BalanceConfig,
    ship_adjustments: &mut BTreeMap<Entity, ShipCollisionAdjustment>,
) {
    if closing_speed <= Fx::from_num(0) {
        return;
    }

    let inv_mass_a = Fx::from_num(1) / ship_a.mass.max(Fx::from_num(1));
    let inv_mass_b = Fx::from_num(1) / ship_b.mass.max(Fx::from_num(1));
    let inv_mass_sum = inv_mass_a + inv_mass_b;
    if inv_mass_sum <= Fx::from_num(0) {
        return;
    }

    let impulse = normal
        * (closing_speed * (Fx::from_num(1) + Fx::from_num(balance.combat.collision_restitution))
            / inv_mass_sum);
    ship_adjustments
        .entry(ship_a.entity)
        .or_default()
        .velocity_delta -= impulse * inv_mass_a;
    ship_adjustments
        .entry(ship_b.entity)
        .or_default()
        .velocity_delta += impulse * inv_mass_b;
}

fn apply_collider_damage(
    collider: &ShipCollider,
    damage: i32,
    collision_energy: WideFx,
    module_damage: &mut BTreeMap<Entity, ModuleCollisionDamage>,
    shield_damage: &mut BTreeMap<Entity, ShieldCollisionDamage>,
) {
    match collider.kind {
        ShipColliderKind::Shield { .. } => {
            let entry = shield_damage.entry(collider.entity).or_default();
            entry.damage += Fx::from_num(damage.max(1));
        }
        ShipColliderKind::Component => {
            let entry = module_damage.entry(collider.entity).or_default();
            let component_damage = if collider.module_kind == ModuleKind::Core {
                damage.max(1)
            } else {
                damage
            };
            entry.damage += component_damage;
            entry.highest_impact_energy = entry.highest_impact_energy.max(collision_energy);
        }
    }
}

fn dot(a: FixedVec2, b: FixedVec2) -> Fx {
    a.x * b.x + a.y * b.y
}

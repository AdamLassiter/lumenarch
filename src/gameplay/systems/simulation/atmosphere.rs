use std::collections::HashMap;

use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    balance::BalanceConfig,
    gameplay::{
        components::{
            AirlockCommandState,
            DestroyedModule,
            EquippedSuit,
            HostileShip,
            MissionState,
            PlayerFieldState,
            PlayerHandleComponent,
            PlayerMotionState,
            PlayerReferenceFrame,
            PlayerShip,
            RuntimeShipModule,
            ShipAtmosphereState,
            ShipRoot,
            ShipboardPlayer,
        },
        helpers::{
            Fx,
            breach_leak_multiplier,
            decompression_signature,
            fx_from_time_delta,
            recompute_decompression_vectors,
        },
    },
};

pub(crate) fn update_ship_atmosphere(
    time: Res<Time>,
    balance: Res<BalanceConfig>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    mut ship_query: Query<
        (
            Entity,
            Option<&PlayerShip>,
            Option<&HostileShip>,
            &mut ShipAtmosphereState,
        ),
        With<ShipRoot>,
    >,
    module_query: Query<(
        &ChildOf,
        &RuntimeShipModule,
        Option<&AirlockCommandState>,
        Option<&DestroyedModule>,
    )>,
) {
    let dt = fx_from_time_delta(&time);
    let mut mission_state = mission_query.into_inner();

    for (ship_entity, player_ship, hostile_ship, mut atmosphere_state) in &mut ship_query {
        let module_state: HashMap<u64, (bool, bool)> = module_query
            .iter()
            .filter(|(parent, _, _, _)| parent.get() == ship_entity)
            .map(|(_, runtime_module, airlock_state, destroyed)| {
                (
                    runtime_module.module_id,
                    (
                        destroyed.is_some(),
                        airlock_state.is_some_and(|state| state.open),
                    ),
                )
            })
            .collect();

        if atmosphere_state.tiles.is_empty() {
            atmosphere_state.average_oxygen = Fx::from_num(0);
            atmosphere_state.minimum_oxygen = Fx::from_num(0);
            atmosphere_state.venting_tiles = 0;
            atmosphere_state.decompression_signature = 0;
            atmosphere_state.decompression_vectors.clear();
            continue;
        }

        let current_levels: Vec<Fx> = atmosphere_state
            .tiles
            .iter()
            .map(|tile| tile.oxygen)
            .collect();
        let mut next_levels = current_levels.clone();
        let tile_positions: HashMap<(i32, i32), usize> = atmosphere_state
            .tiles
            .iter()
            .enumerate()
            .map(|(index, tile)| ((tile.grid_x, tile.grid_y), index))
            .collect();

        let equalization_rate = Fx::from_num(balance.atmosphere.equalization_rate);
        for (index, tile) in atmosphere_state.tiles.iter().enumerate() {
            for neighbor in [
                (tile.grid_x, tile.grid_y - 1),
                (tile.grid_x + 1, tile.grid_y),
                (tile.grid_x, tile.grid_y + 1),
                (tile.grid_x - 1, tile.grid_y),
            ] {
                let Some(&other_index) = tile_positions.get(&neighbor) else {
                    continue;
                };
                if other_index <= index {
                    continue;
                }
                let delta = current_levels[other_index] - current_levels[index];
                let transfer = delta * equalization_rate * dt * Fx::from_num(0.5);
                next_levels[index] += transfer;
                next_levels[other_index] -= transfer;
            }
        }

        let mut venting_tiles = 0u32;
        let mut vent_edges_per_tile = vec![0u8; atmosphere_state.tiles.len()];
        let mut total_open_edges = 0u32;
        let leak_rate = Fx::from_num(balance.atmosphere.leak_rate_per_edge);
        let destroyed_multiplier = Fx::from_num(balance.atmosphere.destroyed_leak_multiplier);
        let max_oxygen = Fx::from_num(balance.atmosphere.max_tile_oxygen);
        for (index, tile) in atmosphere_state.tiles.iter().enumerate() {
            let (destroyed, airlock_open) = module_state
                .get(&tile.module_id)
                .copied()
                .unwrap_or((false, false));
            let exterior_edges = tile.exterior_edges.count_ones();
            let vent_edges = if exterior_edges == 0 {
                0
            } else if destroyed || airlock_open {
                exterior_edges
            } else {
                0
            };
            if vent_edges > 0 {
                venting_tiles += 1;
                vent_edges_per_tile[index] = vent_edges as u8;
                total_open_edges += vent_edges;
            }
        }

        let breach_leak_scale = breach_leak_multiplier(total_open_edges, &balance);
        for (index, tile) in atmosphere_state.tiles.iter().enumerate() {
            let vent_edges = vent_edges_per_tile[index];
            if vent_edges > 0 {
                let (destroyed, _) = module_state
                    .get(&tile.module_id)
                    .copied()
                    .unwrap_or((false, false));
                let multiplier = if destroyed {
                    destroyed_multiplier
                } else {
                    Fx::from_num(1)
                };
                let leak = leak_rate
                    * Fx::from_num(vent_edges as i32)
                    * breach_leak_scale
                    * multiplier
                    * dt;
                next_levels[index] = (next_levels[index] - leak).max(Fx::from_num(0));
            }
            next_levels[index] = next_levels[index].clamp(Fx::from_num(0), max_oxygen);
        }

        let mut oxygen_sum = Fx::from_num(0);
        let mut oxygen_min = max_oxygen;
        for (index, tile) in atmosphere_state.tiles.iter_mut().enumerate() {
            tile.oxygen = next_levels[index];
            oxygen_sum += tile.oxygen;
            oxygen_min = oxygen_min.min(tile.oxygen);
        }

        atmosphere_state.average_oxygen =
            oxygen_sum / Fx::from_num(atmosphere_state.tiles.len().max(1) as i32);
        atmosphere_state.minimum_oxygen = oxygen_min;
        atmosphere_state.venting_tiles = venting_tiles;

        let signature = decompression_signature(&vent_edges_per_tile);
        if signature != atmosphere_state.decompression_signature
            || atmosphere_state.decompression_vectors.len() != atmosphere_state.tiles.len()
        {
            atmosphere_state.decompression_signature = signature;
            atmosphere_state.decompression_vectors = recompute_decompression_vectors(
                &atmosphere_state.tiles,
                &vent_edges_per_tile,
                &balance,
            );
        }

        if player_ship.is_some() && venting_tiles > 0 {
            mission_state.player_ship_breached = true;
        }
        if hostile_ship.is_some()
            && !atmosphere_state.decompression_reported
            && atmosphere_state.average_oxygen
                <= Fx::from_num(balance.atmosphere.hostile_decompression_threshold)
        {
            atmosphere_state.decompression_reported = true;
            mission_state.hostile_decompression_events += 1;
        }
    }
}

pub(crate) fn sample_player_atmosphere(
    balance: Res<BalanceConfig>,
    mission_query: Single<&mut MissionState, (With<PlayerShip>, With<ShipRoot>)>,
    ship_query: Query<&ShipAtmosphereState, With<ShipRoot>>,
    mut player_query: Query<
        (
            &PlayerHandleComponent,
            &PlayerMotionState,
            &EquippedSuit,
            &mut PlayerFieldState,
        ),
        With<ShipboardPlayer>,
    >,
) {
    let mut mission_state = mission_query.into_inner();
    let mut players: Vec<_> = player_query.iter_mut().collect();
    players.sort_by_key(|(handle, _, _, _)| handle.handle);
    for (_, player_motion, equipped_suit, mut player_fields) in players {
        let Some(ship_entity) = (match player_motion.frame {
            PlayerReferenceFrame::Ship(ship_entity) => Some(ship_entity),
            PlayerReferenceFrame::World => None,
        }) else {
            player_fields.local_oxygen = Fx::from_num(0);
            player_fields.oxygen_warning = false;
            player_fields.oxygen_critical = false;
            continue;
        };

        let Ok(atmosphere_state) = ship_query.get(ship_entity) else {
            player_fields.local_oxygen = Fx::from_num(0);
            player_fields.oxygen_warning = false;
            player_fields.oxygen_critical = false;
            continue;
        };

        let Some(local_oxygen) = atmosphere_state
            .tiles
            .iter()
            .filter(|tile| point_inside_tile(player_motion.local_position, tile.local_position))
            .min_by_key(|tile| {
                (tile.local_position - player_motion.local_position)
                    .length_sq()
                    .to_num::<i128>()
            })
            .map(|tile| tile.oxygen)
        else {
            player_fields.local_oxygen = Fx::from_num(0);
            player_fields.oxygen_warning = false;
            player_fields.oxygen_critical = false;
            mission_state.lowest_player_oxygen =
                mission_state.lowest_player_oxygen.min(Fx::from_num(0));
            continue;
        };

        player_fields.local_oxygen = local_oxygen;
        player_fields.oxygen_warning =
            local_oxygen <= equipped_suit.suit.oxygen_warning_threshold(&balance.player);
        player_fields.oxygen_critical = local_oxygen
            <= equipped_suit
                .suit
                .oxygen_critical_threshold(&balance.player);
        mission_state.lowest_player_oxygen = mission_state.lowest_player_oxygen.min(local_oxygen);
    }
}

fn point_inside_tile(
    point: crate::gameplay::helpers::FixedVec2,
    tile_center: crate::gameplay::helpers::FixedVec2,
) -> bool {
    let tile_half = Fx::from_num(16);
    (point.x - tile_center.x).abs() <= tile_half && (point.y - tile_center.y).abs() <= tile_half
}

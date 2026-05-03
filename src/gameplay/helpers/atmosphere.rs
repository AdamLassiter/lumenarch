use std::collections::{HashMap, VecDeque};

use super::{FixedVec2, Fx, narrow_wide_clamped, safe_sqrt_wide};
use crate::{
    balance::BalanceConfig,
    gameplay::components::{ShipAtmosphereState, ShipAtmosphereTile},
};

const EDGE_TOP: u8 = 1;
const EDGE_RIGHT: u8 = 1 << 1;
const EDGE_BOTTOM: u8 = 1 << 2;
const EDGE_LEFT: u8 = 1 << 3;

pub(crate) fn breach_leak_multiplier(total_open_edges: u32, balance: &BalanceConfig) -> Fx {
    if total_open_edges == 0 {
        return Fx::from_num(0);
    }
    let minimum = Fx::from_num(balance.atmosphere.minimum_breach_leak_multiplier);
    let divisor = Fx::from_num(balance.atmosphere.breach_leak_sqrt_divisor).max(Fx::from_num(1));
    let sqrt_edges = narrow_wide_clamped(safe_sqrt_wide(
        crate::gameplay::helpers::WideFx::from_num(total_open_edges as i32),
    ));
    (sqrt_edges / divisor).max(minimum)
}

pub(crate) fn decompression_signature(vent_edges: &[u8]) -> u64 {
    vent_edges
        .iter()
        .enumerate()
        .fold(0xcbf29ce484222325u64, |hash, (index, edges)| {
            let payload = ((index as u64) << 8) ^ (*edges as u64);
            hash.wrapping_mul(0x100000001b3).wrapping_add(payload)
        })
}

pub(crate) fn recompute_decompression_vectors(
    tiles: &[ShipAtmosphereTile],
    vent_edges: &[u8],
    balance: &BalanceConfig,
) -> Vec<FixedVec2> {
    let mut vectors = vec![FixedVec2::zero(); tiles.len()];
    if tiles.is_empty() || vent_edges.len() != tiles.len() {
        return vectors;
    }

    let openings: Vec<usize> = vent_edges
        .iter()
        .enumerate()
        .filter_map(|(index, edges)| (*edges != 0).then_some(index))
        .collect();
    if openings.is_empty() {
        return vectors;
    }

    let total_open_edges: u32 = vent_edges.iter().map(|edges| edges.count_ones()).sum();
    let base_pull = Fx::from_num(balance.atmosphere.decompression_pull_acceleration)
        * breach_leak_multiplier(total_open_edges, balance);
    let falloff =
        Fx::from_num(balance.atmosphere.decompression_pull_falloff_per_tile).max(Fx::from_num(0));

    let positions: HashMap<(i32, i32), usize> = tiles
        .iter()
        .enumerate()
        .map(|(index, tile)| ((tile.grid_x, tile.grid_y), index))
        .collect();

    let mut distance = vec![u32::MAX; tiles.len()];
    let mut queue = VecDeque::new();
    for &opening in &openings {
        distance[opening] = 0;
        queue.push_back(opening);
        vectors[opening] = breach_outflow_vector(vent_edges[opening]) * base_pull;
    }

    while let Some(index) = queue.pop_front() {
        let next_distance = distance[index].saturating_add(1);
        let tile = &tiles[index];
        for neighbor_pos in [
            (tile.grid_x, tile.grid_y - 1),
            (tile.grid_x + 1, tile.grid_y),
            (tile.grid_x, tile.grid_y + 1),
            (tile.grid_x - 1, tile.grid_y),
        ] {
            let Some(&neighbor_index) = positions.get(&neighbor_pos) else {
                continue;
            };
            if distance[neighbor_index] <= next_distance {
                continue;
            }
            distance[neighbor_index] = next_distance;
            queue.push_back(neighbor_index);

            let direction = cardinal_direction_towards(
                tiles[index].local_position - tiles[neighbor_index].local_position,
            );
            let denominator = Fx::from_num(1) + Fx::from_num(next_distance as i32) * falloff;
            vectors[neighbor_index] = direction * (base_pull / denominator.max(Fx::from_num(1)));
        }
    }

    vectors
}

pub(crate) fn sampled_decompression_pull(
    local_position: FixedVec2,
    atmosphere_state: &ShipAtmosphereState,
) -> FixedVec2 {
    atmosphere_state
        .tiles
        .iter()
        .zip(atmosphere_state.decompression_vectors.iter())
        .filter(|(tile, _)| point_inside_tile(local_position, tile.local_position))
        .min_by_key(|(tile, _)| {
            (tile.local_position - local_position)
                .length_sq()
                .to_num::<i128>()
        })
        .map(|(_, vector)| *vector)
        .unwrap_or_else(FixedVec2::zero)
}

fn breach_outflow_vector(vent_edges: u8) -> FixedVec2 {
    let mut x = Fx::from_num(0);
    let mut y = Fx::from_num(0);
    if vent_edges & EDGE_TOP != 0 {
        y += Fx::from_num(1);
    }
    if vent_edges & EDGE_RIGHT != 0 {
        x += Fx::from_num(1);
    }
    if vent_edges & EDGE_BOTTOM != 0 {
        y -= Fx::from_num(1);
    }
    if vent_edges & EDGE_LEFT != 0 {
        x -= Fx::from_num(1);
    }
    FixedVec2::new(x, y)
}

fn cardinal_direction_towards(delta: FixedVec2) -> FixedVec2 {
    if delta.x > Fx::from_num(0) {
        FixedVec2::new(Fx::from_num(1), Fx::from_num(0))
    } else if delta.x < Fx::from_num(0) {
        FixedVec2::new(Fx::from_num(-1), Fx::from_num(0))
    } else if delta.y > Fx::from_num(0) {
        FixedVec2::new(Fx::from_num(0), Fx::from_num(1))
    } else if delta.y < Fx::from_num(0) {
        FixedVec2::new(Fx::from_num(0), Fx::from_num(-1))
    } else {
        FixedVec2::zero()
    }
}

fn point_inside_tile(point: FixedVec2, tile_center: FixedVec2) -> bool {
    let tile_half = Fx::from_num(16);
    (point.x - tile_center.x).abs() <= tile_half && (point.y - tile_center.y).abs() <= tile_half
}

# 22_TUBES_RUNTIME_DEMO

## Summary

Implemented the runtime half of the tubes/infrastructure vision introduced in `21_TUBES_DEMO`.

Ship underlay routes now rebuild into deterministic runtime networks for:

- power wire
- oxygen duct
- raw salvage pipe
- repair charge pipe
- fuel pipe
- ammunition pipe
- oxygen resource pipe

The key rule for this slice is strict routing: runtime systems should read visible infrastructure instead of falling back to ship-global magic pools.

## What Changed

### Infrastructure Runtime

Added `ShipInfrastructureState` as a ship-level runtime summary containing:

- stable network ids
- route/resource kind
- network tiles
- attached module ids
- supply, demand, reserve, flow, and blocker summaries
- per-module infrastructure status and blocked reason

Networks rebuild from runtime foundation tiles with cardinal adjacency only. Same route kinds connect to same route kinds. Missing, destroyed, or blocked route tiles break the graph.

Overlay modules attach to compatible underlay routes at their own cell or cardinally adjacent cells.

### Blockers

Junction boxes and valves now have runtime open/closed command state.

- closed junction boxes split power wiring
- closed valves split oxygen ducts and resource pipes
- both can be toggled manually through station controls
- both expose compact ARCH read/command registers

### Power

Runtime power now comes from routed power wire networks.

- reactors inject into connected wire networks
- consumers require connected routed power
- disconnected or starved consumers report blocked state such as `no wired power`
- the legacy ship power HUD remains populated from aggregated routed network state

The cockpit no longer has special station access behavior. Like every other station, it is interactable only through nearby module interaction.

### Oxygen

Oxygen remains tile-atmosphere based for breathing.

Oxygen ducts now replenish connected/reachable atmosphere tiles when attached infrastructure has oxygen supply. Closed valves stop duct flow because they split the duct network.

### Resources

Automatic resource use now checks typed infrastructure where relevant.

- reactors consume fuel only from connected fuel storage
- ammunition weapons consume ammunition only from connected ammunition storage
- processors pull raw salvage from connected raw-salvage storage
- processors output repair charge, fuel, or ammunition to compatible connected storage when available
- existing manual, manipulator, and drone logistics remain alongside pipe routing

### UI And ARCH

Station readouts now surface infrastructure status:

- connected route/network info
- powered or blocked state
- relevant resource summaries
- blocker controls for junctions and valves

ARCH gained compact infrastructure registers:

- `JBO0`, `JBP0`, `JBS0`, `JBD0` for junction open/power/supply/demand readback
- `VLO0`, `VLP0`, `VLS0`, `VLD0` for valve open/power/supply/demand readback
- `JBC0` and `VLC0` to command junctions/valves open or closed

The parser surface is intentionally minimal here; broader indexed authoring remains part of the deferred complete ARCH slice.

## Design Notes

This slice moves the game toward the concept document's central promise: the player should progress from manual engineering into automation over real physical systems.

The implementation favors deterministic full graph rebuilds for now. That keeps damage, editing, and blocker state straightforward while the infrastructure rules are still settling. Incremental graph updates can come later if profiling demands it.

## Known Follow-Ups

The core runtime path is in, but a few pieces should be hardened in follow-up slices:

- add focused graph/unit tests for deterministic ids, blockers, and compatible attachment rules
- add a richer debug overlay for power, oxygen duct, and typed resource networks
- make per-network reserve/battery behavior more physical instead of summary-only
- route repair-charge consumption for repair workflows
- add indexed ARCH registers or targeting once `15_ARCH_COMPLETE_DEMO` resumes

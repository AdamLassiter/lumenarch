# 21_TUBES_DEMO TODO

## Goal

Implement a two-layer ship engineering model where wiring, ducts, and resource pipes are physical underlay networks that power and feed overlay components.

## Phase 1 - Ship Schema And Reset

- [x] Add a saved underlay/foundation tile model for:
  - hull / shell tiles
  - deck / floor tiles
  - power wiring
  - oxygen ducts
  - typed resource pipes
- [x] Keep `ShipModule` as the overlay component model.
- [x] Add an overlay interior wall build item.
- [x] Allow one underlay tile and one overlay tile to share the same grid coordinate.
- [x] Update ship bounds, lookup helpers, validation, normalization, and id generation for both layers.
- [x] Relax required ship validation to core-only for this reset/demo period.
- [x] Reset saved player ship data to a single core-only ship.
- [x] Reset saved enemy ship data to a single core-only enemy entry or no active prefabs, depending on encounter needs.
- [x] Replace code-seeded enemy prefabs with core-only definitions.

## Phase 2 - Editor Layer Controls

- [x] Add an editor layer state:
  - underlay
  - overlay
- [x] Add UI controls for switching layers.
- [x] Add keyboard shortcuts for switching layers.
- [x] Split toolbox groups so foundation/routes and overlay components are visually distinct.
- [x] Make preview tile rendering respect the active layer.
- [x] Make left-click placement affect only the active layer.
- [x] Make right-click erase affect only the active layer.
- [x] Make selection, marquee select, copy, paste, delete, and move preserve layer identity.
- [x] Make save/load round-trip both layers.

## Phase 3 - Underlay Rendering And Sprites

- [x] Render underlay tiles beneath overlay components.
- [ ] Dim or emphasize layers so the active editing layer is readable.
- [x] Add first-pass sprites or fallback placeholders for:
  - deck/floor
  - power wiring
  - oxygen duct
  - raw salvage pipe
  - repair charge pipe
  - fuel pipe
  - ammunition pipe
  - oxygen resource pipe
  - junction box
  - valve
  - interior wall
  - O2 generator
  - O2 canister storage
- [x] Document all new sprite names and purposes in `assets/tiles/README.md`.

## Phase 4 - Typed Network Graphs

- [ ] Build deterministic adjacency graphs for each infrastructure type.
- [ ] Use cardinal adjacency for same-type route tiles.
- [ ] Attach compatible overlay components to the route tile under or adjacent to them.
- [ ] Treat closed junction boxes and valves as graph blockers.
- [ ] Treat destroyed or missing route tiles as graph breaks.
- [ ] Store per-component network membership for UI, simulation, and ARCH reads.
- [ ] Add tests for stable graph ids or stable graph summaries across repeated rebuilds.

## Phase 5 - Wired Power Runtime

- [ ] Replace global ship power availability with per-network power availability.
- [ ] Let reactors/generators inject power into connected wiring.
- [ ] Let batteries/capacitors buffer only their connected wiring network.
- [ ] Let powered consumers draw only from connected powered networks.
- [ ] Disable consumers with no connected wired power.
- [ ] Report "no wired power" separately from "insufficient generation."
- [ ] Update engine, turret, shield, processor, detector, computer, and drone power checks.
- [ ] Preserve deterministic behavior for rollback.

## Phase 6 - Oxygen Duct Runtime

- [x] Add oxygen as a pipeable/stored resource.
- [x] Add O2 generator behavior that produces oxygen resource.
- [x] Add O2 canister storage that stores oxygen resource.
- [ ] Let oxygen ducts consume connected oxygen resource to replenish atmosphere oxygen.
- [ ] Keep player breathing tied to numeric atmosphere tile oxygen.
- [ ] Keep leakage and equalization tile based.
- [ ] Make closed valves stop duct flow.
- [ ] Add UI/debug readouts for oxygen supply, duct connection, and valve state.

## Phase 7 - Pipeable Resource Runtime

- [x] Expand resource storage/transport to cover:
  - raw salvage
  - repair charge
  - fuel
  - ammunition
  - oxygen
- [x] Add or update storage variants for each resource family.
- [ ] Let reactors consume fuel from connected fuel storage or generators.
- [ ] Let ammunition-using turrets consume ammunition from connected ammo storage.
- [ ] Let processors consume and output compatible resources through connected pipes.
- [ ] Let repair workflows consume repair charge from connected storage where appropriate.
- [ ] Keep existing manual/drone/manipulator logistics interoperable with pipe networks.

## Phase 8 - ARCH Integration

- [ ] Add programmable junction box control registers:
  - open/closed state
  - powered network yes/no
  - local supply/demand summary if available
- [ ] Add programmable valve control registers:
  - open/closed state
  - has oxygen/resource supply yes/no
  - local flow/status summary if available
- [ ] Expose missing-power, missing-resource, and missing-oxygen statuses in compact ARCH-readable form.
- [ ] Add or update ARCH templates that demonstrate emergency isolation and life-support control.

## Phase 9 - UI And Debug Feedback

- [ ] Update component panels to show connected network state.
- [ ] Show why a component is disabled:
  - no wired power
  - no compatible resource
  - closed valve/junction
  - insufficient generation/supply
- [ ] Add editor or runtime debug overlays for:
  - power networks
  - duct networks
  - pipe networks
  - disconnected consumers
  - closed blockers
- [ ] Update HUD summaries that currently assume global ship power or oxygen.

## Phase 10 - Scenario Validation

- [ ] Verify a core-only reset can be redesigned in-game.
- [ ] Build a minimal working ship from scratch with:
  - core
  - floor/hull
  - reactor
  - wiring
  - battery or powered consumer
  - oxygen generator/storage
  - ducting
- [ ] Build a combat-capable ship that requires fuel and ammunition routing.
- [ ] Build an automation example that closes a junction box or valve through ARCH.
- [ ] Verify enemy/core-only encounter handling does not crash even before enemy redesign.

## Test Plan

- [x] Loading after reset produces a valid core-only player ship.
- [x] Loading after reset produces a valid core-only enemy library or safe no-prefab enemy state.
- [x] Editor can place an underlay floor/wire/pipe beneath an overlay component in the same cell.
- [x] Erasing affects only the active layer.
- [x] Selecting and copying preserves layer identity.
- [ ] A wired reactor powers connected consumers.
- [ ] A disconnected consumer is disabled and reports no wired power.
- [ ] ARCH can close and open a junction box.
- [ ] Closing a junction changes which consumers receive power.
- [ ] O2 generator or storage feeds ducts.
- [ ] Ducts raise tile oxygen.
- [ ] Closed valves stop oxygen flow.
- [ ] Reactor fuel, ammunition, raw salvage, repair charge, and oxygen storage only accept compatible resources.
- [ ] Network graph rebuilds deterministically for rollback.

## Test Questions

- [ ] Does the two-layer editor make engineering routes visible without making basic building clumsy?
- [ ] Do wiring and pipes create interesting ship-layout decisions?
- [ ] Can players diagnose an unpowered or unfed component quickly?
- [ ] Do valves and junction boxes create useful ARCH automation opportunities?
- [ ] Does life support feel physical without becoming tedious?

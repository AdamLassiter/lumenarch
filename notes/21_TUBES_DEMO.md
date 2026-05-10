# 21_TUBES_DEMO

## Goal

Add the first real **engineering underlay** for ships: physical wiring, oxygen ducts, and resource pipes beneath the visible ship interior.

This slice moves ship construction away from global "the ship has power" and "the ship has oxygen" abstractions. A working vessel should need visible infrastructure that players route, protect, isolate, and automate.

## Why This Slice Next

The ship already has meaningful components, ARCH automation, resource storage, atmosphere values, and component-local control surfaces.

But several important systems still behave as if the ship is one invisible shared container:

- power generation and power use are summarized globally
- oxygen exists as tile atmosphere, but not as routed life-support infrastructure
- storage is useful, but most consumers do not yet depend on a connected physical supply
- ship layout does not yet express the engineering fantasy of running cables and ducts below the deck

The tubes demo should make ship engineering visible and scriptable.

## Demo Pitch

The player starts from a single ship core and rebuilds the vessel in the editor.

They place hull and deck tiles, then run:

- wiring from reactors, generators, and batteries to powered systems
- oxygen ducts from O2 generation or canister storage to breathable spaces
- resource pipes between storage and components that consume fuel, ammunition, repair material, salvage, or oxygen

Components no longer benefit from a magic whole-ship pool. A reactor with no fuel pipe starves. A turret with no powered wiring and no ammunition route cannot fight. A sealed room with no oxygen duct eventually becomes a bad place to stand.

ARCH programs can control junction boxes and valves to isolate faults, preserve resources, or change ship posture during combat.

## Core Design Direction

Infrastructure should be physical, local, and automatable.

That means:

- route tiles occupy a real editor layer beneath components
- routes connect by grid adjacency
- components only consume what their connected network can provide
- valves and junction boxes can close networks programmatically
- damage, redesign, and poor routing can create meaningful failures
- debug views and UI should explain the active networks clearly enough to iterate in-game

The player should feel the difference between:

- a ship that merely has enough equipment installed
- a ship whose equipment is actually connected
- a ship whose infrastructure can be isolated and reconfigured under ARCH control

## Two-Layer Ship Model

The editor should support two saved layers.

### Underlay Layer

The underlay contains infrastructure and walkable foundation:

- hull shell / hull edges
- deck or floor tiles
- power wiring
- oxygen ducts
- typed resource pipes

Underlay tiles may exist beneath overlay components at the same grid coordinate.

### Overlay Layer

The overlay contains objects that occupy the room:

- ship core
- reactors and generators
- batteries and capacitors
- computers and detector modules
- processors
- storage components
- engines, turrets, shields, airlocks, and other major components
- interior walls
- programmable junction boxes and valves

Overlay components may sit on top of compatible underlay tiles, but should not implicitly create all infrastructure they need.

## Save Schema Direction

The first pass may drop compatibility with old saved ships because this demo intentionally resets the fleet.

Recommended saved shape:

- `foundation_tiles: Vec<ShipFoundationTile>`
- `modules: Vec<ShipModule>`

`ShipFoundationTile` should carry:

- stable id
- foundation kind
- grid coordinate
- rotation
- optional typed route/resource information if not represented by kind

`ShipModule` remains the overlay component record. Interior walls should be represented as overlay build items so they can block movement and atmosphere without becoming deck foundation.

The old single-cell exclusivity rule should be replaced with:

- at most one underlay tile per grid cell
- at most one overlay tile per grid cell
- one underlay and one overlay tile may coexist

## Save And Prefab Reset

Existing player ship saves and prefabricated enemy ships should be dropped for this slice.

The replacement seed should be a single valid ship core:

- one core module
- no cockpit requirement during this demo reset period
- no prebuilt enemy layouts

This keeps the migration honest. The editor becomes the source of truth for redesigning ships around infrastructure instead of trying to preserve layouts built for the old global systems.

## Network Rules

Networks are typed tile graphs built from the underlay.

Recommended first-pass network types:

- power wiring
- oxygen ducting
- raw salvage pipe
- repair charge pipe
- fuel pipe
- ammunition pipe
- oxygen resource pipe

Connections should be deterministic:

- cardinal adjacency only
- same compatible type connects
- closed valves and junction boxes split networks
- destroyed or missing foundation tiles break networks
- overlay components attach to matching underlay route tiles at their grid cell or adjacent cells

This should start as a simple graph rebuild from saved/runtime tile state. It can become incremental later if needed.

## Power Migration

Global ship power should be replaced by routed power.

Power producers:

- reactors
- generators if added later
- batteries/capacitors as storage or buffers

Power consumers:

- engines
- turrets
- shields
- processors
- computers
- detectors
- drones and future powered equipment

First-pass behavior:

- producers inject into connected power wiring
- consumers draw only from their connected powered network
- batteries/capacitors buffer only the network they are connected to
- components without required wired power are disabled
- UI reports "no wired power" distinctly from "insufficient generation"

Junction boxes are overlay components connected to wiring. ARCH registers should expose at minimum:

- open/closed state
- network has power yes/no
- local supply/demand summary if available

## Oxygen Migration

Oxygen should exist in two related forms:

- **oxygen resource** stored in canisters or generated by O2 equipment
- **atmosphere oxygen** as the numeric per-tile breathable value

O2 generators and O2 canister storage feed connected oxygen infrastructure.

Ducts consume oxygen resource and replenish atmosphere on reachable interior tiles. Valves are overlay components that can close duct flow.

First-pass behavior:

- rooms still use tile oxygen values for player breathing
- oxygen equalization/leaking remains tile based
- ducts add oxygen into connected breathable areas
- closed valves stop duct flow
- damaged or missing ducts stop replenishment

Valves should be ARCH-controllable from the first slice. Manual UI can be added if it is cheap, but the automation surface is the important part.

## Pipeable Resources

Every current resource should become pipeable, and oxygen should join that resource family.

Pipeable resource set:

- raw salvage
- repair charge
- fuel
- ammunition
- oxygen

Each pipeable resource needs an appropriate storage component or storage variant:

- general cargo for raw salvage and repair charge
- fuel tank for reactor fuel
- ammunition rack for ammunition
- O2 canister storage for oxygen resource

Consumers should draw from connected compatible networks:

- reactors consume fuel
- turrets consume ammunition where applicable
- processors consume raw salvage and output repair charge, fuel, or ammunition
- repair systems consume repair charge
- oxygen ducts consume oxygen resource to create atmosphere oxygen

The important rule is that "connected and compatible" replaces "globally available."

## Editor Experience

The editor should make layers explicit without making basic construction painful.

Expected controls:

- layer toggle between underlay and overlay
- toolbox groups for foundation, routing, components, and walls
- preview tile respects active layer
- erase affects active layer
- selection defaults to active layer
- copy/paste preserves layer identity
- save/load persists both layers

Rendering should show:

- underlay beneath overlay
- active-layer emphasis while editing
- optional dimming of inactive layer
- route type readability
- selected network/debug overlays when available

## UI And Debugging

The player needs to understand why something is failing.

Useful readouts:

- powered / unpowered
- connected network id or local network summary
- supply, demand, and reserve for power networks
- resource present / missing for pipe consumers
- duct oxygen supply and active valve state
- closed junction or valve state

Useful debug overlays:

- power network coloring
- oxygen duct network coloring
- resource pipe network coloring
- disconnected consumers
- blocked valves and junction boxes

## Art Requirements

Any new sprite added for this slice should be documented in [assets/tiles/README.md](/home/adaml/code/lumenarch/assets/tiles/README.md).

The first expected sprite families are:

- wiring
- oxygen ducts
- typed resource pipes
- junction boxes
- valves
- interior walls
- O2 generator
- O2 canister storage
- resource-specific storage variants

## Risks / Gaps

- the save-schema change touches many systems at once
- editor selection and copy/paste need to avoid accidentally merging layers
- old atmosphere and power systems must be bridged carefully during migration
- debug visibility is essential because invisible network failure will feel unfair
- deterministic graph construction matters for rollback

## Success Criteria

- the editor can place underlay infrastructure beneath overlay components
- a core-only reset loads cleanly for player and enemy ships
- a component without connected power is disabled
- connected storage can feed compatible consumers through typed pipes
- O2 ducts can replenish breathable atmosphere from oxygen resource supply
- ARCH can close and open junction boxes and valves
- the player can diagnose missing power, oxygen, or resource routing from UI/debug feedback

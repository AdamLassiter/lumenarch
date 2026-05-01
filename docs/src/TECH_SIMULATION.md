# TECH_SIMULATION — Runtime Simulation Model

## Purpose

This document refines the simulation layer described in `TECH_DESIGN.md`.

The simulation model must be valid for both:

* single-player local-host sessions
* deterministic rollback multiplayer sessions

## Two Simulation Spaces

LUMEN//ARCH runs in two coordinate spaces at once:

* **world space** for ship movement, weapons, radiation, shields, and other free-space interactions
* **vessel-local tile space** for enclosed interiors, oxygen, and ambient heat

The bridge between them is the vessel root transform.

## Vessel-Local Tile Data

Each vessel root owns a dense tile structure:

```text
InternalTileMap {
  dimensions,
  hull_mask,
  solid_mask,
  room_ids,
  oxygen,
  ambient_heat,
  walkable,
  dirty_regions
}
```

Recommended storage:

* flat vectors indexed by `(x, y)`
* room ids cached after topology rebuild
* dirty bitsets for partial recomputation where possible
* fixed-point or integer-backed numeric arrays for simulation values

## Topology Lifecycle

Topology is rebuilt when structure changes:

* component placed or removed
* hull damaged or repaired
* doors/openings toggled

Rebuild output:

* enclosure membership
* breach connectivity
* room graph
* walkability refresh

Emit `VesselTopologyChanged` after rebuild so atmosphere and pathing can react.

## Oxygen Simulation

Oxygen is not a world field. It is a per-tile scalar volume.

Per fixed tick:

1. apply producer injections from life support or tanks
2. equalize oxygen across connected enclosed tiles
3. vent oxygen through breaches and exposed edges
4. apply consumption from actors or systems if needed
5. clamp values and write back

Sampling:

* locate the tile containing an actor/component center
* read local oxygen value
* apply survival or suffocation effects

## Ambient Heat Simulation

Ambient heat inside ships is also tile-based.

Heat sources:

* reactors
* engines
* weapons
* fires

Heat sinks:

* radiators
* open breaches to space
* specialized coolant systems

Per fixed tick:

1. inject source heat into source tiles
2. diffuse heat across connected enclosed tiles
3. remove heat through sinks
4. sample tile heat onto actors and components

Important distinction:

* **ambient tile heat** is room/interior temperature
* **world heat fields** are free-space hazards such as exhaust, beams, or explosions

Both domains should use deterministic numeric types in gameplay code.

## World-Space Field Pipeline

Ordinary fields follow a separate pipeline:

1. active emitters produce field descriptors
2. descriptors are inserted into a spatial index
3. samplers query nearby fields by world position
4. matching systems consume aggregated values

Field descriptors should be lightweight:

```text
FieldDescriptor {
  field_type,
  shape,
  origin,
  direction,
  intensity,
  falloff,
  source_entity,
  source_vessel
}
```

For networked determinism:

* field descriptors should store fixed-point values, not floats
* insertion and sampling order must be stable

## Component And Actor Sampling

Sampling priority:

* if the target is inside a vessel, sample vessel-local tile values first for oxygen and ambient heat
* always sample world-space fields for radiation, thrust, shields, electrical effects, and beams

This means a component can be influenced by both:

* tile heat from the room it sits in
* world radiation from a nearby external source

## Suggested FixedUpdate Order

1. rebuild topology for dirty vessels
2. finalize committed input for the current tick
3. refresh vessel register snapshots
4. run ARCH and LUMEN evaluation
5. plan logistics reservations
6. resolve activation and power availability
7. emit world-space fields
8. step oxygen and ambient heat tile simulation
9. sample fields/tile volumes onto components and actors
10. resolve movement, weapons, and damage
11. commit deferred state writes
12. compute optional state hash or checkpoint data

## Data Ownership

Keep ownership clear:

* vessel roots own tile maps, topology, and vessel-wide caches
* component entities own module-specific state
* resources own global indexes and configuration
* events announce transitions across those ownership boundaries

This prevents systems from scattering vessel state across unrelated resources.

## Determinism Guardrails

The simulation layer should enforce a few non-negotiable guardrails:

* no floating-point branching in gameplay-critical systems
* no unordered accumulation where result order matters
* no dependence on wall-clock time or render frame count
* no nondeterministic RNG access outside explicit seeded streams

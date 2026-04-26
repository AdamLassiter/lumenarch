# TECH_DESIGN — LUMEN//ARCH

## Purpose

This document defines the high-level technical architecture for **LUMEN//ARCH** in Rust using Bevy.

The design goals are:

* Keep simulation deterministic at the gameplay layer
* Preserve the fantasy of modular ships, physical logistics, and programmable systems
* Use Bevy ECS where entity composition is valuable
* Avoid modeling dense ship-internal data as thousands of tiny entities when a vessel-local data structure is a better fit
* Separate frame-rate rendering from fixed-step simulation
* Support both single-player and multiplayer from the same runtime model
* Prefer reliable low-bandwidth synchronization over rollback-first netcode
* Use fixed-point arithmetic and stable deterministic rules so peers can stay in sync

## Core Technical Direction

The runtime is split into two complementary models:

* **Bevy ECS world model** for ships, components, actors, projectiles, wrecks, drones, zones, and UI-facing state
* **Dense vessel-local simulation data** for ship tiles, enclosure topology, oxygen, ambient heat, and other high-frequency internal grids

Multiplayer uses the same simulation core in all modes:

* **single-player** = one local process running both authoritative simulation and local client presentation
* **multiplayer** = one host or dedicated server runs the authoritative simulation while clients run the same deterministic simulation locally for prediction, validation, and low-bandwidth sync

This lets us use Bevy naturally for gameplay objects while keeping internal ship simulation cache-friendly and deterministic.

## Bevy App Structure

Planned top-level plugins:

* `CoreGamePlugin`: app state, save/load bootstrap, shared schedules
* `NetcodePlugin`: session model, transport integration, replication, sync barriers
* `SectorPlugin`: sector map, travel, encounter spawning
* `VesselPlugin`: vessel roots, tile maps, topology, transforms
* `ComponentPlugin`: ship modules, health, activation, placement
* `FieldPlugin`: world-space fields, emitters, samplers
* `AtmospherePlugin`: oxygen and ambient heat tile simulation
* `PowerPlugin`: generation, draw, reserves, brownouts
* `LogisticsPlugin`: inventories, reservations, manipulators, processors
* `ArchPlugin`: register graph, script runtime, component writes
* `LumenPlugin`: BUFF/NERF influence and budget resolution
* `CombatPlugin`: weapons, damage, projectiles, shields
* `ActorPlugin`: player avatar, drones, boarding actors
* `AiPlugin`: hostile ship behavior and encounter logic
* `UiPlugin`: overlays, inspectors, automation tooling

## States And Schedules

High-level app states:

* `Boot`
* `MainMenu`
* `Loading`
* `InSector`
* `Docked`
* `Paused`

Simulation should primarily run in `FixedUpdate` using a stable gameplay tick.

For multiplayer, each simulation tick must have:

* a monotonic `TickId`
* an agreed input frame for each connected player
* deterministic ordering for all system execution and state commit
* optional sync checkpoints for hash validation

Suggested system sets inside `FixedUpdate`:

1. `InputIntent`
2. `Topology`
3. `RegistersRead`
4. `Automation`
5. `LogisticsPlanning`
6. `PowerAndActivation`
7. `FieldEmission`
8. `TileSimulation`
9. `MovementAndPhysics`
10. `CombatResolution`
11. `DamageAndFailures`
12. `StateCommit`
13. `EventCleanup`

`Update` should drive presentation work:

* camera
* UI
* audio
* interpolation
* debug overlays

In networked play:

* `FixedUpdate` is the gameplay truth path used by the host and mirrored by clients
* `Update` is the presentation path used to render interpolated or predicted state
* local input capture may happen before simulation, but intent should be stamped for a future tick before entering the deterministic pipeline

## Multiplayer Model

The preferred architecture is **authoritative host/server plus deterministic input replication**.

High-level rules:

* the host is authoritative for accepted inputs, world generation seeds, and resync checkpoints
* clients send **player intent**, not high-frequency state transforms
* all peers step the same deterministic simulation once a tick's input set is committed
* clients present locally predicted visuals where helpful, but committed simulation advances only on agreed tick boundaries

This keeps bandwidth low and makes desync detection tractable.

### Why Not Pure Rollback-First

Full rollback netcode is possible, but it should not be the default architecture.

Reasons:

* the simulation is broad and stateful: ships, tile atmospheres, logistics, scripting, projectiles, drones, and fields
* rewinding and replaying large vessel-local state every few frames would be expensive and complex
* the game is more about deliberate systems control than frame-perfect fighting-game reactions

Rollback may still be useful in limited areas:

* local avatar responsiveness
* cursor-driven UI interactions
* small client-side presentation corrections

But the main simulation should not depend on global rollback for correctness.

### Preferred Input Commitment Model

Each player submits input intents for a future tick window.

Example flow:

1. client captures local intent on frame `N`
2. client packages that intent for simulation tick `N + input_delay`
3. host validates and relays the committed intent set
4. every peer simulates tick `N + input_delay` once all required intents are known or defaulted
5. periodic hashes verify that peers remain synchronized

This is closer to deterministic lockstep with input delay than to rollback.

### Handling Latency

Latency should be handled with layered techniques instead of immediate full rollback:

* short configurable input delay for committed simulation
* local client-side preview for movement cursors, interaction outlines, and UI
* optional actor-level prediction for the local controlled avatar
* reconciliation against committed host results
* explicit resync barriers when simulation hashes diverge

For this game, a small input delay is an acceptable tradeoff if it keeps the authoritative simulation stable.

## Entity Model

### Vessel Root

Each ship, station, or large derelict is represented by a root entity.

Typical vessel-root components:

* `VesselId`
* `VesselFaction`
* `Transform`
* `GlobalTransform`
* `LinearVelocity`
* `AngularVelocity`
* `VesselMass`
* `VesselGrid`
* `VesselTopology`
* `InternalTileMap`
* `PowerState`
* `LogisticsNetworkState`
* `RegisterBank`
* `AutomationHosts`

The vessel root owns the dense data needed for internal simulation.

### Component Entities

Ship modules should be ECS entities parented to a vessel root.

Typical component entity components:

* `ComponentId`
* `VesselRef`
* `GridFootprint`
* `LocalTransform`
* `Integrity`
* `Powered`
* `ActivationState`
* `FieldEmitterSet`
* `RegisterInterface`
* type-specific markers such as `Reactor`, `Engine`, `Weapon`, `Shield`, `ArchComputer`, `LumenProcessor`, `LogisticsStorage`

This keeps inspection, damage, scripting, and content authoring modular.

### Actor Entities

Actors that move independently inside or outside ships remain ECS entities:

* player avatar
* drones
* boarding enemies
* floating salvage
* projectiles

These entities sample either vessel-local tile data or world-space fields depending on where they are.

## Dense Vessel Data

The following should live in vessel-local components or structs, not as per-tile entities:

* hull occupancy grid
* room / enclosure graph
* oxygen values per tile
* ambient heat values per tile
* static walkability
* local resource routing metadata
* dirty masks for topology and atmosphere recomputation

Rationale:

* better cache locality
* lower entity count
* easier deterministic stepping
* simpler save serialization

## Determinism Requirements

Reliable netcode depends on deterministic simulation discipline.

Mandatory rules:

* no gameplay-critical floating point arithmetic
* fixed tick rate with no frame-time-derived simulation branches
* stable iteration order for all ECS queries that affect game state
* deterministic random generation from explicit seeded streams
* deterministic entity and component identifiers inside saved/networked state
* deterministic serialization for snapshots and hash generation

### Numeric Model

Gameplay simulation should use fixed-point numbers for:

* vessel movement and force accumulation
* field intensity and falloff
* oxygen and ambient heat values
* logistics quantities
* power, damage, cooldowns, and timers

Suggested approach:

* represent canonical gameplay values as integer-backed fixed-point types
* use floating point only for rendering, camera smoothing, particles, and other presentation-only paths
* convert from deterministic simulation space to visual space at the boundary between `FixedUpdate` and `Update`

### Deterministic ECS Practices

Bevy is compatible with deterministic simulation if we are careful about state mutation order.

Rules:

* never rely on arbitrary query iteration order when order affects outcomes
* sort by stable ids before applying coupled effects
* batch writes into explicit command/state buffers where contention is possible
* commit all deferred writes in a known order during `StateCommit`
* keep side effects out of debug-only systems

## Mapping To Bevy Concepts

### Entities

Use entities for:

* vessels
* ship modules
* actors
* weapons/projectiles
* world salvage
* field emitters when they need independent identity

### Components

Use components for:

* identity and ownership
* simulation state that belongs to one gameplay object
* authoring data and runtime state for modules
* tags that drive system queries

Examples:

* `ReactorStats`
* `EngineNozzle`
* `ShieldArc`
* `StorageInventory`
* `ManipulatorState`
* `ProcessorRecipeState`
* `ArchProgram`
* `LumenBudget`
* `DetectorContacts`

### Resources

Use resources for global or cross-entity coordination:

* `SimulationConfig`
* `FieldSpatialIndex`
* `RegisterSchemaRegistry`
* `ScriptCache`
* `EncounterDirector`
* `AssetDerivedComponentCatalog`
* `DebugOverlayState`
* `NetworkSession`
* `NetworkTickClock`
* `SnapshotPolicy`

Resources should not own per-vessel gameplay state that needs save/load isolation unless they are acting as indexed caches.

### Events

Use events for discrete transitions, requests, and notifications:

* `VesselTopologyChanged`
* `ComponentPlaced`
* `ComponentRemoved`
* `ComponentDamaged`
* `HullBreached`
* `ArchWriteCommitted`
* `LumenEffectApplied`
* `LogisticsReservationCreated`
* `LogisticsReservationReleased`
* `WeaponFired`
* `ProjectileHit`
* `ActorEnteredVessel`
* `JumpRequested`
* `JumpResolved`
* `PlayerIntentReceived`
* `TickCommitted`
* `SnapshotRequested`
* `SnapshotApplied`
* `DesyncDetected`
* `ResyncStarted`
* `ResyncCompleted`

Events should trigger reactions, not hold durable state.

## Register And Automation Architecture

### Register Model

The canonical addressing model remains:

```text
[Category][Type][Property][Channel]
```

Logistics is standardized under category `L`:

* `LS` storage
* `LM` manipulator
* `LP` processor

### ARCH Runtime

ARCH should be implemented as a deterministic fixed-step interpreter:

* read current register snapshot
* execute program with bounded instruction count
* accumulate writes into a next-state command buffer
* commit writes during `StateCommit`

This matches the design docs and avoids order dependence between multiple computers.

For multiplayer, ARCH programs must also be network-safe:

* identical program text and memory state must produce identical writes on every peer
* backward-jump halting behavior must be exact
* register reads must come from the same committed snapshot on all peers

Key components/resources:

* `ArchProgram`
* `ArchMemoryBank`
* `ArchExecutionState`
* `RegisterBank`
* `PendingRegisterWrites`
* `ScriptCache`

### LUMEN Runtime

LUMEN is not a direct writer. It computes modifiers on top of component properties.

Resolution model:

* evaluate active `BUFF` and `NERF` instructions
* identify in-range matching registers
* apply falloff and budget rules
* write modifiers into transient effect buffers
* consume those modifiers in downstream simulation systems

This is best represented as transient components or vessel-local caches cleared each tick.

LUMEN resolution must also use stable ordering:

* sort matching targets by stable id before budget allocation
* use deterministic tie-breaking for equal-cost candidates
* avoid parallel reduction patterns that can reorder fixed-point accumulation

## Simulation Domains

### World-Space Fields

Use world-space field evaluation for:

* radiation
* shield projection
* thrust / force
* beam weapons
* EMP / electrical effects
* LUMEN influence radius
* ARCH control radius

Implementation shape:

* components emit field descriptors
* fields are inserted into a spatial index resource
* samplers query by world position and field type

For deterministic multiplayer, field aggregation must:

* use stable iteration order
* use fixed-point accumulation
* avoid nondeterministic spatial hash traversal affecting final sums

### Vessel Tile Simulation

Use vessel-local tile simulation for:

* oxygen
* ambient heat equalization
* enclosure detection
* breach propagation

Sampling rule:

* components and actors are affected by the tile under their center point

This matches the agreed design and avoids awkward free-floating oxygen fields.

Tile simulation is a good fit for multiplayer because:

* vessel-local arrays are compact to hash
* topology changes are explicit and event-driven
* resync can replace one vessel's interior state without rebuilding the whole world

## Logistics Architecture

Logistics has three layers:

1. Storage state
2. Reservation planning
3. Physical transfer execution

Suggested model:

* inventories live on storage component entities
* reservation tables live on vessel-local logistics state
* manipulators and drones execute transfer jobs as entities/systems

This preserves the “resources do not teleport” rule while still allowing planning to be centralized per vessel.

For deterministic networking:

* reservation allocation must use stable ordering
* task queues must be priority- and id-sorted
* cargo quantities should use fixed-point or integer unit counts

## Physics And Vessel Motion

Ships should move as rigid aggregate bodies at the vessel-root level.

Derived values:

* total mass from structure and cargo
* thrust vectors from active engines
* torque from engine placement relative to center of mass

Bevy role split:

* vessel root stores physical state
* engine component entities contribute forces
* movement systems integrate root transform in fixed time

Component-local transforms remain relative to the vessel root.

Host authority should remain final for vessel motion, but clients can safely mirror movement deterministically if the numeric model is fixed-point and all force aggregation is stable.

## Damage And Failure Model

Damage should resolve in layers:

1. shields mitigate incoming effects
2. hull/components receive damage
3. failures update registers, emit hazards, and dirty topology if needed

Useful components/events:

* `Integrity`
* `FailureMode`
* `Overheated`
* `Irradiated`
* `Disabled`
* `HullBreached`
* `ComponentDestroyed`

## Data Authoring

Static game data should be asset-driven:

* component definitions
* recipes
* field profiles
* scriptable register schemas
* AI loadouts

Prefer serializable data assets over hard-coded Rust enums where designers will need iteration.

## Save Model

Save data should persist:

* vessel roots and component layouts
* inventories
* script programs and memory banks
* tile simulation arrays
* sector state
* actor state

Do not persist ephemeral caches such as spatial indices or per-frame field aggregates.

The same snapshot structure should be usable for:

* save files
* host-to-client resync payloads
* late-join world bootstrap

## Desync Detection And Recovery

Desyncs should be treated as a first-class engineering concern.

Recommended flow:

1. compute compact deterministic state hashes at regular tick intervals
2. compare peer hashes against host hashes
3. if hashes differ, identify the affected vessel or sector scope where possible
4. pause forward prediction for the affected scope if needed
5. apply authoritative snapshot data from the host
6. resume from the repaired tick barrier

The design should prefer **resync barriers** over deep rollback for large-world correction.

## Single-Player As Local Multiplayer

Single-player should run the same architecture in-process:

* one local authoritative host
* one local client
* zero network latency path through the same intent pipeline

This keeps single-player and multiplayer behavior aligned and reduces the number of simulation modes we must maintain.

## Initial Implementation Order

1. Vessel root, component placement, and internal tile map
2. Fixed-step schedule and register bank
3. Basic power + engine + movement loop
4. Oxygen and ambient heat tile simulation
5. ARCH runtime with a minimal register set
6. Storage/manipulator/processor logistics
7. Combat, shields, and damage
8. LUMEN modifiers
9. Drones, boarding, and late-game scaling

## Follow-On Detail Docs

This document is intentionally high-level. Detailed follow-ups should live beside it, starting with:

* `TECH_SIMULATION.md` for field and tile simulation order
* `TECH_NETCODE.md` for deterministic networking, sync, and recovery
* future docs for save format, scripting runtime, and content pipelines

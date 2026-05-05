# 19_DRONES_DEMO

## Goal

Add a first real **drone logistics** layer that extends the existing storage / manipulator / processor loop instead of replacing it.

This slice should follow the direction in [docs/src/LOGISTICS.md](/home/adaml/code/lumenarch/docs/src/LOGISTICS.md), especially the late-game drone section:

- drones are mobile manipulators with autonomy
- they extend logistics range
- they operate on the same reservation / transport / consume model
- they are enabled by a dedicated **drone station**

## Why This Slice Next

The logistics model is now rich enough that adjacency and manipulator range create meaningful bottlenecks. Drones are the next scaling step promised by the concept and logistics doc.

They should open:

- longer-range cargo movement
- more flexible salvage recovery
- more advanced onboard production chains
- late-game scaling without removing the importance of local layout

## Demo Pitch

The player can install a **drone station** that spawns a limited number of drones. Those drones move physical resources between storage, processors, airlocks, and other drone-aware work points within a controlled radius.

Manipulators remain valuable for short-range local routing. Drones extend that routing across the ship or a nearby operation area.

## Scope

### 1. Drone Station Component

Add a new ship component family:

- `Drone Station`

Responsibilities:

- spawns and manages drones
- exposes max drone count
- exposes mode / enable state
- exposes equipment/readiness state if simplified for the first pass

This should be a real runtime module with UI and ARCH-facing surface, not a hidden subsystem.

### 2. Drone Entities

Add drone actors/entities that:

- have a position and movement model
- can reserve and carry resources
- can travel between source and destination
- can idle, travel, pick up, deliver, and return

First pass should keep drones simple:

- logistics-only
- no combat role
- no boarding role
- no full damage model beyond maybe simple disable/despawn if needed

### 3. Logistics Integration

Drones should use the same underlying resource model as existing logistics:

- no teleportation
- resources still live in inventories
- reservation and transfer semantics remain intact

Good first jobs:

- move raw salvage from intake/airlock storage to cargo storage
- move raw salvage from storage to processor
- move processor output to suitable storage
- optionally move ammo/fuel/repair material where needed

### 4. Drone Station UI / Registers

Follow the doc’s spirit with a first-pass control surface:

- active drones
- max drones
- drone mode
- status / task summary
- power use if modeled

This should appear:

- in component UI
- in ARCH-facing registers / channel-driven surfaces where appropriate

### 5. Range / Field Constraint

Drones should not be magic global movers.

First pass should define:

- station operational radius
- where drones can path
- what endpoints count as valid pickup/drop targets

This can start simpler than the full vision, but it should preserve the idea that drones operate in space and range, not as abstract inventory workers.

## Design Rules

### 1. Drones enhance, not replace, logistics

Manipulators and careful adjacency should still matter.

### 2. Drones must stay physical

They are entities doing transport work, not invisible throughput multipliers.

### 3. Start with logistics, not combat

Keep scope focused on cargo movement and late-game scaling first.

## Risks / Gaps

- pathing and endpoint selection need to stay deterministic if brought into rollback simulation
- drone task assignment can easily become an AI system too broad for the first slice
- the reservation model must stay robust under multiple drones targeting similar resources

## Success Criteria

- installing a drone station materially changes logistics throughput/range
- drones visibly move resources rather than simulating invisible transfer
- manipulator-only and manipulator-plus-drone ships feel meaningfully different
- the system still matches the underlying logistics design from `LOGISTICS.md`

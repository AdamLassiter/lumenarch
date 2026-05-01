# TECH_NETCODE — Deterministic Multiplayer And Sync

## Purpose

This document defines the preferred multiplayer model for **LUMEN//ARCH**.

The primary goal is reliability:

* deterministic simulation first
* rollback-driven peer synchronization through `bevy_ggrs`
* compact per-player intent replication
* strong checksum-backed desync detection

## Networking Philosophy

LUMEN//ARCH now uses a rollback-oriented peer session built on `bevy_ggrs`.

Current runtime model:

* listen-host UX for creating and coordinating a session
* GGRS P2P session for synchronized gameplay authority
* deterministic peers advancing the same rollback simulation
* compact input frames as the only synchronized gameplay command stream
* periodic checksum validation for debugging and desync diagnosis

## Session Roles

Supported runtime roles:

* `Host`: creates the session, defines the peer list, and loads the initial deterministic snapshot
* `Client`: joins the host-provided peer session and runs the same rollback simulation

Current constraints:

* all players must be present before the session starts
* no late join
* no host migration
* no separate authoritative snapshot/resync protocol outside GGRS rollback behavior

## Tick Model

Simulation runs at a fixed tick rate.

Each tick has:

* `TickId`
* committed player intents
* deterministic RNG cursor state
* optional state hash

Suggested tick flow:

1. collect local hardware input in `ReadInputs`
2. submit compact `PlayerGgrsInput` frames through GGRS
3. advance deterministic simulation in `GgrsSchedule`
4. save rollback snapshots and checksums
5. expose presentation-only state after rollback advancement

`D` remains the configured local input delay.

## Input Replication

Peers should send intents, not simulation results.

Examples of replicated intent:

* move direction
* interact command
* selected target
* fire trigger
* UI-confirmed component command
* script upload or edit action

Examples of state that should not normally be sent every tick:

* full vessel transform
* oxygen tile arrays
* logistics inventory contents
* projectile positions

Those should emerge from deterministic rollback simulation.

## Input Delay

Committed simulation should use a small input delay.

Why:

* absorbs ordinary network jitter
* reduces immediate need for rollback
* gives all peers a clean committed input set

Tradeoff:

* slightly slower response in committed game state
* much stronger stability and lower correction churn

This trade is appropriate for LUMEN//ARCH.

## Local Prediction

Some prediction is still useful.

Safe prediction targets:

* local cursor feedback
* interaction previews
* movement reticles
* UI selection state
* local avatar presentation smoothing

Riskier prediction targets:

* logistics reservations
* script-driven component behavior
* projectile outcomes
* topology-changing damage

Those should generally wait for committed simulation or be very carefully reconciled.

## When Rollback Is Worth Considering

Rollback should be optional and scoped.

Good candidates:

* local actor locomotion feel
* aim visualization
* short-horizon player action preview

Poor candidates:

* full-vessel state
* atmospheric tile volumes
* logistics networks
* multi-ship combat state

If rollback is introduced, it should be limited to a narrow prediction layer rather than the full game world.

## Deterministic Numeric Rules

Gameplay simulation must use fixed-point arithmetic.

Priority domains:

* positions and velocities
* force and torque
* field strengths
* oxygen and ambient heat
* power and heat budgets
* timers and cooldowns
* resource quantities

Requirements:

* fixed scaling must be globally defined
* overflow handling must be explicit
* rounding behavior must be identical on all peers
* serialization must preserve exact values

Presentation systems may still use floating point after converting from the deterministic state.

## Deterministic Ordering Rules

Netcode reliability depends on stable ordering as much as numeric precision.

Required practices:

* sort all coupled-effect targets by stable id
* use stable priority queues for logistics and AI tasks
* never rely on hashmap iteration for gameplay outcomes
* never rely on scheduler parallelism for commutative-looking but order-sensitive writes
* resolve ties with explicit deterministic rules

## Snapshot Strategy

Maintain snapshot history for:

* host recovery and resend
* late join
* desync correction
* debugging

Recommended layers:

* `full snapshot`: authoritative world or sector state
* `scoped snapshot`: one vessel or encounter
* `delta snapshot`: optional later optimization once correctness is proven

Early implementation should prefer correctness over compression sophistication.

## Hash Validation

Periodically compute deterministic hashes over gameplay state.

Good hash inputs:

* vessel root state
* component states
* inventories
* register banks
* tile arrays
* actor states
* active projectiles

Avoid including:

* particle state
* UI state
* camera state
* debug-only resources

Hash cadence can be lower than the simulation tick if needed, for example every 5 or 10 ticks.

## Desync Recovery

Recommended recovery flow:

1. host detects client hash mismatch
2. host identifies the smallest useful correction scope
3. host sends authoritative snapshot for that scope
4. client pauses affected simulation scope
5. client replaces local state
6. client resumes on the next barrier tick

Preferred correction scope order:

1. single vessel
2. local encounter
3. whole sector
4. full world

## Transport Expectations

Transport should support:

* reliable ordered messages for session and snapshot control
* unreliable or sequenced messages for frequent intent packets if desired
* connection health and RTT measurement

The transport layer should be abstracted so gameplay systems do not care whether the session is:

* local loopback
* peer-hosted
* dedicated server hosted

## Late Join And Rejoin

Joining players should not reconstruct the world from historical inputs alone.

Preferred flow:

1. host sends current authoritative snapshot
2. client loads deterministic state
3. client receives current tick clock and seed state
4. client begins consuming committed future intents

This is simpler and safer than replaying the whole session.

## Debugging And Tooling

Deterministic multiplayer needs dedicated tooling from the start.

Useful tools:

* per-tick state hash logging
* snapshot diff viewer
* deterministic replay capture
* input timeline inspector
* vessel-state dump utilities
* forced desync test harness

## Recommended Initial Scope

For the first multiplayer-capable milestone:

1. implement fixed tick clock and intent buffering
2. keep one authoritative host
3. mirror deterministic simulation on clients
4. add periodic vessel or sector hash checks
5. add scoped snapshot resync
6. delay rollback experimentation until drift-free sync is proven

This keeps the first version reliable and understandable.

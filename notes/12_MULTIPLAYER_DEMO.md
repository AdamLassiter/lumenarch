# MULTIPLAYER_DEMO — Twelfth Vertical Slice

## Goal

Ship the first working rollback multiplayer slice for **LUMEN//ARCH** using `bevy_ggrs`, with enough real game flow to prove the architecture instead of only proving packet transport.

This slice now validates:

* a host and at least one client can bootstrap into the same session
* synchronized docked, sector-map, and encounter phase transitions work
* encounter simulation runs through a shared-crew, one-ship model
* rollback state is the authoritative in-session source of truth
* the project has useful logging and diagnostics for session bootstrap, phase changes, and rollback inputs

The loop this slice now proves is:

`Open Lobby -> Join Lobby -> Host Starts Session -> Shared Docked / Sector Flow -> Shared Encounter -> Return To Dock`

## What Changed During Implementation

This slice started from a custom host/client networking stack and moved through a deeper architectural rewrite than originally planned.

The final direction for this slice is:

* the old custom TCP gameplay sync stack was removed from active runtime use
* `bevy_ggrs` now owns synchronized gameplay/session progression
* a separate pre-session lobby/bootstrap channel exists so the host can discover connected clients and broadcast the final peer list
* rollback phase is the only in-session phase authority
* presentation/UI mirrors rollback state instead of fighting it

That means this slice became both:

* a multiplayer feature slice
* a simulation-architecture migration slice

## Final Demo Pitch

One host and one or more clients can enter a pre-session lobby, start a shared rollback session, move together through docked and sector-map flow, and then enter the same encounter where multiple crew actors operate the same ship.

The host controls synchronized outer-loop decisions:

* opening the sector map
* entering/leaving the ship editor
* selecting a sector node
* launching an encounter
* returning to dock

All peers then simulate the same in-session rollback state, and the local presentation follows that state instead of maintaining a second app-phase machine.

## What This Slice Actually Implements

### 1. Rollback Session Ownership

The synchronized runtime now uses:

* `PlayerGgrsInput` as the canonical per-peer gameplay/meta input packet
* `RollbackGameState` as authoritative in-session state
* `RollbackPhase` for:
  * `Docked`
  * `SectorMap`
  * `Editing`
  * `Encounter`

`FrontendMode` remains only as a shell router:

* `Menu`
* `Session`
* `DebugEnemyEditor`

This removed the old dual runtime state fight that caused UI flicker and client desync around phase changes.

### 2. Pre-Session Lobby / Bootstrap Channel

The project now has a lightweight pre-session control channel used only before GGRS starts.

The lobby flow is:

* host opens a lobby from the descriptor-based menu
* clients connect and send a join message
* host tracks connected players and reassigns handles
* host broadcasts a full lobby snapshot whenever membership changes
* host starts the session once ready
* every peer receives the same final session bootstrap config and starts the same GGRS topology

This solved the earlier failure mode where host and client could accidentally start different peer topologies.

### 3. Shared-Crew Encounter Runtime

The encounter runtime now spawns one synchronized crew entity per GGRS handle.

This slice includes:

* per-handle crew entity ownership
* local observed-player tracking for camera/HUD
* deterministic `PlayerHandleMap`
* removal of the previous “real local player plus remote ghost” model from the active path

The intended gameplay model is now explicit:

* one shared player ship
* multiple synchronized crew actors
* host-only outer-loop meta control

### 4. GGRS-Driven Meta Transitions

The host now emits synchronized meta commands through rollback input rather than mutating runtime state locally.

Implemented synchronized meta operations include:

* open editor
* leave editor
* open sector map
* select sector node
* launch encounter
* return to dock
* repair ship

All peers apply the host handle’s meta input during rollback stepping.

### 5. Rollback / Presentation Separation

The slice now has a clearer split between authoritative and non-authoritative layers.

Rollback-owned:

* current in-session phase
* shared ship definition
* campaign/progression
* sector state
* mission return/report data
* synchronized encounter state

Presentation-only:

* menu flow
* shell routing
* camera smoothing
* HUD text/layout
* local debug enemy editor

### 6. Diagnostics And Logging

Logging was expanded to make the new session model debuggable.

The current logs cover:

* lobby/bootstrap lifecycle
* local handle assignment
* session start
* rollback state checkpoints
* meta command publication and application
* presentation phase transitions
* encounter scene spawn/cleanup

This is intentionally aimed at debugging the real failure modes found during implementation:

* mismatched peer topology
* host-only phase mutation
* rollback/presentation phase fighting

## Current Architecture Summary

### Session Bootstrap

* descriptor-based menu flow still exists
* host opens lobby
* clients join lobby
* host starts session
* lobby distributes canonical GGRS peer list and start state

### In-Session Phase Control

* session bootstrap always seeds rollback into `Docked`
* docked, sector map, player editor, and encounter are all rollback-driven
* clients do not author synchronized outer-loop state changes

### Encounter Control Model

* multiple crew entities are simulated
* one local observed player drives camera/HUD
* the runtime is structurally ready for full per-peer control/input decoding
* not every gameplay path has been fully generalized yet

## What Was Removed Or Superseded

This slice superseded the older plan assumptions around:

* custom authoritative gameplay networking
* bespoke snapshot push/pull sync
* drift-correction-first architecture
* dual app-state/runtime-state phase control
* remote-presence ghost modeling as the core multiplayer actor representation

The implementation direction is now firmly rollback-first for synchronized gameplay.

## Known Remaining Gaps

This slice is working, but it is not yet the full end-state for the game’s multiplayer architecture.

The biggest remaining gaps are:

* synchronized player-editor content mutation is not fully encoded as deterministic per-frame editor ops yet
* some gameplay systems still need deeper generalization from “observed local player” assumptions to fully per-peer authoritative application
* several large gameplay/runtime files still need further structural cleanup
* rollback coverage exists, but not the full determinism and multiplayer regression matrix originally envisioned

So this slice proves the architecture and the primary runtime loop, but it does not yet mean “all multiplayer work is finished.”

## Why This Slice Still Matters

This slice now proves the most important things early:

* the project can run real rollback multiplayer instead of only theorizing it
* the outer loop and inner loop can share one synchronized state authority
* the game can support multi-crew play on one ship
* later content/system slices can build on a real multiplayer architecture instead of a placeholder

That makes this slice less about “adding online play” and more about establishing the runtime model the rest of the game can safely build on.

# MULTIPLAYER_DEMO — Twelfth Vertical Slice

## Goal

Build the first real multiplayer slice for **LUMEN//ARCH**, while also tightening deterministic simulation testing and sync discipline.

This slice should validate:

* the current game loop can run across host and client cleanly
* deterministic simulation is stable enough to trust as the long-term model
* sector play, ship operation, boarding, and component interaction can all remain coherent over the network
* the project has testing and debugging tools for simulation drift

This slice should prove the loop:

`Connect -> Sync Ship / Sector State -> Play Shared Encounter -> Compare Outcomes -> Detect / Correct Drift`

## Why This Slice Here

By this point, the project should have:

* a deeper UI layer
* more component variation
* a stronger single-player simulation base

That is the right moment to focus on multiplayer and determinism, because:

* there is enough game to actually stress the model
* the simulation is complex enough that drift detection matters
* testing needs to happen before even more late-stage systems are added

## Demo Pitch

Two players can connect to the same session and participate in the same ship or encounter flow. The host remains authoritative for session progress, but deterministic client simulation is exercised and verified. Inputs, outcomes, and debug tools should make it possible to catch where the simulation diverges and why.

This slice is not primarily about content expansion. It is about making the simulation architecture trustworthy under multiplayer conditions.

Core loop for the demo:

`Host Session -> Join Session -> Operate Shared World -> Observe Sync Quality -> Debug Drift`

## In Scope

* multiplayer host/client flow beyond the current bootstrap
* deterministic simulation checks and drift diagnostics
* session state synchronization for:
  * sector state
  * ship state
  * player presence
  * encounter selection
* multiplayer-safe station interaction
* multiplayer-safe boarding / EVA state
* resync and mismatch reporting tools
* test scenarios specifically for deterministic simulation stability

## Explicitly Out Of Scope

* production-ready matchmaking
* internet-scale hosting infrastructure
* large-scale persistence backend
* rollback-first redesign unless truly required
* full anti-cheat work

## Core Design Rules

### 1. Reliability comes before cleverness

The multiplayer model should remain understandable and debuggable, even if that means a conservative approach in places.

### 2. Determinism must be observable

This slice should not only assume the simulation is deterministic. It should expose hashes, comparisons, and drift reports that make determinism testable.

### 3. Existing systems must survive networking pressure

It is not enough to sync ship translation. The slice should stress:

* component interaction
* logistics
* boarding
* atmosphere
* hostile ships

## First Systems To Prove

### 1. Shared Session Flow

Deliver:

* host and client enter same campaign/session
* selected sector nodes and encounter transitions stay aligned

### 2. Multi-Actor Presence

Deliver:

* more than one player represented in simulation
* boarding / EVA / onboard movement remain coherent

### 3. Deterministic Verification

Deliver:

* state hashes
* mismatch logging
* tick-based sync diagnostics

### 4. Recovery / Resync

Deliver:

* pragmatic resync path when drift is detected
* instrumentation for debugging what diverged

## Relation To The Concept

The concept does not describe multiplayer as its core identity, but your technical direction already assumes a strong deterministic simulation architecture. This slice is where that promise becomes real engineering rather than a future intention.

It also protects all later slices by forcing the simulation model to be tested under the kind of pressure that reveals architectural weaknesses early.

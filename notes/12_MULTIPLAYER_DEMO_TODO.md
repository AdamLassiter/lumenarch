# TODO — Multiplayer Demo Implementation Breakdown

This file turns `12_MULTIPLAYER_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* preserve the deterministic fixed-step architecture as the long-term simulation base
* keep host authority and client simulation roles understandable
* prioritize drift detection, observability, and recovery over feature breadth
* avoid requiring a rollback-first redesign unless testing proves it necessary
* treat multiplayer as a simulation-quality slice, not only a networking slice

## Expected Areas Of Change

Likely touched modules:

* `src/client/mod.rs`
* `src/client/net.rs`
* `src/client/state.rs`
* `src/client/docked.rs`
* `src/client/sector_map.rs`
* `src/client/gameplay/`
* `src/ship/storage.rs`
* host/runtime networking modules
* protocol definitions

Likely new module areas:

* deterministic hash / state-audit helpers
* sync packet types
* resync handlers
* multiplayer player/session state
* automated determinism test harnesses

---

## Phase 1 — Session Model And Protocol Audit

### Goal

Define what a multiplayer session must synchronize.

### Tasks

- [x] Audit the current host/client bootstrap and protocol.
- [x] Define session state boundaries for:
  - campaign / sector state
  - ship state
  - player presence
  - encounter state
- [x] Decide what is:
  - authoritative host state
  - deterministic client-simulated state
  - replicated presentation-only state
- [x] Update protocol notes/types accordingly.

Definition of done:

* the project has a clear multiplayer state contract

---

## Phase 2 — Multi-Player Actor Representation

### Goal

Represent more than one player in the same simulation cleanly.

### Tasks

- [x] Add runtime support for multiple player entities.
- [x] Distinguish local player from remote players.
- [x] Represent boarding/EVA/onboard state per player.
- [x] Ensure player visuals and interaction targeting scale beyond one actor.

Definition of done:

* the simulation can host multiple embodied players without collapsing into one-player assumptions

---

## Phase 3 — Input Replication And Commitment

### Goal

Synchronize player intent rather than raw outcomes where possible.

### Tasks

- [x] Define per-tick replicated input payloads.
- [x] Replicate manual control inputs for:
  - movement
  - station use
  - cockpit/turret control
  - cargo interaction
- [x] Commit inputs on the simulation tick model.
- [x] Ensure locally-hosted single-player still follows the same path.

Definition of done:

* multiplayer uses a consistent input-driven simulation path

---

## Phase 4 — Shared Campaign And Encounter Flow

### Goal

Keep both peers aligned through the outer and inner loop.

### Tasks

- [x] Sync docked/sector/encounter state transitions.
- [x] Sync selected sector nodes and encounter specs.
- [x] Sync mission return flow and results.
- [x] Ensure save/load-backed session progression remains coherent for the host.

Definition of done:

* both peers move through the same campaign/encounter loop reliably

---

## Phase 5 — Multi-Actor Interaction Safety

### Goal

Make component/station interactions safe under multiplayer pressure.

### Tasks

- [x] Decide first-pass authority/locking semantics for stations.
- [x] Prevent conflicting writes from causing undefined behavior.
- [ ] Ensure interactions on hostile ships work for multiple players.
- [ ] Ensure shared-player-ship operation remains deterministic.

Definition of done:

* multiple players can interact with the same world without silently corrupting state

---

## Phase 6 — Deterministic State Hashing

### Goal

Make drift visible rather than speculative.

### Tasks

- [x] Define deterministic state-hash boundaries.
- [x] Compute periodic hashes for:
  - ship runtime state
  - player state
  - encounter state
  - campaign state where appropriate
- [x] Compare host/client hashes.
- [x] Log mismatches with enough context to debug them.

Definition of done:

* the game can tell when simulation divergence has happened

---

## Phase 7 — Drift Diagnostics And Debug UI

### Goal

Make it practical to investigate determinism failures.

### Tasks

- [x] Add debug output for last matching / first mismatching tick.
- [x] Add readable mismatch categories where possible.
- [x] Surface hash/drift status in a debug overlay or diagnostics panel.
- [x] Keep this tooling available in local-host development flows.

Definition of done:

* determinism failures are debuggable without deep ad hoc instrumentation every time

---

## Phase 8 — Resync / Recovery Path

### Goal

Recover from drift pragmatically without requiring a whole rollback architecture first.

### Tasks

- [x] Define first-pass resync payloads.
- [x] Support reapplying authoritative host state when mismatch is detected.
- [x] Limit the first pass to scoped or encounter-level resync if needed.
- [x] Ensure resync restores continued play rather than forcing full restart.

Definition of done:

* the session can recover from at least common drift cases

---

## Phase 9 — Deterministic Test Harness

### Goal

Add repeatable testing around the simulation contract.

### Tasks

- [ ] Add automated deterministic playback / replay tests where practical.
- [ ] Add test fixtures for:
  - ship movement
  - boarding
  - atmosphere
  - logistics
  - combat
- [ ] Add host/client consistency checks in integration-style tests.
- [x] Preserve repeatable seeds and inputs for debugging.

Definition of done:

* determinism regressions become testable and catchable in CI/dev workflows

---

## Phase 10 — Multiplayer UX Pass

### Goal

Make the multiplayer flow understandable for humans, not only technically functional.

### Tasks

- [x] Improve join/connect state messaging.
- [x] Show peer presence/state where useful.
- [x] Show sync or waiting states clearly.
- [x] Keep local-host single-player flow painless.

Definition of done:

* multiplayer is usable enough to test without guessing what the session is doing

---

## Phase 11 — Stress Scenarios

### Goal

Test the architecture against the systems most likely to drift.

### Tasks

- [ ] Run scenarios with:
  - simultaneous station use
  - ship movement while boarding
  - atmosphere venting
  - cargo carrying / depositing
  - hostile ship destruction
- [ ] Record observed drift hotspots.
- [ ] Fix the most common deterministic breakpoints.

Definition of done:

* the main simulation stressors are exercised, not merely assumed safe

---

## Phase 12 — Tuning And Reliability Pass

### Goal

Decide whether the current deterministic multiplayer model is sufficient.

### Tasks

- [ ] Tune input delay / synchronization cadence.
- [ ] Tune hash/resync frequency.
- [ ] Verify play remains coherent under realistic interaction load.
- [x] Decide whether rollback is still avoidable for the current design.
- [x] Capture the remaining known limits honestly.

Definition of done:

* the multiplayer architecture is either validated or has clear next constraints identified

## Immediate Next Task

Start with **Phase 1**:

* audit the existing protocol and session model
* define exact authoritative vs deterministic state ownership
* identify what needs hashing and what does not

That foundation determines whether the later multiplayer work stays coherent.

# TODO — Multiplayer Demo Implementation Breakdown

This file now records what went into the multiplayer slice that was actually implemented, plus the remaining work that still belongs conceptually to this slice.

## Final Constraints Chosen For This Slice

The implementation locked these decisions in:

* `bevy_ggrs` rollback networking is the active synchronized gameplay path
* shared-crew, one-ship multiplayer is the gameplay model
* the host controls synchronized docked / sector-map / player-editor state changes
* menu/lobby/bootstrap remain outside rollback
* late join and host migration are still out of scope
* the debug enemy editor remains local-only and outside synchronized rollback flow

## Major Implementation Areas Completed

### Phase 1 — Remove The Old Gameplay Netcode Direction

Goal:

Replace the old bespoke gameplay sync direction with rollback-owned session flow.

Completed:

- [x] Disconnect the old custom multiplayer modules from the active app runtime.
- [x] Remove the old host/snapshot/protocol stack from the active gameplay path.
- [x] Replace the temporary sync-test-first bootstrap with a real P2P session direction.
- [x] Update project docs and notes to reflect rollback as the intended implementation model.

Definition of done:

* the active runtime no longer depends on the old custom gameplay networking stack

---

### Phase 2 — Add Pre-Session Lobby / Bootstrap Control Channel

Goal:

Make the host the source of truth for connected players and the final GGRS peer topology.

Completed:

- [x] Add a lightweight lobby/bootstrap channel separate from rollback gameplay sync.
- [x] Let clients send a join event before session start.
- [x] Track connected clients on the host.
- [x] Broadcast a full lobby snapshot whenever membership changes.
- [x] Broadcast a canonical `StartSession` config so every peer starts the same GGRS topology.
- [x] Fix the earlier “host started a one-player session while a client started a two-player session” failure mode.

Definition of done:

* host and clients no longer rely on manually matching full peer topology by hand

---

### Phase 3 — Collapse Runtime Authority To Rollback Phase

Goal:

Remove the duplicated in-session state machine and make rollback the only runtime authority.

Completed:

- [x] Remove the old in-session `ClientAppState` phase ownership model.
- [x] Replace it with `FrontendMode` for shell routing only.
- [x] Keep `RollbackPhase` as the only in-session phase authority.
- [x] Drive presentation phase transitions from rollback state instead of state-entry/state-exit hooks.
- [x] Fix host UI phase flicker caused by rollback/presentation state fighting.
- [x] Fix clients staying in menu because phase transitions were previously host-local.

Definition of done:

* docked, sector-map, editor, and encounter presentation now follow rollback phase instead of competing with it

---

### Phase 4 — Add GGRS Meta Commands For Shared Session Flow

Goal:

Synchronize outer-loop transitions through rollback input rather than local UI mutations.

Completed:

- [x] Add `RollbackMetaOp`.
- [x] Route host-authored docked/sector/player-editor transitions through `PlayerGgrsInput`.
- [x] Apply host meta ops on every peer during rollback stepping.
- [x] Synchronize:
  - open sector map
  - select node
  - launch encounter
  - open editor
  - leave editor
  - return to dock
  - repair ship
- [x] Fix the earlier bug where clients did not follow host transitions.

Definition of done:

* synchronized outer-loop transitions now happen through rollback input instead of local state writes

---

### Phase 5 — Establish Multi-Crew Encounter Runtime

Goal:

Replace the single-local-player runtime assumption with synchronized crew entities.

Completed:

- [x] Spawn one crew entity per GGRS handle.
- [x] Add stable per-player handle ownership (`PlayerHandleComponent` / `PlayerHandleMap`).
- [x] Add observed-local-player presentation tracking for camera/HUD.
- [x] Remove the active remote-ghost model from the main path.
- [x] Make several interaction / field / atmosphere paths operate over multiple crew entities in stable order.

Definition of done:

* encounter runtime can represent shared-crew play structurally instead of pretending multiplayer is only local-plus-ghosts

---

### Phase 6 — Push Deterministic Execution Onto Rollback Scheduling

Goal:

Move synchronized logic away from ad hoc local-input/runtime-state flow and into the rollback path.

Completed:

- [x] Expand `PlayerGgrsInput` into the current canonical synchronized input packet.
- [x] Add decoded per-peer rollback command state.
- [x] Move authoritative simulation work onto `GgrsSchedule`.
- [x] Route synchronized control/meta input away from direct local state changes.
- [x] Register a broader set of rollback resources/components than the original scaffolding.

Definition of done:

* the active multiplayer runtime is now genuinely rollback-driven, not just rollback-bootstrapped

---

### Phase 7 — Improve Logging And Debuggability

Goal:

Make the new multiplayer/runtime model inspectable when things go wrong.

Completed:

- [x] Add `info` logs for one-time bootstrap/session/phase transitions.
- [x] Add `debug`/`trace` logs for input publication, decoded commands, rollback checkpoints, and lobby updates.
- [x] Reserve `warn` for rollback regressions, lobby failures, and other non-ideal events.
- [x] Add related logs in menu, docked, sector-map, and scene-spawn paths to correlate user action with rollback transition.

Definition of done:

* multiplayer issues can now be debugged from logs without guessing where phase/input ownership broke

---

## Remaining Work Still Belonging To This Slice

These items are still conceptually part of the multiplayer slice, even though the main architecture is now in place.

### Remaining Gameplay / Synchronization Work

- [ ] Finish synchronized player-editor mutations as deterministic rollback input operations instead of direct host-side rollback resource edits.
- [ ] Continue removing remaining observed-local-player assumptions from authoritative gameplay logic.
- [ ] Continue pushing synchronized gameplay-critical systems toward full per-peer deterministic command application.
- [ ] Expand multiplayer/determinism regression coverage beyond the current small test set.

### Remaining Codebase Cleanup

- [ ] Continue breaking up oversized gameplay/runtime files that still concentrate too many systems or responsibilities.
- [ ] Keep aligning module layout with ECS/gameplay concepts so multiplayer-specific logic remains debuggable.

## Slice Outcome

This slice should now be considered:

* **architecturally successful**
* **runtime-functional for host/client sector and encounter flow**
* **not yet feature-complete for all remaining multiplayer polish and determinism tooling**

That is an important distinction. The slice is no longer “planned multiplayer work”; it is now “working rollback multiplayer with known follow-up gaps.”

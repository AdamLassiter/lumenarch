# 24_MISSION_ARTIFACTS_DEMO

## Goal

Turn contract work into concrete mission objectives by making some jobs require physical recovered artifacts, not just launch and debrief text.

Players should be able to accept a retrieval contract, launch to the target encounter, find a specific artifact or log, carry it back to the ship, return to station, and see the recovered evidence unlock Archives material and docked follow-up.

## What This Slice Covers

- Mission artifacts and recovered logs as physical carried items.
- Artifact pickup, carry, deposit, and mission return through the existing embodied cargo loop.
- Retrieval contract completion gated by returning the required artifact.
- Archive unlocks from returned artifacts outside the current contract-only lore unlock path.
- A docked debrief or inbox acknowledgement after returning from a mission.
- One authored Blueglass Hush / Peregrine Cho follow-up beat.
- One small Null Swarm-flavored encounter or identity beat to broaden opposition texture.

## Data Model

This slice adds a small artifact model rather than a general quest-item framework.

V1 artifact kinds:

- `BlueglassArchiveShard`
- `NullSwarmTelemetry`
- `ContinuantLedger`

Artifacts need:

- stable serialized ids
- short player-facing labels
- display colors for carried-item feedback
- optional lore unlock ids
- optional contract objective use

The existing carried-item and storage models should be extended so artifacts behave like other physical mission cargo:

- the player can carry one artifact at a time
- artifacts can be deposited into player ship storage
- mission resolution reads returned artifacts from ship storage
- returned artifacts are summarized in the last mission report

Contracts gain an optional required artifact field. For this slice, `Blueglass Archive Pull` requires returning `BlueglassArchiveShard`.

## Gameplay Loop

The intended first pass loop is:

1. Talk to the Contract Broker or open the contract board.
2. Accept `Blueglass Archive Pull`.
3. Launch to Blueglass Hush.
4. Find the `BlueglassArchiveShard` in the encounter space.
5. Pick it up as carried cargo.
6. Deposit it aboard the player ship.
7. Return to Needle Rest.
8. See the mission debrief acknowledge the returned artifact.
9. Open Archives and read the newly unlocked entry.

If the player returns without the required artifact, ordinary salvage can still pay out, but the retrieval contract should remain incomplete and should not grant contract bonus scrap or its artifact-gated lore unlock.

## Docked Debrief

Docked return should have an explicit acknowledgement surface instead of relying only on the editor mission report panel.

The first version can reuse existing docked UI style and mission report data. It should show:

- mission headline
- whether the contract objective was completed
- recovered artifacts
- newly unlocked archive entries
- next suggested action, such as visiting Archives or relaunching

Browsing or dismissing this debrief is local presentation state. Contract completion, artifact recovery, lore unlocks, and rewards remain rollback-authored state.

## Multiplayer Model

Artifact pickup, deposit, storage, mission return, contract completion, rewards, and lore unlocks must be deterministic and rollback-safe.

Debrief browsing, Archive browsing, and NPC dialogue presentation remain local per player unless a selected action emits an existing rollback meta command.

## Relationship To Existing Slices

This slice extends `16_STATIONS_DEMO` by making contracts and Archives part of a physical mission loop rather than only station board content.

It extends `23_DOCKED_SPACEPORT_DEMO` by giving docked NPCs and Archives a stronger reason to be revisited after missions.

It builds on the player embodiment and salvage work by using the existing one-carried-item rule instead of adding abstract objective collection.

## Assumptions

- Needle Rest and Blueglass Hush are the only required authored locations for this slice.
- This is not a full quest-chain or reputation system.
- This is not a broad multi-station progression pass.
- Physical recovery is preferred over abstract scan or objective flags.
- Returning without a required artifact should be a partial success, not a total mission failure.

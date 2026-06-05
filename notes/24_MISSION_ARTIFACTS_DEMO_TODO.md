# 24_MISSION_ARTIFACTS_DEMO TODO

## Data And Types

- [x] Add a small `MissionArtifactKind` enum with stable serialization.
- [x] Add artifact labels and display colors.
- [x] Add artifact-to-lore unlock mapping.
- [x] Extend carried cargo with artifact items.
- [x] Extend player ship storage with returned artifacts.
- [x] Add serde defaults for new save/report/contract fields.
- [x] Keep artifacts separate from raw salvage, damaged components, and generic resources.

## Contract Objective Logic

- [x] Add optional `required_artifact` to station contracts.
- [x] Update `Blueglass Archive Pull` to require `BlueglassArchiveShard`.
- [x] Complete retrieval contracts only when the required artifact returns in player ship storage.
- [x] Award ordinary salvage independently from contract objective completion.
- [x] Withhold contract bonus scrap when the required artifact is missing.
- [x] Withhold artifact-gated lore unlocks when the required artifact is missing.
- [x] Leave incomplete retrieval contracts active or re-acceptable after a failed recovery.
- [x] Report missing objective status clearly in the mission debrief.

## Runtime Encounter

- [x] Spawn required contract artifacts when launching the relevant encounter.
- [x] Place artifacts near the relevant wreck, hostile objective, or authored encounter point.
- [x] Avoid spawning required artifacts beside the player ship.
- [x] Support artifact pickup through the existing carried-item input flow.
- [x] Support artifact deposit through the existing ship-storage input flow.
- [x] Prevent duplicate artifact recovery from the same spawned artifact entity.

## Mission Resolution

- [x] Collect returned artifacts from player ship storage during mission return.
- [x] Add returned artifact summaries to the last mission report.
- [x] Unlock artifact-linked lore through progression.
- [x] Include recovered artifacts in docked debrief text.
- [x] Preserve current damaged-component and resource return behavior.
- [x] Ensure returned artifacts do not silently become scrap.

## Docked Debrief And Archives

- [x] Add a docked debrief or inbox acknowledgement surface.
- [x] Show mission headline and contract objective status.
- [x] Show recovered artifacts.
- [x] Show newly unlocked Archives entries.
- [x] Add a short Peregrine Cho reaction for the Blueglass archive shard.
- [x] Ensure Archives immediately shows the artifact-unlocked entry.
- [x] Keep debrief browsing local unless a button emits a rollback meta command.

## Content

- [x] Add `BlueglassArchiveShard` artifact content.
- [x] Add one Archive entry unlocked by the Blueglass artifact.
- [x] Add one Null Swarm-flavored telemetry or encounter framing beat.
- [x] Add concise artifact pickup/debrief text.
- [x] Keep new written content short and practical.

## Tests

- [x] Artifact labels, colors, and lore unlock mapping are stable.
- [ ] Artifact pickup produces carried artifact cargo.
- [x] Depositing an artifact stores it on the player ship and clears carried cargo.
- [ ] Mission return transfers stored artifacts into mission report/progression.
- [ ] Retrieval contract completes when the required artifact returns.
- [ ] Retrieval contract remains incomplete when the required artifact is missing.
- [ ] Missing artifact does not grant contract bonus or artifact-gated lore.
- [ ] Artifact archive unlock persists through save/load.

## Verification

- [x] `cargo fmt`
- [x] `cargo test -q`
- [ ] Interactive smoke test: accept Blueglass contract, launch, recover artifact, deposit it, return, view debrief, and read the unlocked Archive entry.

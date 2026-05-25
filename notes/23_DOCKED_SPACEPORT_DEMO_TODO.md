# 23_DOCKED_SPACEPORT_DEMO TODO

## Data And Authoring

- [x] Add station layout fields to station catalog.
- [x] Add NPC definitions with ids, position, facing, dialogue node, and optional service action.
- [ ] Add `EditorMode::Station`.
- [ ] Reuse grid editor patterns for station tiles and NPC placement.
- [ ] Persist station layout edits to `saves/stations.json`.

## Ship Docking Requirement

- [x] Require at least one Airlock on player ships.
- [x] Add editor validation/status messaging for missing Airlock.
- [x] Block launch/refit acceptance when no valid docking Airlock exists.

## Runtime Docked Scene

- [x] Spawn station layout during `RollbackPhase::Docked`.
- [x] Spawn player ship docked to station anchor.
- [x] Spawn players near the ship Airlock.
- [x] Add cleanup when leaving Docked.
- [x] Replace old docked menu with local status/help overlays.

## NPC Interaction And Dialogue

- [x] Add `bevy_yarnspinner = "0.8"`.
- [x] Add `assets/dialog/` Yarn files for first station NPCs.
- [x] Detect focused-tile NPC interactions.
- [x] Open local dialogue overlay on `E`.
- [x] Route dialogue service choices to existing rollback meta commands.

## Services And Navigation

- [x] Engineer NPC offers Repair and Refit.
- [x] Contract NPC exposes contract browsing/acceptance.
- [x] Archive NPC exposes lore/dialogue.
- [x] Cockpit/Helm interaction opens Sector Map while docked.
- [x] Disable or bypass ship flight controls while docked.

## Tests

- [x] Station catalog serialization covers layout, docking anchor, and NPCs.
- [x] Ship validation rejects missing Airlock.
- [ ] Focused NPC tile opens NPC interaction.
- [ ] Unfocused nearby NPC does not interact.
- [x] Docked scene spawns deterministically.
- [ ] Engineer repair/refit queues existing meta commands.
- [ ] Cockpit/Helm opens sector map from Docked.
- [ ] Local dialogue state does not mutate rollback state unless a service command is selected.

## Verification

- [x] `cargo fmt`
- [x] `cargo test`
- [ ] Interactive smoke test: docked spawn, walk to NPC, dialogue opens, repair/refit works, cockpit opens sector map.
- [x] Rotate the docked player ship 90 degrees counterclockwise against station layouts.
- [x] Make docked NPCs behave like component-style focused interactables.

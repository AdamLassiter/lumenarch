# TODO — Hostile Ships Demo Implementation Breakdown

This file turns `07_HOSTILE_SHIPS_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* reuse the existing ship-definition language for enemies
* keep the first enemy AI readable and deterministic
* extend the current encounter runtime rather than replacing it
* make enemy content iteration fast through a debug editor
* persist enemy ship content to `saves/enemy_ships.json`

## Expected Areas Of Change

Likely touched modules:

* `src/client/menu.rs`
* `src/client/state.rs`
* `src/client/editor/`
* `src/client/gameplay/`
* `src/ship/`
* new save/content helpers for enemy ship libraries

---

### Phase 1 — Enemy Ship Content Model

#### Goal

Define a persistent enemy ship library.

#### Tasks

- [x] Add `EnemyShipLibrary`.
- [x] Add `EnemyShipEntry` with:
  - display name
  - ship definition
  - threat tier
  - behavior tag
- [x] Add load/save support for `saves/enemy_ships.json`.
- [x] Seed the file with at least two sample hostile ships.

Definition of done:

* hostile ship content exists outside hardcoded encounter logic

---

### Phase 2 — Debug Menu Entry

#### Goal

Make enemy ship authoring accessible from the client.

#### Tasks

- [x] Add a debug menu item to enter enemy ship editing.
- [x] Add client state/resource needed to distinguish player-ship vs enemy-ship editing context.
- [x] Keep the flow clearly marked as debug-only.

Definition of done:

* a user can intentionally enter enemy ship editing from the menu or debug UI

---

### Phase 3 — Enemy Ship Editor Flow

#### Goal

Reuse or adapt the current editor for enemy authoring.

#### Tasks

- [x] Add enemy ship selection within the debug editor flow.
- [x] Allow creating a new enemy entry.
- [x] Allow editing module layout and saved ARCH template data for an enemy ship.
- [x] Save the edited library back to `saves/enemy_ships.json`.

Definition of done:

* enemy ships can be authored and iterated in-engine

---

### Phase 4 — Hostile Ship Runtime Structure

#### Goal

Spawn enemy ships as real runtime ship entities.

#### Tasks

- [x] Add hostile ship root/runtime markers and state.
- [x] Spawn hostile ship modules from enemy ship definitions.
- [x] Give hostile ships module integrity, movement, and weapon runtime components.
- [x] Preserve compatibility with existing module damage systems where possible.

Definition of done:

* a hostile ship exists in the arena as a modular vessel rather than a turret point

---

### Phase 5 — First Enemy AI

#### Goal

Make hostile ships move and fight in a simple readable way.

#### Tasks

- [x] Add first-pass hostile ship control logic.
- [x] Support at least:
  - turn toward player
  - move or hold range
  - fire when reasonably aligned
- [x] Keep AI deterministic and intentionally simple.

Definition of done:

* hostile ships behave like ships, even if crudely

---

### Phase 6 — Hostile Damage And Disablement

#### Goal

Make enemy ships degrade meaningfully under fire.

#### Tasks

- [x] Route projectile hits into hostile ship modules.
- [x] Disable hostile movement when engines are lost.
- [x] Reduce hostile fire capability when turrets are lost.
- [x] Reduce hostile survivability/behavior when reactors/core are lost.

Definition of done:

* the player can de-weapon or de-mobilize an enemy ship before total destruction

---

### Phase 7 — Wreck And Salvage Outcome

#### Goal

Turn defeated hostile ships into more meaningful encounter aftermath.

#### Tasks

- [x] Add defeated hostile ship resolution.
- [x] Spawn a wreck/salvage outcome based on enemy ship defeat.
- [x] Tie payout or recoverable salvage to the defeated ship.

Definition of done:

* defeating a hostile ship produces a better narrative and reward result than deleting a point target

---

### Phase 8 — EncounterSpec Hostile Loadouts

#### Goal

Drive encounters from enemy ship compositions.

#### Tasks

- [x] Extend `EncounterSpec` to include hostile ship entries or loadouts.
- [x] Update route nodes to reference hostile ship mixes.
- [x] Keep existing salvage/hazard knobs working alongside the new hostile loadouts.

Definition of done:

* travel nodes differ by enemy vessel composition rather than only turret count

---

### Phase 9 — Combat Readability Pass

#### Goal

Make hostile ship combat readable at a glance.

#### Tasks

- [x] Add clear hostile ship tinting / state feedback.
- [x] Make hostile turrets and hull silhouettes legible.
- [x] Surface basic hostile status in the encounter HUD if useful.

Definition of done:

* the player can tell whether a hostile ship is still dangerous, crippled, or finished

---

### Phase 10 — Debug Authoring Polish

#### Goal

Make enemy ship authoring fast enough for iteration.

#### Tasks

- [x] Add simple enemy entry switching.
- [ ] Add visible metadata editing for name / threat tier / behavior tag if feasible.
- [x] Confirm save/load round-trips cleanly for enemy ship content.

Definition of done:

* enemy content iteration is practical without code edits

---

### Phase 11 — Travel Integration And Tuning

#### Goal

Tune the slice so it works as part of the campaign loop.

#### Tasks

- [x] Tune which nodes spawn which hostile ships.
- [x] Tune reward vs threat for the first enemy ship archetypes.
- [x] Verify station return, refit, and relaunch still feel coherent.

Definition of done:

* hostile ships feel like a natural extension of the travel loop

---

### Phase 12 — Playtest And Tuning

#### Goal

Playtest the slice until enemy ships feel worth having.

#### Tasks

- [ ] Tune AI aggression and movement.
- [ ] Tune enemy ship layouts for readability and fairness.
- [ ] Check whether ship-vs-ship combat meaningfully improves the prototype.
- [ ] Confirm the debug editor supports fast hostile iteration.

Definition of done:

* the prototype feels materially closer to the intended game because of hostile ships

## Immediate Next Task

Implementation status:

* phases 1 through 9 and 11 are now in place in the prototype
* phase 10 is mostly in place, with metadata editing still intentionally lightweight
* phase 12 remains open for real playtest and tuning

# TODO — Player Demo Implementation Breakdown

This file records what has already gone into the `PLAYER_DEMO` slice, plus the remaining work that still belongs conceptually to this player-agency and survival pass.

## Final Constraints Chosen For This Slice

The implementation direction for this slice currently assumes:

* the player is a real embodied actor distinct from ship systems
* the player has one equipped suit slot and one carried-item slot
* player inputs are the only intended nondeterministic source during missions
* manual repair, extraction, hauling, and EVA are all part of the core mission loop
* the first pass should stay legible and simple rather than turning into a large inventory game

## Major Implementation Areas Completed

### Phase 1 — Establish Player Equipment And Carry State

Goal:

Give the player a concrete embodied equipment model instead of treating them as a generic mover.

Completed:

- [x] Add explicit suit state to the player runtime.
- [x] Add a broader carried-item model instead of only raw resource cargo.
- [x] Support:
  - [x] standard suit
  - [x] radiation suit
  - [x] welder suit
  - [x] EVA suit
- [x] Support carried item kinds for:
  - [x] resources
  - [x] suits
  - [x] extracted hostile components

Definition of done:

* the player has a visible, meaningful equipment/carry state rather than only abstract interaction permissions

---

### Phase 2 — Make Suit Choice Mechanically Matter

Goal:

Turn suits into real operational differences instead of cosmetic loadout labels.

Completed:

- [x] Apply suit-specific heat tolerance modifiers.
- [x] Apply suit-specific electrical tolerance modifiers.
- [x] Apply suit-specific oxygen warning/critical thresholds.
- [x] Apply suit-specific EVA movement speed multipliers.

Definition of done:

* the chosen suit changes what the player can safely survive and how fast they can operate in space

---

### Phase 3 — Broaden Carry / Pickup / Deposit Flow

Goal:

Make recovery and hauling more embodied and less abstract.

Completed:

- [x] Allow the player to pick up non-resource carried items.
- [x] Allow the player to carry loose suits.
- [x] Allow the player to equip a carried suit in the field.
- [x] Keep the carried-item model intentionally single-slot.
- [x] Allow carried salvage to be deposited back aboard the player ship.
- [x] Convert extracted hostile components into salvage value on deposit.

Definition of done:

* the player can physically move value through the mission using one carried item at a time

---

### Phase 4 — Add Manual Repair And Extraction Gating

Goal:

Make recovery work something the player personally performs under equipment constraints.

Completed:

- [x] Gate ship repair behind the welder suit.
- [x] Add a dedicated `Extract` interaction kind.
- [x] Detect extractable hostile modules in encounters.
- [x] Make extraction a held interaction rather than an instant action.
- [x] Award an extracted hostile component item on successful extraction.
- [x] Mark extracted modules as removed from further normal use.

Definition of done:

* repair and hostile-ship stripping are now embodied player work with explicit tool/suit requirements

---

### Phase 5 — Seed The Slice With Real Mission Content

Goal:

Ensure the slice is actually playable without external setup.

Completed:

- [x] Spawn loose suit pickups near the player ship in encounters.
- [x] Keep salvage and hostile wreck cargo compatible with the broader carried-item model.
- [x] Let hostile component extraction feed back into the salvage return loop.

Definition of done:

* a normal encounter already contains enough gear and recovery affordances to exercise the slice

---

### Phase 6 — Surface Player Condition In The HUD

Goal:

Keep the slice readable without forcing players to infer too much from invisible state.

Completed:

- [x] Show equipped suit in gameplay status/HUD paths.
- [x] Show carried-item state in gameplay status/HUD paths.
- [x] Update controls/help text to reflect pickup, equip, deposit, and embodied recovery flow.

Definition of done:

* the player can tell what they are wearing, what they are carrying, and how to act on it

---

## Remaining Work Still Belonging To This Slice

These items are still conceptually part of `PLAYER_DEMO`, even though the first playable pass now exists.

### Remaining Survival / Condition Work

- [x] Add stronger direct player-condition consequences beyond warnings and movement pressure.
- [x] Decide whether player heat/electrical/oxygen exposure should produce explicit injury/incapacitation states.
- [x] Add clearer readouts or overlays for hazardous compartments and exterior recovery spaces.
- [ ] Tune suit balance so:
  - [ ] radiation suit feels like the obvious hazard-work choice
  - [ ] welder suit feels like the repair/extraction choice
  - [ ] EVA suit feels meaningfully faster without becoming mandatory

### Remaining Recovery / Salvage Work

- [x] Preserve extracted hostile components as richer returnable artifacts instead of converting all of them straight into raw salvage.
- [ ] Decide which hostile module kinds should have special recovery value or post-mission outcomes.
- [ ] Add more mission situations where carrying a specific thing back actually matters.
- [ ] Add stronger post-mission reporting around:
  - [ ] suits used
  - [ ] repairs performed manually
  - [ ] components extracted
  - [ ] carried salvage returned

### Remaining Equipment / Loadout Work

- [x] Move suit availability and selection into the docked/refit loop instead of encounter-only pickups.
- [ ] Decide whether the player starts each mission with:
  - [x] a chosen equipped suit
  - [ ] one carried backup suit
  - [ ] neither, with all suits sourced in-mission
- [ ] Add clearer rules for suit swapping under pressure.
- [x] Add save/persistence for any intended player loadout state across launches.

### Remaining Mission / Encounter Content Work

- [ ] Add encounter content that really pressures manual survival choices rather than only making them available.
- [ ] Add scenarios where:
  - [ ] the radiation suit is required for worthwhile recovery
  - [ ] the welder suit is required to stabilize the mission
  - [ ] the EVA suit materially changes whether salvage can be reached in time
- [ ] Add hostile/wreck layouts that reward boarding, extraction, and physical hauling more deliberately.

### Remaining Code / Architecture Follow-Up

- [ ] Continue removing local-authority gameplay gaps so embodied player actions stay aligned with the rollback/determinism model.
- [ ] Continue migrating any remaining relevant player interactions to authoritative synchronized input paths where needed.
- [ ] Add stronger regression coverage for:
  - [ ] suit effects
  - [ ] extraction flow
  - [ ] carried-item equip/drop/deposit behavior
  - [ ] manual repair gating

## Additional Completed Work Since First Pass

- [x] Add a damaged-component inventory to progression instead of treating extracted parts as immediate scrap.
- [x] Require scrap-based repair of damaged recovered components during player refit.
- [x] Make the player ship refit consume and refund concrete component inventory in player mode.
- [x] Keep the enemy ship editor on effectively unlimited component supply.
- [x] Add pre-game lobby profile selection for player name, role/default suit, and avatar color.
- [x] Show the chosen player identity in the lobby, docked station readout, and over the in-mission actor.
- [x] Add actor sprite expectations under `assets/actors/README.md`.

## Slice Outcome

This slice should currently be considered:

* **playably established**
* **mechanically meaningful in first-pass form**
* **not yet fully realized as the broader player-survival loop described in the design note**

That distinction matters. `PLAYER_DEMO` is no longer just a plan: the first meaningful embodiment/suit/carry/extraction loop exists now, but the deeper survival, loadout, persistence, and mission-pressure work is still ahead.

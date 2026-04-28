# TODO — Playable Demo Implementation Breakdown

## Purpose

This file turns `PLAYABLE_DEMO.md` into a concrete implementation checklist for the current codebase.

The focus is:

* move from ship editor utility to playable local demo
* keep scope tight
* sequence work so each step unlocks visible progress

## Current Baseline

Already in place:

* local host/client handshake
* shared editor/runtime ship pipeline
* ship editor UI
* tile placement/removal/rotation
* sprite-based module rendering
* sample ship snapshot and asset contract
* editor-to-runtime launch path

Not yet in place:

* movement
* combat
* salvage

## Phase 1 — Shared Ship Model

### Goal

Make the editor ship the canonical source for runtime spawning.

### Tasks

- [x] Audit `src/protocol.rs` and decide whether `ShipSnapshot` becomes the short-term canonical ship definition or whether we introduce a separate runtime/editor `ShipDefinition`.
- [x] Add a dedicated ship data module, likely something like `src/ship.rs` or `src/ship/mod.rs`, for shared ship-domain types.
- [x] Move tile/module definitions out of UI-specific code and into shared data structures.
- [x] Define a stable module kind enum or strongly-typed ids instead of loose strings where practical.
- [x] Add helpers for:
  - [x] iterating modules by grid position
  - [x] replacing/removing modules
  - [x] finding ship bounds
  - [x] validating required modules like `core` or `cockpit`
- [x] Decide how editor state is stored between scenes:
  - [x] keep it in a Bevy resource for now
  - [ ] optionally serialize to disk later

### Files Likely Affected

* `src/protocol.rs`
* new shared ship module
* `src/client/editor.rs`

### Phase 1 Notes

Completed approach:

* `src/ship/mod.rs` now owns the canonical ship-domain model
* `ShipSnapshot` in `src/protocol.rs` is now a thin alias to the shared `ShipDefinition`
* `ModuleKind` is now a typed enum instead of loose string ids
* editor state now stores a shared `ShipDefinition` in a Bevy resource for scene-to-scene continuity

## Phase 2 — Editor To Runtime Launch Path

### Goal

Launch a playable mission using the ship currently built in the editor.

### Tasks

- [x] Extend client app state beyond `Menu` and `Viewing`:
  - [x] rename `Viewing` to `Editing` if helpful
  - [x] add a `Playing` state
- [x] Add a `Launch` button to the editor UI.
- [x] Add a system that transitions from editor mode into gameplay mode.
- [x] Create a runtime scene bootstrap that reads the current edited ship resource.
- [x] Ensure leaving the mission can return to the editor without losing ship data.
- [x] Add cleanup systems so editor-only and gameplay-only entities do not leak across states.

### Files Likely Affected

* `src/client/mod.rs`
* `src/client/editor.rs`
* new gameplay/runtime module

### Phase 2 Notes

Completed approach:

* client state now uses `Menu -> Editing -> Playing`
* the editor has a `Launch` button and `L` shortcut
* a new gameplay bootstrap scene spawns directly from the current shared `EditorShip` resource
* runtime scene entities can return to the editor with a button or `Tab`
* editor-only and runtime-only entities now use separate cleanup paths

## Phase 3 — Runtime Ship Spawn

### Goal

Spawn the edited ship into a playable world scene.

### Tasks

- [x] Define runtime ship entity structure:
  - [x] vessel root entity
  - [x] module child entities or equivalent runtime entities
- [x] Add a simple combat/test arena scene.
- [x] Spawn the ship at a known starting position.
- [x] Spawn module visuals in runtime from the same module data used in the editor.
- [x] Add a camera follow target for the ship.
- [x] Confirm runtime rotations match editor rotations exactly.

### Suggested First Runtime Components

- [x] `PlayerShip`
- [x] `ShipRoot`
- [x] `RuntimeShipModule`
- [x] `ModuleKind`
- [x] `Integrity`
- [x] `PowerProducer`
- [x] `PowerConsumer`
- [x] `EngineModule`
- [x] `WeaponModule`

### Files Likely Affected

* new gameplay/runtime module
* new ship spawning module

### Phase 3 Notes

Completed approach:

* runtime ship spawning now creates a `ShipRoot` / `PlayerShip` root entity plus one child entity per installed module
* runtime module transforms are centered relative to ship bounds, while the ship root spawns at a fixed arena origin
* the play scene now includes a simple boxed arena backdrop so runtime mode reads as a world scene instead of a UI-only preview
* the main camera follows the player ship root during the `Playing` state
* editor and runtime both use the same quarter-turn rotation mapping, so visual orientation matches across the transition

## Phase 4 — Movement And Flight Feel

### Goal

Make the ship controllable and readable in motion.

### Tasks

- [x] Add movement input mapping:
  - [x] thrust
  - [x] rotate left
  - [x] rotate right
- [x] Add ship velocity / angular velocity state.
- [x] Add engine contribution logic based on installed modules.
- [x] Start with simple aggregate movement:
  - [x] total thrust from active engines
  - [x] simple torque or turn rate
- [x] Add camera follow smoothing if needed.
- [x] Tune controls until the ship feels weighty but responsive enough for testing.

### First-Pass Simplifications

- [ ] Ignore precise per-engine torque if that slows implementation.
- [ ] Start with simple forward thrust and fixed angular speed if needed.
- [ ] Add more detailed mass/thrust modeling later.

### Files Likely Affected

* new movement module
* gameplay state wiring

### Phase 4 Notes

Completed approach:

* the `ShipRoot` now carries `LinearVelocity`, `AngularVelocity`, and a `ShipMovementModel`
* `W` or `Up` applies forward thrust, while `A` / `Left` and `D` / `Right` apply aggregate turning
* movement stats are derived from the installed engine count and total module count, giving heavier ships less acceleration
* runtime motion currently uses a simple deterministic aggregate model rather than per-engine torque simulation
* camera follow now eases toward the ship root instead of snapping directly, which makes motion easier to read during testing

## Phase 5 — Minimal Power Model

### Goal

Give modules meaningful operational constraints without overbuilding simulation.

### Tasks

- [x] Add simple total power generation.
- [x] Add simple total power draw.
- [x] Add battery reserve support if present.
- [x] Define what happens on power deficit:
  - [x] disable weapons first
  - [x] disable engines
  - [x] or just flag “insufficient power”
- [x] Make module presence matter:
  - [x] no reactor = dead ship
  - [x] more batteries = more reserve
  - [x] more engines = more draw

### First-Pass Simplifications

- [x] Use aggregate vessel-wide power instead of full network routing.
- [x] Skip per-tick power topology logic for the first demo.

### Phase 5 Notes

Completed approach:

* the `ShipRoot` now owns a vessel-wide `ShipPowerModel` and `ShipPowerState`
* reactor output, passive draw, battery reserve, engine demand, and weapon demand are aggregated from the installed module mix
* battery reserve drains when draw exceeds reactor output and recharges when the ship has surplus generation
* on deficit, the runtime sheds weapon power first and then reduces or disables engine power by scaling `engine_power_ratio`
* movement now responds directly to power availability, so ships without enough generation can still drift but lose thrust authority as reserve runs out

## Phase 6 — Weapons And Projectiles

### Goal

Let the player shoot and destroy something.

### Tasks

- [x] Add fire input.
- [x] Choose first weapon behavior:
  - [x] instant projectile
  - [x] simple forward bullet
- [x] Spawn projectiles from turret modules.
- [x] Add projectile lifetime and movement.
- [x] Add projectile collision with hostile targets.
- [x] Add weapon cooldown.
- [x] Gate weapon firing on power availability.

### First-Pass Simplifications

- [x] Use one generic turret behavior for all `turret` tiles.
- [x] Fire in ship-forward direction first if turret aiming is too much.

### Phase 6 Notes

Completed approach:

* the player ship now tracks `ShipWeaponState` with turret count and shared cooldown
* `Space` fires one simple forward projectile from each installed turret when weapons have power and cooldown is clear
* projectiles use a short lifetime, fixed speed, and arena-bounded cleanup
* the runtime arena now includes a few stationary hostile targets so projectile collision has real feedback before enemy AI exists
* firing is explicitly gated by the Phase 5 power system, so weapon outages show up immediately in the runtime HUD and behavior

## Phase 7 — Damage And Failure

### Goal

Make hits matter and ships fail in understandable ways.

### Tasks

- [x] Add health/integrity to runtime modules.
- [x] Route projectile hits to module damage.
- [x] Decide first damage resolution rule:
  - [ ] hit nearest tile
  - [x] hit overlap tile
  - [ ] hit ship aggregate then map to module
- [x] Destroy or disable modules at zero integrity.
- [x] Define basic failure outcomes:
  - [x] destroyed turret stops firing
  - [x] destroyed engine stops thrust
  - [x] destroyed reactor stops power
  - [x] destroyed core means mission failure
- [x] Add simple visual feedback for damage if easy.

### Phase 7 Notes

Completed approach:

* hostile projectiles now resolve against individual runtime module entities using simple overlap checks against module centers
* destroyed modules are marked with `DestroyedModule`, tinted dark, and excluded from live ship capability counts
* ship movement, turret count, and power production are recalculated from surviving modules each frame, so destroyed engines, turrets, reactors, and batteries immediately change behavior
* core destruction sets mission failure on the player ship and disables further control/fire input
* static arena hostiles now emit slow return fire so the damage path is testable before the fuller enemy-encounter phase

## Phase 8 — Enemy Encounter

### Goal

Create one threat that makes the mission a game instead of a sandbox.

### Tasks

- [x] Add one enemy type:
  - [x] turret platform
  - [ ] or tiny hostile drone ship
- [x] Add simple AI:
  - [x] face player
  - [x] move toward player or hold position
  - [x] fire on cooldown
- [x] Spawn enemy into the arena.
- [x] Add mission failure on player destruction.
- [x] Add mission success when threat is destroyed or disabled.

### First-Pass Simplifications

- [x] Prefer a fixed turret platform if mobile AI slows progress.

### Phase 8 Notes

Completed approach:

* the arena threats are now explicit hostile turret platforms rather than anonymous damage dummies
* hostile platforms track the player by rotating to face the ship and then fire on cooldown while holding position
* mission state now distinguishes failure from completion, so the encounter can end cleanly in either direction
* destroying the player core still causes mission failure, while clearing all hostile platforms marks the encounter complete
* player control and hostile fire both stop once the encounter is resolved, keeping the result stable for the next loop phase

## Phase 9 — Salvage Interaction

### Goal

Add the reward half of the loop.

### Tasks

- [x] Add one salvage object or wreck.
- [x] Add a simple interaction range.
- [x] Add a pickup action such as `F`.
- [x] Define reward payload:
  - [x] scrap count
  - [ ] or unlock one module placement
  - [ ] or grant one component token
- [x] Store the reward in a persistent demo resource.
- [x] Show reward summary after mission.

### First-Pass Simplifications

- [x] Do not implement real logistics transport yet.
- [x] Salvage can be direct collection on interaction.

### Phase 9 Notes

Completed approach:

* the runtime arena now includes one salvage wreck with a fixed scrap reward
* salvage can only be collected after the encounter is cleared, which keeps the reward tied to mission success
* the player can pick up salvage directly with `F` inside a simple radius check
* collected scrap is stored in a persistent `DemoProgression` resource on the client, ready for the return loop and later editor-side progression
* the runtime HUD now surfaces salvage state, pickup prompt, collected reward, and total scrap so the payoff is visible immediately

## Phase 10 — Return To Editor Loop

### Goal

Close the vertical slice.

### Tasks

- [x] Add mission completion flow.
- [x] Add mission failure flow.
- [x] Return to editor state after mission ends.
- [x] Preserve edited ship layout.
- [x] Apply salvage reward to editor/progression state.
- [x] Show a lightweight summary screen or banner if needed.

### Phase 10 Notes

Completed approach:

* mission completion and mission failure now both trigger a short resolved-state countdown before automatically returning to the editor
* the edited ship remains stored in the shared `EditorShip` resource, so returning from a run preserves the current layout
* salvage rewards continue to accumulate in the persistent `DemoProgression` resource and are visible again once back in the editor
* a `LastMissionReport` resource now carries the previous run’s outcome, summary text, scrap awarded, and total scrap into the editor HUD
* the editor HUD now acts as the lightweight post-mission summary banner, which closes the loop without adding a separate intermediate screen

## Phase 11 — Minimal Progression Stub

### Goal

Make repeated runs feel purposeful without implementing a full economy.

### Tasks

- [x] Add a small `DemoProgression` resource.
- [ ] Track at least one reward value:
  - [x] `scrap`
  - [ ] `recovered_modules`
- [ ] Add one simple gating mechanic:
  - [x] some components require scrap to place
  - [ ] or new parts unlock after first salvage
- [x] Surface progression in the editor UI.

### First-Pass Simplifications

- [x] One currency is enough.
- [x] Flat placement costs are enough.

### Phase 11 Notes

Completed approach:

* scrap is now a real editor-side progression currency carried by the persistent `DemoProgression` resource
* each module kind has a flat placement cost, and unaffordable placements are blocked at edit time
* replacing a tile only charges the positive cost difference between the old and new module, while deletes do not refund scrap
* toolbox buttons now show module costs and visually distinguish unaffordable options
* the editor HUD shows current scrap, selected module cost, affordability, and last mission summary so the progression loop is immediately visible

## Phase 12 — Usability And Readability

### Goal

Make the slice understandable enough to playtest.

### Tasks

- [x] Add clear HUD labels for:
  - [x] hull/integrity
  - [x] power
  - [x] selected tool in editor
  - [x] mission outcome
- [x] Improve module readability in runtime:
  - [ ] simple selection tint
  - [x] damage tint
  - [x] destruction hide/remove
- [x] Add on-screen controls help for gameplay.
- [x] Add on-screen controls help for editor if needed.

### Phase 12 Notes

Completed approach:

* the editor and runtime HUD text now use clearer labeled sections instead of dense unlabeled values
* both modes now include dedicated on-screen controls panels, so first-time playtesters can operate the slice without external instructions
* runtime damage readability is stronger because damaged modules still tint by integrity while destroyed modules now fade out of view entirely
* the editor HUD now makes progression and mission outcome more legible by grouping scrap, placement cost, and last-mission data together

## Suggested Codebase Expansion

### Near-Term Module Structure

Possible next structure:

* `src/client/`
  * `mod.rs`
  * `state.rs`
  * `menu.rs`
  * `editor.rs`
  * `net.rs`
* `src/ship/`
  * `mod.rs`
  * `data.rs`
  * `spawn.rs`
* `src/gameplay/`
  * `mod.rs`
  * `state.rs`
  * `spawn.rs`
  * `movement.rs`
  * `power.rs`
  * `combat.rs`
  * `enemy.rs`
  * `salvage.rs`

This does not need to appear all at once, but it is a good direction.

## Priority Order

If we want the fastest route to a playable demo, do work in this order:

1. Shared ship model
2. Editor launch into runtime
3. Ship runtime spawn
4. Movement
5. Power
6. Weapons/projectiles
7. Damage/failure
8. Enemy encounter
9. Salvage
10. Return loop
11. Minimal progression
12. Polish/readability

## Definition Of Done For Each Stage

### Stage A Done

* editor ship data can be used outside the editor

### Stage B Done

* clicking launch enters gameplay with the edited ship

### Stage C Done

* player can move the ship in a runtime scene

### Stage D Done

* player can shoot and be shot

### Stage E Done

* one enemy encounter can be won or lost

### Stage F Done

* player can collect salvage and return to the editor

### Stage G Done

* loop is replayable without manual developer setup

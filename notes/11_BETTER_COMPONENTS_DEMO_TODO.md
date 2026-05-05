# TODO — Better Components Demo Implementation Breakdown

This file turns `11_BETTER_COMPONENTS_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* expand component families without breaking the existing ship/save/runtime model
* preserve deterministic simulation and clear command surfaces for future automation
* prefer mechanical differentiation over shallow stat inflation
* keep the editor and station UIs coherent as component count grows
* accept that this slice is broad, but still break it into testable vertical phases

## Expected Areas Of Change

Likely touched modules:

* `src/ship/mod.rs`
* `src/ship/storage.rs`
* `src/client/state.rs`
* `src/client/editor/`
* `src/client/gameplay/components/`
* `src/client/gameplay/helpers/`
* `src/client/gameplay/spawn/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/simulation/`
* `src/client/gameplay/systems/ui/`
* `saves/balance_config.json`

Likely new module areas:

* component variant definitions / shared specs
* ammunition and fuel resource support
* shield simulation helpers
* fabricator mechanics
* core-size / ship-limit validation helpers

---

## Phase 1 — Variant-Capable Data Model

### Goal

Teach the ship/editor/runtime data model how to distinguish component variants cleanly.

### Tasks

- [x] Decide how variants are represented:
  - expanded `ModuleKind`
  - subtype field
  - spec lookup table
- [x] Add save/load support for variant identity.
- [x] Keep existing ships migrating safely to sensible defaults.
- [x] Update editor/tooling to display variant identity clearly.

Definition of done:

* the project can represent multiple variants per component family without ambiguity

---

## Phase 2 — Shared Variant Spec System

### Goal

Avoid hardcoding every new part as unrelated bespoke behavior.

### Tasks

- [x] Define shared spec/config data for major families.
- [x] Move variant-tunable values into config or data tables where practical.
- [x] Distinguish:
  - integrity
  - power use/output
  - heat behavior
  - logistics needs
  - control complexity
- [x] Keep the first pass compatible with deterministic runtime use.

Definition of done:

* new component variants can be added with less duplicated gameplay code

---

## Phase 3 — Weapon Family Expansion

### Goal

Add multiple turret behaviors with real gameplay consequences.

### Tasks

- [x] Add **hitscan laser turret** as the baseline weapon.
- [x] Add **projectile ballistic turret** as the stronger/more complex upgrade.
- [x] Add ammunition requirement for ballistic turrets.
- [x] Add nearby ammo storage dependency and consumption rules.
- [x] Differentiate manual and automated use difficulty between the two.
- [x] Update UI and combat logic accordingly.

Definition of done:

* weapon choice affects logistics, automation difficulty, and combat feel

---

## Phase 4 — Fuel And Reactor Family Expansion

### Goal

Split reactor families into meaningfully different control problems.

### Tasks

- [x] Add **fission reactor** baseline behavior.
- [x] Add **fusion reactor** with more interdependent control variables.
- [x] Move fuel storage out of the reactor body and into neighboring fuel storage.
- [x] Add first-pass fuel transfer/consumption rules.
- [x] Keep both systems manually operable and automatable.
- [x] Update UI to reflect the different control surfaces.

Definition of done:

* reactors are now a family of control/logistics problems rather than one slider pair

---

## Phase 5 — Better Helm / Cockpit Variants

### Goal

Make helm quality a real design choice.

### Tasks

- [x] Add at least one improved cockpit/helm variant.
- [x] Decide what “better control” means mechanically:
  - faster response
  - better damping
  - improved turn authority
  - better visibility / HUD support
- [x] Reflect those benefits in the helm UI and flight model.
- [x] Keep manual and future automated control paths compatible.

Definition of done:

* cockpit choice materially changes piloting quality

---

## Phase 6 — Batteries And Capacitors

### Goal

Make energy storage about more than total capacity.

### Tasks

- [x] Add maximum charge/discharge rate to storage devices.
- [x] Rework batteries around capacity plus rate.
- [x] Add **capacitors** with lower capacity but much higher throughput.
- [x] Ensure power simulation respects these transfer limits.
- [x] Update UI/readouts to show not only stored energy but flow constraints.

Definition of done:

* power buffering decisions now include response profile, not only reserve size

---

## Phase 7 — Core Progression And Ship Limits

### Goal

Make the core the anchor for vessel scale and inertia field size.

### Tasks

- [x] Add better core variants with higher ship-size allowance.
- [x] Define and enforce first-pass maximum supported ship size.
- [x] Link inertia-field size more explicitly to core quality.
- [x] Reflect size/field limits in the editor and runtime.
- [x] Keep current small-core behavior as the baseline.

Definition of done:

* the core becomes the architectural limiter and growth gate it should be

---

## Phase 8 — Fabricators And Resource Conversion

### Goal

Expand logistics into real onboard production choices.

### Tasks

- [x] Add slower and faster fabricator variants.
- [x] Add recipes to turn scrap into:
  - ammunition
  - fuel
- [x] Feed these outputs back into existing storage/logistics systems.
- [x] Show throughput and bottleneck behavior clearly.
- [x] Keep the first recipe set intentionally small.

Definition of done:

* the ship can convert recovered resources into meaningful operational supplies

---

## Phase 9 — Shield Family Introduction

### Goal

Introduce the first shield variants with distinct control demands.

### Tasks

- [x] Add **radial shield generator**.
- [x] Add **directional shield generator**.
- [x] Decide first-pass shield damage/coverage semantics.
- [x] Make directional shielding meaningfully stronger but harder to use.
- [x] Expose command surfaces that future automation can target.
- [x] Add UI/readouts for shield state and directionality.

Definition of done:

* shields become a real control and automation problem, not a generic passive stat

---

## Phase 10 — Automation / ARCH Compatibility Pass

### Goal

Make sure the richer machinery can still be automated coherently.

### Tasks

- [x] Identify writable command surfaces for the new variants.
- [x] Keep hitscan vs ballistic turret commands structurally coherent.
- [x] Keep fission vs fusion reactor controls consistent where possible.
- [x] Add the minimum register/readout support needed for future fuller ARCH work.
- [x] Avoid creating manual-only mechanics that block later automation.

Definition of done:

* better components deepen the automation game instead of fragmenting it

---

## Phase 11 — Editor / UI / UX Integration

### Goal

Keep the expanded part set understandable in refit and operation.

### Tasks
- [x] Diagnose and fix variant availability in the player ship editor.
  Diagnosis:
  the current player editor toolbox and placement flow only reason about `ModuleVariant::default_for_kind(kind)`.
  `toolbox_button_system`, `sync_toolbox_visuals`, `toolbox_label`, and the initial availability checks all ask progression only for the default variant of a module kind, so upgraded variants like `Fusion`, `BallisticTurret`, `Capacitor`, `ExpandedCore`, `DirectionalShield`, and other better-components variants never appear as independently available stock.
  The inventory model already supports `(ModuleKind, ModuleVariant)` pairs in `DemoProgression`, but the editor UI is still kind-centric rather than variant-centric.
- [x] Update the assets/tiles/README.md for new sprites that are expected for upgraded components.
- [x] Refactor the player editor toolbox to present variant-aware entries instead of a single button per `ModuleKind`.
  - [x] Refactor the player ship editor to have a number of top-level tools - select and build for now.
  - [x] Within the build mode, show sprites with tooltips, instead of just buttons for component names, with multiple components per row, with components grouped by type - hulls, reactors, turrets, shields, etc. - with components disabled if none are available in inventory.
  - [x] Within the select mode, show a summary of components currently selected, defaulting to all components currently on the ship when there is no selection.
- [x] Make variant selection inventory-aware, including ready/damaged counts per variant.
- [x] Keep enemy editor on unlimited availability across all variants.
- [x] Improve component placement UX:
  - add a dedicated `Select` tool and a `Place/Delete` tool
  - allow drag placement in `Place/Delete`
  - allow drag deletion in `Place/Delete`
  - add marquee selection in `Select`
  - allow moving selected placed components as a group
  - allow copying selected groups
  - allow deleting selected groups
- [x] Fix rotation behavior so it advances cleanly clockwise instead of occasionally picking up unintended orientation state from replacement/update paths.
- [x] Add an `Auto Hull` action that wraps exposed ship edges using hull edges, inner corners, and outer corners where missing.
- [x] Update editor help text and status text to explain:
  - tool mode
  - current variant
  - selection contents
  - auto-hull behavior
- [x] Keep the overlay/console workflow working cleanly with the new selection and placement modes.

Definition of done:

* better component variants are genuinely visible and placeable in player refit
* the editor supports both quick painting and structured group editing
* rotation and hull cleanup feel deliberate rather than fiddly

- [x] Update the editor toolbox/panels for variant families.
- [x] Add clear naming and grouping for variants.
- [x] Update station UIs to reflect variant-specific controls and readouts.
- [x] Show logistical dependencies like ammo/fuel adjacency clearly.
- [x] Keep the interface usable despite the larger content surface.

Definition of done:

* the larger component library is operable rather than overwhelming

---

## Phase 12 — Encounter / Travel Integration

### Goal

Let the new components appear in normal play, not only test setups.

### Tasks

- [x] Add some enemy and/or node content that uses the new variants.
- [x] Ensure salvage and refit loops can surface the new parts naturally.
- [x] Tune reward and threat so upgraded parts matter in practice.
- [x] Ensure boarding and station-use still work on the new variants.

Definition of done:

* the richer parts ecosystem shows up in the actual campaign loop

---

## Phase 13 — Tuning And Stability Pass

### Goal

Make the broader component ecosystem playable rather than merely implemented.

### Tasks

- [x] Tune variant costs, throughput, and combat value.
- [x] Tune ammunition and fuel burden.
- [x] Tune shield power and control burden.
- [x] Tune batteries/capacitors and reactor families together.
- [x] Verify the expanded component set remains deterministic and readable.

Definition of done:

* component choice creates interesting ship identities instead of obvious dominant picks

## Immediate Next Task

Start with **Phase 1**:

* settle the variant representation model
* ensure save/load migration is safe
* decide how much of the behavior should be data-driven from the outset

That decision shapes every later part of the slice.

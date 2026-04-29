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

- [ ] Decide how variants are represented:
  - expanded `ModuleKind`
  - subtype field
  - spec lookup table
- [ ] Add save/load support for variant identity.
- [ ] Keep existing ships migrating safely to sensible defaults.
- [ ] Update editor/tooling to display variant identity clearly.

Definition of done:

* the project can represent multiple variants per component family without ambiguity

---

## Phase 2 — Shared Variant Spec System

### Goal

Avoid hardcoding every new part as unrelated bespoke behavior.

### Tasks

- [ ] Define shared spec/config data for major families.
- [ ] Move variant-tunable values into config or data tables where practical.
- [ ] Distinguish:
  - integrity
  - power use/output
  - heat behavior
  - logistics needs
  - control complexity
- [ ] Keep the first pass compatible with deterministic runtime use.

Definition of done:

* new component variants can be added with less duplicated gameplay code

---

## Phase 3 — Weapon Family Expansion

### Goal

Add multiple turret behaviors with real gameplay consequences.

### Tasks

- [ ] Add **hitscan laser turret** as the baseline weapon.
- [ ] Add **projectile ballistic turret** as the stronger/more complex upgrade.
- [ ] Add ammunition requirement for ballistic turrets.
- [ ] Add nearby ammo storage dependency and consumption rules.
- [ ] Differentiate manual and automated use difficulty between the two.
- [ ] Update UI and combat logic accordingly.

Definition of done:

* weapon choice affects logistics, automation difficulty, and combat feel

---

## Phase 4 — Fuel And Reactor Family Expansion

### Goal

Split reactor families into meaningfully different control problems.

### Tasks

- [ ] Add **fission reactor** baseline behavior.
- [ ] Add **fusion reactor** with more interdependent control variables.
- [ ] Move fuel storage out of the reactor body and into neighboring fuel storage.
- [ ] Add first-pass fuel transfer/consumption rules.
- [ ] Keep both systems manually operable and automatable.
- [ ] Update UI to reflect the different control surfaces.

Definition of done:

* reactors are now a family of control/logistics problems rather than one slider pair

---

## Phase 5 — Better Helm / Cockpit Variants

### Goal

Make helm quality a real design choice.

### Tasks

- [ ] Add at least one improved cockpit/helm variant.
- [ ] Decide what “better control” means mechanically:
  - faster response
  - better damping
  - improved turn authority
  - better visibility / HUD support
- [ ] Reflect those benefits in the helm UI and flight model.
- [ ] Keep manual and future automated control paths compatible.

Definition of done:

* cockpit choice materially changes piloting quality

---

## Phase 6 — Batteries And Capacitors

### Goal

Make energy storage about more than total capacity.

### Tasks

- [ ] Add maximum charge/discharge rate to storage devices.
- [ ] Rework batteries around capacity plus rate.
- [ ] Add **capacitors** with lower capacity but much higher throughput.
- [ ] Ensure power simulation respects these transfer limits.
- [ ] Update UI/readouts to show not only stored energy but flow constraints.

Definition of done:

* power buffering decisions now include response profile, not only reserve size

---

## Phase 7 — Core Progression And Ship Limits

### Goal

Make the core the anchor for vessel scale and inertia field size.

### Tasks

- [ ] Add better core variants with higher ship-size allowance.
- [ ] Define and enforce first-pass maximum supported ship size.
- [ ] Link inertia-field size more explicitly to core quality.
- [ ] Reflect size/field limits in the editor and runtime.
- [ ] Keep current small-core behavior as the baseline.

Definition of done:

* the core becomes the architectural limiter and growth gate it should be

---

## Phase 8 — Fabricators And Resource Conversion

### Goal

Expand logistics into real onboard production choices.

### Tasks

- [ ] Add slower and faster fabricator variants.
- [ ] Add recipes to turn scrap into:
  - ammunition
  - fuel
- [ ] Feed these outputs back into existing storage/logistics systems.
- [ ] Show throughput and bottleneck behavior clearly.
- [ ] Keep the first recipe set intentionally small.

Definition of done:

* the ship can convert recovered resources into meaningful operational supplies

---

## Phase 9 — Shield Family Introduction

### Goal

Introduce the first shield variants with distinct control demands.

### Tasks

- [ ] Add **radial shield generator**.
- [ ] Add **directional shield generator**.
- [ ] Decide first-pass shield damage/coverage semantics.
- [ ] Make directional shielding meaningfully stronger but harder to use.
- [ ] Expose command surfaces that future automation can target.
- [ ] Add UI/readouts for shield state and directionality.

Definition of done:

* shields become a real control and automation problem, not a generic passive stat

---

## Phase 10 — Automation / ARCH Compatibility Pass

### Goal

Make sure the richer machinery can still be automated coherently.

### Tasks

- [ ] Identify writable command surfaces for the new variants.
- [ ] Keep hitscan vs ballistic turret commands structurally coherent.
- [ ] Keep fission vs fusion reactor controls consistent where possible.
- [ ] Add the minimum register/readout support needed for future fuller ARCH work.
- [ ] Avoid creating manual-only mechanics that block later automation.

Definition of done:

* better components deepen the automation game instead of fragmenting it

---

## Phase 11 — Editor / UI / UX Integration

### Goal

Keep the expanded part set understandable in refit and operation.

### Tasks

- [ ] Update the editor toolbox/panels for variant families.
- [ ] Add clear naming and grouping for variants.
- [ ] Update station UIs to reflect variant-specific controls and readouts.
- [ ] Show logistical dependencies like ammo/fuel adjacency clearly.
- [ ] Keep the interface usable despite the larger content surface.

Definition of done:

* the larger component library is operable rather than overwhelming

---

## Phase 12 — Encounter / Travel Integration

### Goal

Let the new components appear in normal play, not only test setups.

### Tasks

- [ ] Add some enemy and/or node content that uses the new variants.
- [ ] Ensure salvage and refit loops can surface the new parts naturally.
- [ ] Tune reward and threat so upgraded parts matter in practice.
- [ ] Ensure boarding and station-use still work on the new variants.

Definition of done:

* the richer parts ecosystem shows up in the actual campaign loop

---

## Phase 13 — Tuning And Stability Pass

### Goal

Make the broader component ecosystem playable rather than merely implemented.

### Tasks

- [ ] Tune variant costs, throughput, and combat value.
- [ ] Tune ammunition and fuel burden.
- [ ] Tune shield power and control burden.
- [ ] Tune batteries/capacitors and reactor families together.
- [ ] Verify the expanded component set remains deterministic and readable.

Definition of done:

* component choice creates interesting ship identities instead of obvious dominant picks

## Immediate Next Task

Start with **Phase 1**:

* settle the variant representation model
* ensure save/load migration is safe
* decide how much of the behavior should be data-driven from the outset

That decision shapes every later part of the slice.


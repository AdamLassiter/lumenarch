# TODO — UI Demo Implementation Breakdown

This file turns `10_UI_DEMO.md` into an implementation plan tied to the current codebase.

## Guiding Constraints

* improve readability without discarding the current station/control architecture
* keep manual interaction routed through the same deterministic command surfaces already in place
* prefer interactive controls over pure text, but keep the first pass pragmatic enough to ship
* make the HUD easier to read during live play before chasing final visual polish
* ensure the basic ARCH editor is usable in-engine before attempting the later fuller language slice

## Expected Areas Of Change

Likely touched modules:

* `src/client/mod.rs`
* `src/client/state.rs`
* `src/client/docked.rs`
* `src/client/sector_map.rs`
* `src/client/editor/`
* `src/client/gameplay/components/`
* `src/client/gameplay/spawn/scene/`
* `src/client/gameplay/systems/control.rs`
* `src/client/gameplay/systems/ui/`
* `src/client/gameplay/systems/interactions/`
* `src/ship/arch.rs`

Likely new module areas:

* shared panel widgets / layout helpers
* station-panel view-state resources
* ARCH editor widget/state helpers
* iconography / panel-specific UI markers

---

## Phase 1 — General UI Audit And Layout State

### Goal

Define the shared UI state and layout scaffolding needed to clean up the current presentation.

### Tasks

- [x] Audit current docked, sector, editor, and encounter UI panels.
- [x] Identify overlapping or redundant panes.
- [x] Add view-state resources for:
  - panel visibility
  - focused station panel mode
  - temporary modal/editor state
- [x] Define a shared layout approach for:
  - top-level HUD
  - side panels
  - modal/full-screen station panels

Definition of done:

* the UI has a clear structural plan instead of ad hoc panel growth

---

## Phase 2 — Encounter HUD Refresh

### Goal

Make the general in-sector HUD readable during normal play.

### Tasks

- [x] Rework the encounter HUD into clearer sections.
- [x] Reduce text duplication across status and alert panels.
- [x] Separate always-on essentials from detailed inspection info.
- [x] Make flight, EVA, boarding, and station-use states visibly distinct.
- [x] Keep debug/diagnostic overlays separate from the main player HUD.

Definition of done:

* the player can understand the live encounter state without reading a wall of text

---

## Phase 3 — Docked / Sector / Editor UX Refresh

### Goal

Bring the non-encounter UI up to the same standard as the runtime HUD.

### Tasks

- [x] Improve docked screen hierarchy and spacing.
- [x] Improve sector-map node details and risk/reward presentation.
- [x] Improve editor sidebars and tool/status grouping.
- [x] Keep save/load and enemy-ship editing controls legible.
- [x] Ensure these screens feel visually related but functionally distinct.

Definition of done:

* the outer loop no longer feels like a collection of debug screens

---

## Phase 4 — Shared Station Panel Framework

### Goal

Create the reusable panel shell for full-screen or near-full-screen station interaction.

### Tasks

- [x] Define a shared station-panel root layout.
- [x] Add title, context, status, and controls regions.
- [x] Add common close/exit affordances.
- [x] Support per-station custom content areas.
- [x] Keep panel transitions compatible with the existing station focus flow.

Definition of done:

* component stations can open richer UIs without each one reinventing panel plumbing

---

## Phase 5 — Helm UI

### Goal

Turn cockpit use into a real graphical helm interface.

### Tasks

- [x] Add throttle slider / lever visualization.
- [x] Add steering wheel, heading bar, or equivalent turn-demand visualization.
- [x] Show desired vs actual ship response where useful.
- [x] Support both keyboard and mouse interaction cleanly.
- [x] Keep the world-view camera behavior aligned with the panel.

Definition of done:

* piloting feels like operating a helm station, not only pressing movement keys

---

## Phase 6 — Turret UI

### Goal

Make turret control graphical and locally readable.

### Tasks

- [x] Add a turret panel with local aim display.
- [x] Show desired vs actual turret angle.
- [x] Show cooldown, power state, and readiness.
- [x] Add basic target/lead cues if needed for readability.
- [x] Keep ballistic-vs-hitscan future growth in mind without requiring it yet.

Definition of done:

* the player can visually understand turret behavior and aiming lag

---

## Phase 7 — Reactor UI

### Goal

Replace text-first reactor operation with a graphical control surface.

### Tasks

- [x] Add graphical controls for reaction rate and turbine load.
- [x] Add gauges/bars/graphs for:
  - heat
  - power output
  - fuel
  - stability or electrical condition where applicable
- [x] Show input/output relationships clearly enough to teach the system.
- [x] Keep the UI compatible with future fission/fusion divergence.

Definition of done:

* the reactor is understandable at a glance and operable without reading raw numbers alone

---

## Phase 8 — Logistics Panels

### Goal

Make logistics interaction visually operational rather than mostly textual.

### Tasks

- [x] Add storage inventory presentation with icons or clear slots/lists.
- [x] Add manipulator route/source/target visualization.
- [x] Add processor recipe/progress presentation.
- [x] Show blockages, starvation, and full-capacity states clearly.
- [x] Keep airlock logistics and carried-cargo interaction readable.

Definition of done:

* logistics feels like running machinery, not inspecting counters

---

## Phase 9 — Computer Panel And Basic ARCH Editor

### Goal

Ship the first usable in-game ARCH editing interface.

### Tasks

- [x] Add a dedicated computer/ARCH station panel.
- [x] Show program name, enabled state, and recent execution status.
- [x] Add a basic line-oriented program editor:
  - insert line
  - remove line
  - reorder line
  - change opcode
  - change operands / constants
- [x] Add validation / parse / runtime error feedback.
- [x] Save edited programs back into ship data cleanly.

Definition of done:

* the player can meaningfully edit ship automation in-engine without relying only on templates

---

## Phase 10 — Input And Interaction Pass

### Goal

Ensure the richer UI does not fight the rest of the game’s input model.

### Tasks

- [x] Prevent gameplay clicks/keys from leaking through active panels.
- [x] Support mouse interaction for the new controls.
- [x] Preserve keyboard paths for rapid operation.
- [x] Ensure panel focus / close behavior is predictable.
- [x] Verify editor and encounter interactions remain separate and safe.

Definition of done:

* richer UI is actually usable during play and does not introduce input ambiguity

---

## Phase 11 — Visual Language And Readability Pass

### Goal

Give the new UI a coherent visual identity.

### Tasks

- [x] Define a consistent panel visual language.
- [x] Add icons, bars, dials, highlights, and color semantics consistently.
- [x] Reduce verbosity where visual encoding can replace text.
- [x] Keep danger and urgency readable without overwhelming the player.
- [x] Preserve debugging access where still needed.

Definition of done:

* the UI feels intentionally designed rather than debug-first

---

## Phase 12 — Playtest And Tuning

### Goal

Make sure the improved UI actually improves play.

### Tasks

- [ ] Check that the player can fight, board, and stabilize without losing track of key info.
- [ ] Check that each station panel teaches its system faster than the old text UI.
- [ ] Check that the basic ARCH editor is understandable enough to iterate with.
- [ ] Tune panel density, size, and hierarchy.
- [ ] Remove or demote any remaining redundant information.

Definition of done:

* the UI materially improves decision-making and comprehension

## Immediate Next Task

Start with **Phase 1**:

* audit the current HUD/panel sprawl
* define shared panel/view state
* decide what stays always-on versus what becomes modal or station-local

That is the foundation the rest of the UI slice depends on.

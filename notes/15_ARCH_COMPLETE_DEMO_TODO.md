# TODO — ARCH Complete Demo Implementation Breakdown

This file turns [15_ARCH_COMPLETE_DEMO.md](/home/adaml/code/lumenarch/notes/15_ARCH_COMPLETE_DEMO.md) into an implementation plan tied to the current codebase.

## Guiding Constraints

* keep the control surface deterministic and shared between manual play and programming
* do not let draft text mutate live program state until parsing/commit succeeds
* treat channel visibility as part of the programming surface, not a cosmetic UI extra
* reuse one text-authoring foundation for both ARCH and LUMEN
* prefer clear engineering-style error feedback over editor complexity

## Expected Areas Of Change

Likely touched modules:

* `src/ship/arch.rs`
* `src/ship/lumen.rs`
* `src/ship/mod.rs`
* `src/editor/`
* `src/gameplay/components/modules.rs`
* `src/gameplay/components/ship.rs`
* `src/gameplay/systems/ui/status/`
* `src/gameplay/systems/interactions/`
* `src/state/editor.rs`
* `src/state/frontend.rs`
* `docs/src/ARCHLANG.md`
* `docs/src/LUMENLANG.md`

Likely new module areas:

* parser/tokenizer helpers
* shared program text editor state
* program diagnostics types
* register-surface metadata
* channel assignment helpers

---

## Phase 1 — Canonical Control Surface Audit

### Goal

Define the exact author-facing register and channel model before building more UI.

### Tasks

- [ ] Audit current runtime readable/writable state exposed by:
  - [ ] reactor
  - [ ] turret
  - [ ] cockpit/helm
  - [ ] storage/manipulator/processor
  - [ ] shields/engines/computers where already meaningful
- [ ] Decide the canonical register names shown in component UIs.
- [ ] Decide which component families are channel-addressable in v1.
- [ ] Decide whether duplicate channels are:
  - [ ] allowed
  - [ ] warned about
  - [ ] blocked
- [ ] Align the agreed runtime surface with `ARCHLANG` and `LUMENLANG`, or explicitly document deltas.

Definition of done:

* there is one stable register/channel contract for runtime, editor, parser, and docs

---

## Phase 2 — Channel Data Model

### Goal

Make channels real saved ship/module state rather than spec-only meaning.

### Tasks

- [x] Add explicit per-module channel data to ship definitions for channel-capable components.
- [x] Add save/load and migration behavior for existing ships.
- [x] Add enemy ship support using the same model.
- [x] Decide defaults for newly placed modules by kind.
- [x] Add helpers for:
  - [x] reading a module channel
  - [x] mutating a module channel
  - [x] enumerating modules by channel/family

Definition of done:

* placed modules can carry stable saved channel assignments

---

## Phase 3 — Channel Editing In Refit And Runtime

### Goal

Expose channels to players wherever components are configured or operated.

### Tasks

- [x] Add channel controls to the player ship editor.
- [x] Add channel display and change controls to component runtime UIs.
- [ ] Surface channel warnings where multiple relevant modules share a channel.
- [x] Keep enemy editor on unlimited supply while still using the same channel data model.
- [ ] Make sure runtime channel changes follow the chosen synchronization rules.

Definition of done:

* the player can inspect and intentionally configure channels in both editor and gameplay

---

## Phase 4 — Register-Named Component UIs

### Goal

Show the real programming surface directly in component UIs.

### Tasks

- [x] For each component UI, add register-name labels next to values.
- [ ] Distinguish readable and writable registers where useful.
- [x] Ensure bar/toggle displays still remain readable and not overloaded.
- [x] Add channel display to each relevant panel.
- [x] Make focused-module and station-console views reflect the same register naming.

Definition of done:

* the player can inspect a component and see the real program-visible surface without cross-referencing docs

---

## Phase 5 — Shared Multi-Line Text Editor

### Goal

Upgrade the current textbox system into a bounded multi-line programming editor.

### Tasks

- [x] Add a reusable multi-line text buffer resource/state model.
- [x] Support:
  - [x] cursor movement
  - [x] selection basics
  - [x] line splitting/joining
  - [x] left/right/up/down
  - [x] home/end
  - [x] backspace/delete
  - [x] ctrl-a / ctrl-x / ctrl-c / ctrl-v
- [x] Keep a fixed maximum line count in v1.
- [x] Add a visible active line/cursor presentation.
- [x] Make the editor reusable for both ARCH and LUMEN program text.

Definition of done:

* the project has one generic bounded text editor suitable for in-game programming

---

## Phase 6 — ARCH Text Parser

### Goal

Parse real text into ARCH program structures.

### Tasks

- [x] Add tokenizer/parser for ARCH source text.
- [ ] Support:
  - [ ] labels
  - [x] comments
  - [x] opcodes
  - [x] register operands
  - [x] immediate literals
- [ ] Validate:
  - [x] opcode existence
  - [x] argument counts
  - [x] writable destination rules
  - [x] known register names
  - [ ] legal channel suffixes
  - [x] control-flow constraints
- [x] Produce structured diagnostics with line/column or at least line/token context.

Definition of done:

* a text ARCH source buffer can become a validated program or a useful diagnostic list

---

## Phase 7 — LUMEN Text Parser

### Goal

Give LUMEN the same text-authoring foundation, even if ARCH remains the focus of the slice.

### Tasks

- [x] Add parser for the current canonical LUMEN text syntax.
- [x] Support BUFF/NERF and the currently implemented target/property vocabulary.
- [ ] Validate:
  - [x] target form
  - [ ] register/property family
  - [ ] channel references
  - [x] operand count
  - [x] numeric/weight arguments
- [x] Produce diagnostics consistent with ARCH diagnostics.

Definition of done:

* LUMEN is text-authorable through the same UI model, not left behind on a separate structured editor forever

---

## Phase 8 — Draft / Compile / Commit Model

### Goal

Prevent invalid draft text from corrupting saved or active programs.

### Tasks

- [x] Add per-processor draft text state.
- [x] Separate:
  - [x] active compiled program
  - [x] saved authored source text
  - [x] current draft text
- [x] Add explicit actions for:
  - [x] parse/check
  - [x] commit/apply
  - [x] revert to active/saved
- [x] Ensure failed parse/validation does not replace the active program.

Definition of done:

* players can experiment with text safely and understand which program is actually live

---

## Phase 9 — Runtime Processor UI

### Goal

Make interacting with computers/processors a real programming activity in-game.

### Tasks

- [ ] Add a runtime programming panel for ARCH computers.
- [ ] Add a runtime programming panel for LUMEN processors.
- [ ] Show:
  - [x] current channel context where relevant
  - [x] source text
  - [ ] parse/validation errors
  - [x] active program summary
  - [x] last runtime halt/error state
- [x] Ensure the UI fits the current encounter presentation rules and panel model.

Definition of done:

* the player can meaningfully inspect and edit code while operating a programming-capable station

---

## Phase 10 — Error Handling And Diagnostics

### Goal

Make invalid authored programs understandable and actionable.

### Tasks

- [ ] Define a shared diagnostic model for parser/validator/runtime failures.
- [ ] Show line-specific error messages in editor and runtime programming panels.
- [ ] Distinguish:
  - [x] parse error
  - [x] validation error
  - [x] runtime halt/error
  - [x] inactive draft vs active program
- [ ] Add logs for invalid program activation attempts without spamming.

Definition of done:

* program failure states are clear to players and useful for debugging

---

## Phase 11 — Synchronization And Ownership Rules

### Goal

Close the design gap around how program and channel edits work in synchronized play.

### Tasks

- [ ] Decide and document multiplayer ownership rules for:
  - [ ] docked editor program changes
  - [ ] docked editor channel changes
  - [ ] runtime channel edits
  - [ ] runtime ARCH commits
  - [ ] runtime LUMEN commits
- [ ] Implement the chosen synchronized commit path for any in-session edits that are allowed.
- [x] Keep local drafts local until commit.
- [x] Ensure presentation/editor resources do not become gameplay authority again.

Definition of done:

* programming and channel changes obey explicit multiplayer/rollback rules instead of ad hoc local mutation

---

## Phase 12 — Docs And Final Alignment

### Goal

Bring the design docs back in line with the actual shipped implementation.

### Tasks

- [ ] Update `docs/src/ARCHLANG.md` to match the implemented parser/register/channel model.
- [ ] Update `docs/src/LUMENLANG.md` where target/channel rules changed.
- [ ] Note any intentionally deferred language features.
- [ ] Add short usage help for the in-game text editor and program workflow.

Definition of done:

* the docs describe the real programmable surface players see in game

---

## High-Risk Gaps To Watch

- [x] Current runtime/editor code still contains structured template-edit assumptions that may fight the text-authoring model.
- [x] The existing textbox implementation may need one more abstraction step before it cleanly supports multi-line editing.
- [ ] Register naming drift between docs, runtime internals, and UI labels could create player confusion unless normalized early.
- [ ] Runtime program editing in rollback/multiplayer needs explicit ownership boundaries before implementation, not after.
- [ ] Channel semantics must be chosen deliberately; forced uniqueness and tactical channel-sharing lead to very different game behavior.

---

## Slice Outcome

This slice is complete when the player can:

* configure channels on placed components
* inspect those channels and canonical register names in UIs
* open an ARCH or LUMEN programming-capable station
* edit a bounded text program
* parse and validate it
* see useful errors if it is invalid
* commit it safely if it is valid
* observe the resulting behavior in the same mission/runtime systems the ship already uses

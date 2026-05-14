# TODO — ARCH Demo Implementation Breakdown

This file turns [04_ARCH_DEMO.md](/home/adaml/code/lumenarch/notes/04_ARCH_DEMO.md) into an implementation plan tied to the current codebase.

It now explicitly assumes [05_COMPONENTS_DEMO.md](/home/adaml/code/lumenarch/notes/05_COMPONENTS_DEMO.md) is the immediate next vertical slice and that ARCH should build on the register-backed component interaction model defined there.

## Guiding Constraints

* manual and autonomous control must share the same writable command surfaces
* keep the first register surface small and curated
* keep execution bounded and deterministic
* prefer readable failure modes over broad feature count
* use existing reactor, logistics, combat, and computer hooks rather than inventing parallel systems

## Prerequisite From 05

Before or alongside major ARCH runtime work, 05 should establish:

* component-local control panels
* ship-local onboard camera behavior
* cockpit and turret command surfaces
* reactor control surface
* logistics control surface
* staged command writes consumed by simulation

ARCH should target those surfaces instead of bypassing them.

## Expected Areas Of Change

Likely touched files and modules:

* `src/client/editor/`
* `src/client/gameplay/components/`
* `src/client/gameplay/systems/`
* `src/client/gameplay/spawn/`
* `src/client/state.rs`
* `src/ship/`

## Phase 1 — Canonical Program Model

Goal:
Define the smallest real ARCH program representation that can be saved, loaded, and attached to a computer module.

Tasks:

- [x] Add a new shared data model for ARCH programs.
- [x] Define instruction, operand, and register identifier enums or structs.
- [x] Keep the first instruction subset intentionally narrow:
  - `MOV`
  - `ADD`
  - `SUB`
  - `GT` / `LT` / `EQ`
  - `JMP` / `JNZ`
- [x] Support serialization so ships with configured programs can be saved and loaded.
- [x] Decide where a computer’s authored program lives in the shared ship model.

Definition of done:

* a saved ship can include at least one authored ARCH program payload

Implementation notes:

* added `src/ship/arch.rs`
* computer programs now live on `ShipModule.arch_program`
* new computer modules default to a starter `BalancedOps` program

## Phase 2 — Register Bank And Command Surface Mapping

Goal:
Expose a small register bank that maps directly to the manual station surfaces introduced by 05.

Tasks:

- [x] Define the first readable register set.
- [x] Define the first writable command registers for:
  - cockpit movement demand
  - turret desired angle / fire gate
  - reactor reaction rate / turbine load
  - logistics priority / recipe / transfer intent
- [x] Build a deterministic per-tick register snapshot step.
- [x] Build deterministic write staging into next-state command buffers.

Definition of done:

* ARCH can write the same effective controls the player can manipulate manually

Implementation notes:

* the current shipped register surface focuses on existing runtime hooks:
  - ship power reserve / average heat / mission threat
  - reactor heat / instability
  - storage and processor inventories
  - turret readiness / cooldown
* writable command registers currently stage:
  - reactor cooling bias
  - logistics enable / preference
  - turret assist / auto-fire
* cockpit and reactor fine-grained component-surface mapping should be retargeted onto 05 command panels once those land

## Phase 3 — Tick-Bounded Interpreter

Goal:
Run a real deterministic ARCH program each fixed tick.

Tasks:

- [x] Implement a small interpreter over the Phase 1 instruction set.
- [x] Enforce a per-tick instruction budget.
- [x] Ensure reads come from current state and writes stage into next state.
- [x] Halt safely on invalid instructions or bad operands.
- [x] Record useful execution outcomes:
  - executed instruction count
  - halted / invalid status
  - recent writes

Definition of done:

* a computer program can execute once per tick and produce deterministic staged outputs

Implementation notes:

* execution is bounded per computer by an instruction budget
* backward jumps are rejected in this first slice to keep control flow simple and deterministic
* invalid register writes and budget exhaustion are surfaced in runtime diagnostics

## Phase 4 — Computer Host Integration

Goal:
Make computer modules real automation hosts rather than abstract mode toggles.

Tasks:

- [x] Extend runtime computer components with:
  - hosted program
  - budget / speed
  - last execution result
  - online / offline state
- [x] Route destroyed or disabled computers to a clear automation failure mode.
- [x] Preserve current hardcoded automation behaviors only as starter-template behavior references.

Definition of done:

* computers execute hosted logic at runtime and ship behavior changes when they fail

Implementation notes:

* runtime computers are now explicit ECS components
* the old automation presets have been replaced by starter templates plus execution summaries
* computer interaction now toggles a hosted runtime online or offline instead of cycling abstract modes

## Phase 5 — Editor Authoring Workflow

Goal:
Let the player configure ARCH before a mission.

Tasks:

- [x] Add editor UI for selecting a computer module and assigning a program.
- [x] Add starter program templates:
  - `ReactorGuard`
  - `LogisticsFeed`
  - `TurretAssist`
  - `BalancedOps`
- [x] Allow limited editing:
  - tweak constants
  - enable / disable lines
  - possibly reorder a tiny instruction list
- [x] Persist the authored program with the ship save.

Definition of done:

* the player can alter a ship’s ARCH behavior in the editor without touching code

Implementation notes:

* the current editor UI supports:
  - per-computer template cycling
  - adjustment of two starter constants
* line enable/disable and instruction reordering remain intentionally deferred to a later authoring pass

## Phase 6 — Runtime ARCH UI

Goal:
Make automation legible during missions.

Tasks:

- [x] Add a runtime ARCH status panel.
- [x] Show:
  - active program
  - execution budget use
  - recent output writes
  - invalid or blocked status
- [x] Surface the currently selected computer in inspection UI.
- [x] Ensure the display complements, rather than replaces, the component-local UIs from 05.

Definition of done:

* a player can tell what automation is trying to do during a mission

## Phase 7 — Reactor Automation Hook

Goal:
Convert reactor help into true register-driven automation.

Tasks:

- [x] Define readable reactor registers.
- [x] Define writable reactor control registers.
- [x] Make one starter script successfully protect the reactor under stress.
- [x] Remove duplicated special-case behavior where it conflicts with scripted output.

Definition of done:

* `ReactorGuard` is a true program, not a hidden special case

## Phase 8 — Logistics Automation Hook

Goal:
Make cargo flow visibly better or worse based on real authored logic.

Tasks:

- [x] Expose storage / processor / transfer state via registers.
- [x] Allow program output to influence manipulator task priority and processor settings.
- [x] Let one starter script keep processors fed more reliably than no script.
- [x] Keep the behavior deterministic and inspectable.

Definition of done:

* automation changes logistics throughput in a measurable way

## Phase 9 — Turret / Combat-Support Hook

Goal:
Give ARCH one combat-adjacent responsibility grounded in the same turret command surface the player uses manually.

Tasks:

- [x] Expose turret actual vs desired state through registers.
- [x] Allow program output to influence turret desired angle or fire policy.
- [x] Ensure this support helps but does not replace the player.
- [x] Tune the encounter so the value is visible.

Definition of done:

* a combat-support script is observably useful under pressure

Implementation notes:

* this slice now exposes turret desired and actual angle as read registers (`WTD0`, `WAC0`)
* this first slice still ships fire-policy support rather than writable desired-angle aiming

## Phase 10 — Encounter And Reporting

Goal:
Make the mission structure teach the player about automation quality.

Tasks:

- [x] Tune one scenario around overlapping system pressures.
- [x] Record mission-side ARCH outcomes:
  - automation triggers
  - invalid executions
  - stalled outputs
  - domains helped
- [x] Return that data to the editor mission report.
- [x] Add redesign hints that mention control logic, not just module layout.

Definition of done:

* post-mission feedback helps the player improve scripts between runs

## Phase 11 — Readability And Onboarding

Goal:
Reduce the intimidation factor of first-time scripting.

Tasks:

- [x] Add short inline descriptions for the first register set.
- [x] Add starter-program explanations.
- [x] Improve diagnostics for:
  - invalid register access
  - no-op writes
  - budget exhaustion
- [x] Keep terminology aligned with the component panels from 05.

Definition of done:

* the first usable ARCH experience is understandable without external docs

Implementation notes:

* this phase is partially complete
* runtime diagnostics and mission reporting are in place
* editor-side explanatory copy can be expanded once 05 component panels establish the final shared terminology
* component panels and editor/runtime program views now show register names, source previews, execution status, and template names

## Immediate Next Task For 04

Current follow-up from 04:

* finish 05 component panels so cockpit, turret, reactor, and logistics manual controls become the true shared command surfaces
* expand ARCH authoring beyond template-plus-constants into line-level editing
* add richer turret and cockpit command registers once those surfaces exist

That is the cleanest way to keep ARCH aligned with the game’s actual interaction model.

# TODO — Logistics Demo Implementation Breakdown

## Purpose

This file turns `LOGISTICS_DEMO.md` into a concrete implementation checklist for the current codebase.

The focus is:

* move from ship repair and one-off automation into real onboard flow
* keep scope tight around one complete logistics workflow
* sequence work so each step unlocks visible progress
* avoid prematurely implementing full generalized logistics, broad crafting, drones, or a full ARCH interpreter

---

## Current Baseline

Already in place from the playable and systems demos:

* shared editor/runtime ship pipeline
* launch-to-runtime mission flow
* shipboard player movement and interaction
* component runtime state, fields, repair, and stabilization
* first ARCH slice through the `computer` module
* hostile encounter flow
* salvage and return-to-editor loop
* mission reporting and editor feedback HUD
* fixed-point gameplay simulation scaffolding

Now in place for this milestone:

* tangible onboard cargo and resource flow
* first playable storage/manipulation/processing loop
* logistics-focused ARCH coordination

---

## Phase 1 — Resource Representation

### Goal

Turn recovered salvage into real runtime resource state instead of immediate abstract reward.

### Tasks

- [x] Audit the current salvage flow and identify where immediate scrap award should be replaced or supplemented.
- [x] Add a first-pass resource model:
  - [x] `ResourceKind`
  - [x] `ResourceInventory`
  - [x] runtime mission/resource telemetry
- [x] Decide where first-pass resource state lives:
  - [x] directly on logistics module entities
  - [x] mirrored into mission and return-report state where needed
- [x] Update mission/runtime state so collected salvage can persist as onboard material during a mission.
- [x] Decide the first resource kinds:
  - [x] raw salvage
  - [x] repair charge

### Phase 1 Notes

* salvage is now recovered as `RawSalvage` rather than immediately paying out progression scrap
* module-local inventories are the authoritative first-pass runtime source
* mission/report state mirrors recovered, processed, consumed, and transferred quantities for UI and editor feedback

### Phase 1 Definition Of Done

* recovered salvage becomes tangible runtime resource state
* the player can identify what resource exists and where it currently resides

---

## Phase 2 — `LS` Storage Slice

### Goal

Make storage modules hold real resource quantities with visible capacity and contents.

### Tasks

- [x] Add a dedicated storage runtime component:
  - [x] `StorageModule`
  - [x] `StorageInventory`
  - [x] capacity rules
- [x] Decide how storage capacity is measured:
  - [x] simple unit capacity
- [x] Add insertion and extraction rules for first-pass resources.
- [x] Define first storage targets:
  - [x] `cargo`
  - [x] `airlock` as intake-adjacent storage
- [x] Add runtime inspection UI for storage contents.
- [x] Add blocked/full state visibility.

### Phase 2 Notes

* `cargo` is the first broad storage module
* `airlock` doubles as intake storage for recovered salvage
* storage contents and fullness are visible in the runtime inspection/alerts UI

### Phase 2 Definition Of Done

* at least one module type can store resources
* the player can inspect that storage and understand its current contents and capacity

---

## Phase 3 — `LM` Manipulator Transfer Slice

### Goal

Allow resources to move between ship modules in a readable and bottleneck-friendly way.

### Tasks

- [x] Add a manipulator runtime model:
  - [x] `ManipulatorModule`
  - [x] transfer progress
  - [x] blocked/idle/active state
- [x] Decide how transfer links are determined:
  - [x] short deterministic local reach around the manipulator
- [x] Add source/destination selection for first-pass transfers.
- [x] Add progress over time rather than instant teleportation.
- [x] Add blocked/idle/active states and make them visible.
- [x] Ensure manipulator performance can be affected by:
  - [x] damage
  - [x] power/system stress through disabled module state

### Phase 3 Notes

* `airlock` is the first manipulator-capable logistics module
* transfers are deterministic and proximity-limited rather than full-route pathfinding
* the slice supports one active transfer at a time with visible bottlenecks

### Phase 3 Definition Of Done

* a manipulator can move a resource from one module to another
* blocked and active transfer states are readable

---

## Phase 4 — `LP` Processor Slice

### Goal

Prove one real onboard conversion workflow.

### Tasks

- [x] Add a processor runtime model:
  - [x] `ProcessorModule`
  - [x] process progress/state
  - [x] input/output inventory
- [x] Define one first-pass recipe:
  - [x] raw salvage -> repair charge
- [x] Add input and output buffer behavior.
- [x] Add processing progress over time.
- [x] Add power/heat-linked operating constraints.
- [x] Add blocked states:
  - [x] no input
  - [x] no output room
  - [x] no power

### Phase 4 Notes

* `processor` is now a real module kind in the ship model/editor/runtime
* the first recipe consumes raw salvage and produces repair charge
* blocked reasons are surfaced directly in the runtime inspection and alerts UI

### Phase 4 Definition Of Done

* one processor consumes a resource input and produces a useful output
* the player can understand when and why the processor is running or blocked

---

## Phase 5 — Salvage Intake Path

### Goal

Connect the encounter loop to ship logistics instead of awarding salvage directly.

### Tasks

- [x] Audit the current salvage wreck flow and replace direct scrap payout.
- [x] Decide how salvage enters the ship:
  - [x] through airlock/cargo intake storage
- [x] Add one first-pass deterministic intake rule.
- [x] Ensure salvage collection now creates resource state rather than instantly ending the logic.
- [x] Decide when abstract scrap is awarded:
  - [x] on mission return from onboard resource totals

### Phase 5 Notes

* salvage is collected into ship storage if free intake space exists
* recovery can now be blocked by full intake/storage, producing a logistics bottleneck
* mission payout is deferred to the return step and depends on onboard resources

### Phase 5 Definition Of Done

* salvage recovered in the mission can enter the ship’s logistics flow
* the player can see the difference between recovery and final usable output

---

## Phase 6 — First Useful Output

### Goal

Make processed cargo matter to the mission or return loop.

### Tasks

- [x] Decide what the first processed output actually does:
  - [x] create repair charge
- [x] Add one concrete use path for that output.
- [x] Ensure the result is visible enough that the player notices successful processing.
- [x] Connect output consumption or delivery to mission and report state.

### Phase 6 Notes

* repair charge is now consumed by repairs for stronger restoration and better status clearing
* leftover onboard repair charge also improves the return payout
* this makes processing relevant both during the mission and after it

### Phase 6 Definition Of Done

* processing creates something the player actually benefits from
* the logistics loop is not merely cosmetic

---

## Phase 7 — Logistics ARCH Slice

### Goal

Expand ARCH from emergency assistance into simple operational coordination.

### Tasks

- [x] Decide the first logistics ARCH control model:
  - [x] hardcoded preset modes
- [x] Define one automatable logistics behavior:
  - [x] keep the processor fed
  - [x] move processed output back toward storage
- [x] Add a minimal deterministic execution/update path for that behavior.
- [x] Add enough UI to:
  - [x] show the current automation mode
  - [x] show whether it is active
  - [x] show what flow state it is helping with
- [x] Ensure manual intervention still matters when automation exists.

### Phase 7 Notes

* the `computer` now cycles through `Off`, `ReactorGuard`, `LogisticsFlow`, and `Balanced`
* `Balanced` keeps the earlier reactor safety helper while also enabling logistics assistance
* logistics automation prioritizes feeding the processor and returning useful output to storage

### Phase 7 Definition Of Done

* one logistics automation feature works in live gameplay
* the player can feel the difference between manual and assisted flow
* logistics is still not fully hands-off

---

## Phase 8 — Logistics Pressure Pass

### Goal

Tune the scenario so throughput problems actually appear during play.

### Tasks

- [x] Audit the current encounter and identify where logistics can remain irrelevant.
- [x] Adjust the mission so at least one of these reliably happens:
  - [x] intake delay matters
  - [x] storage can fill
  - [x] processor can starve or block
  - [x] damage/power stress can interfere with throughput
- [x] Ensure combat/system pressure can interfere with resource flow without making the slice unreadable.
- [x] Keep the scenario deterministic or strongly guided enough for reliable playtesting.

### Phase 8 Notes

* the sample ship now includes cargo, processor, airlock intake, and computer support
* the encounter already creates system pressure; the logistics loop now has explicit blocked states on top of that
* cargo handling can fail due to full storage, disabled modules, or lack of processor power

### Phase 8 Definition Of Done

* the mission reliably produces at least one logistics problem
* layout and automation choices meaningfully affect throughput

---

## Phase 9 — Return Loop And Editor Feedback

### Goal

Make the editor reflect flow and bottleneck lessons, not only combat outcomes.

### Tasks

- [x] Expand mission report data to record simple logistics outcomes:
  - [x] recovered input amount
  - [x] processed output amount
  - [x] consumed useful output amount
  - [x] transfer count and processor cycles
  - [x] bottleneck summary
  - [x] whether logistics automation was used
- [x] Surface that information in the editor HUD.
- [x] Add lightweight redesign hints around:
  - [x] intake distance / blocked intake
  - [x] storage congestion
  - [x] processor placement / no useful cycle
  - [x] automation coverage
- [x] Ensure the editor remains readable with this added context.

### Phase 9 Notes

* return payout is now based on onboard resource state
* the editor HUD reports throughput, processor output, repair-charge use, and bottleneck hints
* the mission report now carries both systems and logistics lessons back into design time

### Phase 9 Definition Of Done

* the player returns with concrete throughput/layout feedback
* ship redesign is motivated by flow and bottlenecks, not only combat stats

---

## Phase 10 — Usability And Readability Pass

### Goal

Make the logistics slice understandable enough to judge whether it is fun.

### Tasks

- [x] Add or refine on-screen help for:
  - [x] salvage intake
  - [x] storage inspection
  - [x] transfer state
  - [x] processor operation
  - [x] logistics automation mode
- [x] Improve prompt clarity and avoid UI overlap between:
  - [x] mission HUD
  - [x] inspection panel
  - [x] logistics status readouts
  - [x] alerts
- [x] Improve visual distinction for:
  - [x] idle storage
  - [x] active processor state
  - [x] blocked processors
  - [x] stalled logistics
- [x] Add small polish to make the flow readable:
  - [x] progress bars
  - [x] blocked reason text
  - [x] warning clear feedback
- [ ] Do a playtest pass focused on player confusion points and trim complexity where needed.

### Phase 10 Notes

* runtime HUD now reports recovered raw salvage, repair-charge processing, transfers, cycles, and bottlenecks
* inspection panels expose storage and processor state directly
* the remaining open item is a real live tuning pass rather than code scaffolding

### Phase 10 Definition Of Done

* a first-time playtester can understand the cargo-flow loop
* the slice is readable enough to evaluate whether logistics is compelling

---

## Suggested Codebase Expansion

### Near-Term Module Structure

The current implementation keeps logistics in the existing gameplay modules for momentum, but the next clean refactor direction is still:

* `src/client/gameplay/logistics/`
  * `mod.rs`
  * `components.rs`
  * `storage.rs`
  * `transfer.rs`
  * `processing.rs`
  * `automation.rs`
  * `ui.rs`

---

## Priority Order

Implemented in this order:

1. Resource representation
2. Storage behavior
3. Transfer behavior
4. Processor behavior
5. Salvage intake path
6. Useful output hookup
7. Logistics ARCH slice
8. Encounter tuning
9. Return-loop feedback
10. Polish/readability

---

## Definition Of Done For Each Stage

### Stage A Done

* recovered salvage exists as real resource state

### Stage B Done

* storage can hold and expose resource contents

### Stage C Done

* manipulators can move resources with readable progress and bottlenecks

### Stage D Done

* one processor recipe works and creates a real output

### Stage E Done

* salvage enters a ship-internal logistics flow during a live mission

### Stage F Done

* one logistics automation feature works and is useful

### Stage G Done

* encounters can create throughput problems

### Stage H Done

* returning to the editor reflects logistics and layout lessons

---

## Immediate Next Task

The best next implementation task is:

- [ ] Run an interactive playtest/tuning pass on the completed logistics slice, focusing on throughput pacing, repair-charge usefulness, and ARCH mode clarity

That is the bridge between the implemented logistics slice and the next round of tuning or deeper ARCH/logistics expansion.

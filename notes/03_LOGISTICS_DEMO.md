# LOGISTICS_DEMO — Third Vertical Slice

## Goal

Build a **third playable vertical slice** for **LUMEN//ARCH** that proves ships are interesting not only as combat platforms and repair spaces, but as **operational machines** with internal flow, prioritization, and automation.

This milestone should validate:

* logistics components feel materially different from generic “inventory slots”
* the player benefits from designing flow, not only placing strong modules
* storage, manipulation, and processing create meaningful internal routing problems
* ARCH becomes more valuable when it coordinates a real ship process rather than a single emergency reaction
* salvage matters because it feeds onboard handling and conversion, not just a scrap counter
* returning to the editor suggests better layouts, better throughput, and better automation patterns

The first demo proved the outer loop:

`Edit Ship -> Launch -> Fight / Salvage -> Return -> Edit Again`

The second demo proved the inner intervention loop:

`Detect Problem -> Move / Interact -> Stabilize -> Resume Fighting`

This next demo should prove the **operational flow loop**:

`Acquire Resource -> Route It -> Process It -> Use It -> Improve Layout / Automation`

---

## Demo Pitch

The player launches a small utility-combat ship into a controlled encounter with salvage opportunities and limited onboard infrastructure.

The ship can now do more than survive damage:

* collect physical salvage or resource items
* store them in ship modules
* move them between logistics components
* process one resource into another useful form
* automate part of that flow with a small ARCH program or preset behavior

The player should feel the difference between:

* a ship that merely has the right parts
* and a ship where those parts are arranged and automated well

Core loop for the demo:

`Edit Ship -> Launch -> Recover Salvage -> Route / Process Cargo -> Use Output -> Return -> Redesign`

---

## Demo Scope

### In Scope

* first playable logistics components:
  * `LS` storage
  * `LM` manipulator
  * `LP` processor
* tangible runtime items or resource packets
* one simple salvage-to-ship transfer path
* one simple ship-internal transfer path
* one simple processing recipe
* one broader ARCH automation hook that manages a logistics task
* UI for inventories, transfer state, and processor activity
* editor implications for placement and flow
* one encounter that produces enough salvage pressure to make this matter

### Explicitly Out Of Scope

* full generalized logistics simulation
* broad recipe trees
* drones
* full economy layer
* multi-step manufacturing chains
* full ARCH interpreter for arbitrary logistics programming
* faction markets, trade routes, or strategic sector economy
* crew labor simulation
* oxygen-linked logistics hazards

These can come later. The point here is to prove the first satisfying **resource-flow fantasy**.

---

## Player Experience

### Start State

The player opens the editor and now has to care about more than combat survivability:

* where collected salvage enters the ship
* where it is stored
* how far it must move before being useful
* whether the processor and storage layout create congestion
* whether the first ARCH automation slot is helping throughput

### Launch State

The player enters a local encounter with:

* the same small-ship baseline from the systems slice
* one or two salvage sources that drop concrete cargo rather than only awarding scrap instantly
* enough combat or systems pressure that logistics cannot be treated as a detached minigame

### In-Mission Experience

The player should be able to:

* secure or approach salvage
* bring salvage into the ship
* inspect whether it is actually being routed correctly
* see processors consume an input and produce a useful output
* notice when poor placement or poor automation slows the ship down
* intervene manually if routing stalls or a system is starved

### Return State

The player returns to the editor with concrete operational lessons:

* intake is too far from storage
* storage is too far from processing
* the processor is bottlenecked
* automation priorities are wrong
* the ship can fight, but cannot efficiently exploit what it recovers

This is the point where the game should begin to feel like **ship architecture**, not only ship outfitting.

---

## Core Design Questions

The logistics demo should answer these questions:

1. Does cargo flow create interesting ship-design decisions?
2. Do storage, manipulation, and processing feel like different jobs?
3. Is manual intervention still meaningful once logistics exists?
4. Does the next ARCH slice feel like coordination rather than a gimmick?
5. Does salvage become more satisfying when it feeds a ship process?
6. Does the return to the editor create new “throughput and layout” questions?

If the answer to most of these is “no”, deeper logistics and advanced automation later will feel like bookkeeping instead of fantasy.

---

## First Systems To Prove

### 1. Tangible Resource Units

Recovered salvage should become concrete onboard state, not only immediate currency.

Minimum feature set:

* one or two resource kinds
* a runtime representation for carried or stored units
* visibility in storage / processor UI
* persistence through the mission and return flow

Definition of success:

* the player can point to where recovered material currently is

---

### 2. First Logistics Components

The first ARCHLANG logistics family should become playable.

Recommended first components:

* `LS` storage
* `LM` manipulator
* `LP` processor

Minimum behaviors:

* storage can hold resource units
* manipulator can transfer between adjacent or linked modules
* processor can consume one input and produce one output

Definition of success:

* the ship can perform one complete internal resource workflow

---

### 3. First Real Onboard Flow Problem

The player must face a situation where resource location and movement matter.

Good first examples:

* salvage enters near an airlock but the processor is far away
* ammo or repair charge is produced slowly unless routing is efficient
* a useful processor output is unavailable because storage is clogged

Definition of success:

* layout affects operational success, not just aesthetics

---

### 4. Second ARCH Slice

ARCH should graduate from a single safety helper into a small coordination tool.

Recommended first logistics automation behaviors:

* keep the processor fed when input exists
* prioritize moving recovered salvage from intake to storage
* send processed output to one preferred destination
* pause logistics during a power or damage emergency

This still does **not** require the full final scripting model. A preset-driven or constrained-script version is acceptable.

Definition of success:

* the player feels a noticeable difference between manual cargo babysitting and assisted flow

---

### 5. Logistics And Systems Interlock

The slice should connect to the systems demo rather than replace it.

Recommended interactions:

* damaged manipulators slow transfers
* processors add heat or power demand
* a stressed ship sometimes forces the player to choose between fighting, stabilizing, and maintaining throughput
* ARCH may need to prioritize one subsystem over another

Definition of success:

* logistics feels like part of ship operation, not a separate screen

---

### 6. Return-Loop Design Feedback

The editor should now support questions like:

* should intake be closer to cargo?
* should the processor be better protected?
* should the computer sit nearer to the systems it manages?
* is this ship laid out for recovery work or only combat?

Definition of success:

* the player returns wanting to improve workflow, not only firepower

---

## Recommended Runtime Scenario

One good first scenario:

* the player clears a small threat
* one wreck drops raw salvage units
* salvage must enter through an airlock or intake-adjacent location
* onboard logistics can move it to storage
* a processor converts part of it into:
  * repair charge
  * ammo
  * refined scrap
* using or bringing home that output improves the return reward

This scenario is narrow, repeatable, and lets layout mistakes become obvious quickly.

---

## Recommended System Order

Build in this order:

1. Tangible runtime resource representation
2. `LS` storage behavior and inventory UI
3. `LM` manipulator transfer behavior
4. `LP` processor with one recipe
5. Salvage intake path into ship logistics
6. Ship use of processed output
7. Second ARCH slice for logistics coordination
8. Encounter tuning so throughput matters under pressure
9. Return-to-editor reporting for flow and bottlenecks
10. Lightweight polish and readability improvements

This order keeps the slice focused on proving one full ship process before broadening into more recipes or more component families.

---

## Technical Implementation Plan

### Phase 1: Resource Representation

Deliverables:

* one or two resource enums or ids
* storage-friendly runtime quantities
* save/load support
* mission-report support

Suggested result:

* recovered salvage is a real onboard thing, not only an instant score

### Phase 2: Storage Slice

Deliverables:

* storage module runtime state
* capacity rules
* inventory inspection UI
* basic item insertion and extraction

Suggested result:

* the ship can hold resources in named places

### Phase 3: Transfer Slice

Deliverables:

* manipulator runtime logic
* adjacency or link rules
* transfer progress
* blocked / idle / active state visibility

Suggested result:

* resources can move through the ship with readable bottlenecks

### Phase 4: Processing Slice

Deliverables:

* one processor module
* one recipe
* power / heat cost
* input / output buffers

Suggested result:

* the ship can turn recovered material into something operationally useful

### Phase 5: Logistics Automation Slice

Deliverables:

* one ARCH-controlled logistics behavior
* minimal configuration UI
* automation status / priority feedback
* interaction with damage, power, and manual overrides

Suggested result:

* automation reduces a real repetitive task without making the ship autonomous

### Phase 6: Flow-Aware Return Loop

Deliverables:

* mission report with bottleneck / throughput notes
* editor hints about intake, storage, and processing placement
* progression integration for processed outputs or delivered cargo

Suggested result:

* the player returns with concrete layout lessons about ship workflow

---

## Minimal UI Requirements

### Runtime UI

* current storage contents
* active transfer state
* processor status
* blocked / starved / full warnings
* automation mode and whether it is helping
* visibility of where recovered salvage currently sits

### Editor UI

* component costs as before
* identification of logistics components
* last-run notes about bottlenecks
* hints about distance / placement problems if available

Avoid turning this into a giant spreadsheet. The first pass should be readable in motion.

---

## What To Fake Or Simplify

To reach this slice faster, deliberately simplify several systems:

* represent cargo as simple typed units or stacks
* use adjacency or vessel-local links instead of generalized pathfinding
* support exactly one processor recipe at first
* use preset automation behaviors before full scripting
* allow transfers to be abstracted progress bars rather than visible moving items
* keep the economy small and local

If a detail does not improve the feeling of **“this ship has working internal flow”**, simplify it.

---

## Playtest Questions

The demo should answer these questions:

1. Do players understand where cargo is and where it should go?
2. Does a better layout noticeably improve throughput?
3. Does the processor output feel worth caring about?
4. Does the logistics ARCH helper remove busywork without removing decisions?
5. Do combat/system problems meaningfully interfere with onboard flow?
6. Does the player come back wanting to redesign the ship around handling and throughput?

If the answer to most of these is “no”, the logistics fantasy is not landing yet.

---

## Definition Of Done

The logistics demo is done when:

* salvage can become tangible onboard resource state
* storage, manipulation, and processing all exist in a playable first-pass form
* one full internal resource workflow works during a live mission
* one limited logistics automation behavior is useful
* encounter pressure can disrupt or complicate throughput
* the player can complete the mission and return to edit again
* the return loop now teaches layout lessons about resource flow

---

## Recommended Next Doc

After this milestone document, the next useful planning document should be one of:

* `ARCH_LOGISTICS_SLICE.md`
* `RESOURCE_FLOW.md`
* `LOGISTICS_RUNTIME.md`

That follow-up document should define in detail:

* the first runtime resource data model
* how `LS`, `LM`, and `LP` behave in ECS and vessel-local state
* how the next ARCH slice chooses priorities and destinations
* how mission reporting summarizes throughput and bottlenecks

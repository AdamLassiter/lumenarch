# SYSTEMS_DEMO — Second Vertical Slice

## Goal

Build a **second playable vertical slice** for **LUMEN//ARCH** that proves the game remains interesting once ships are more than moving weapon platforms.

This milestone should validate:

* modules feel like distinct systems rather than flat stat blocks
* damage creates **operational pressure**, not just health loss
* the player can manually intervene in meaningful ways
* field-driven hazards make layout and ship state matter
* the first layer of automation adds value without replacing the player
* returning to the editor after a run suggests a path toward deeper ships and deeper automation

The first demo proved the outer loop:

`Edit Ship -> Launch -> Fight / Salvage -> Return -> Edit Again`

This next demo should prove the **inner ship loop**:

`Detect Problem -> Move / Interact -> Stabilize -> Resume Fighting`

---

## Demo Pitch

The player launches a small ship into a controlled encounter. During the mission, systems begin to come under pressure: reactors run hot, damaged components become less reliable, and critical modules may need direct attention.

The player can:

* move around the ship interior
* inspect and manually operate certain components
* perform simple repairs
* experience the first layer of field-based gameplay
* use one basic ARCH computer to automate one limited task
* survive and return to edit again with a clearer sense of ship identity

Core loop for the demo:

`Edit Ship -> Launch -> Fight -> Stabilize Systems -> Salvage -> Return -> Edit Again`

---

## Demo Scope

### In Scope

* player embodiment inside the ship
* walkable ship interior layer or simplified internal movement representation
* manual interaction with selected components
* basic per-module runtime state beyond integrity
* first field systems:
  * heat / cooling
  * electrical interference or instability
* simple repair interaction
* one very limited ARCH automation hook
* component inspection UI
* ship status overview or equivalent high-level system display
* encounter pressures that create reasons to intervene during combat or immediately after

### Explicitly Out Of Scope

* full ARCH interpreter integration across the whole ship
* full LUMEN mechanics
* full logistics pipeline
* drones
* oxygen fail-state simulation
* full radiation gameplay
* boarding combat
* complex crew simulation
* fully dynamic internal pathfinding challenges
* broad crafting economy integration
* late-game automation scaling

These can all come later. The point of this slice is to prove the **manual-to-automation transition**, not complete the whole simulation.

---

## Player Experience

### Start State

The player opens the editor and configures a small ship intended for a slightly more demanding mission.

The editor should imply new concerns:

* where the reactor is placed
* how exposed important systems are
* what the player can reach quickly
* which system gets the first automation support

### Launch State

The player enters a local encounter with:

* one hostile threat or small hostile group
* one salvageable wreck or debris source
* enough combat pressure to create system problems without becoming unreadable

### In-Mission Experience

The player should be able to:

* pilot and fight as before
* notice that specific systems are in trouble
* leave their normal control role to inspect or stabilize systems
* perform a manual repair or reset
* use one small automation helper to reduce workload
* feel the tension between operating the ship and tending the ship

### Return State

The player returns to the editor having learned something concrete about their design:

* the reactor placement was awkward
* the weapon was too exposed
* the automated helper solved one problem but not another
* component layout now matters for both combat and maintenance

This is the moment that should begin to sell the long-term fantasy of growing from operator into architect.

---

## Core Design Questions

The systems demo should answer these questions:

1. Is moving around the ship meaningful rather than decorative?
2. Do component-level failures create interesting decisions?
3. Do field hazards make layout and damage more legible?
4. Is manual intervention tense but manageable during combat?
5. Does the first automation hook feel valuable without trivialising gameplay?
6. Does returning to the editor create new layout questions beyond “more guns, more engines”?

If the answer to most of these is “no”, adding deeper logistics or advanced automation later will not help.

---

## First Systems To Prove

### 1. Player Embodiment

The player must exist **inside** the ship rather than only as the ship.

Minimum embodiment feature set:

* move between walkable interior tiles or interaction points
* interact with nearby components
* receive feedback from local conditions
* have a clear “current station” or “current position” in the ship

This does not need to become a complex character controller immediately. The first version can be simple as long as the player’s location matters.

Why first:

* the whole milestone depends on “being aboard” the ship
* it is the foundation for repairs, manual control, and local field danger

---

### 2. Manual Component Interaction

The player should be able to interact with a few high-value systems directly.

Recommended first interactable components:

* `cockpit` — flight control anchor
* `reactor` — monitor and stabilize output
* `turret` — local manual control or reset
* `shield` — toggle or adjust a simple mode
* `repair point` or damaged component — interact to restore function

Definition of success:

* interacting with the right component at the right time noticeably improves survival

---

### 3. Per-Module Runtime State

Modules need more than integrity to feel like systems.

Recommended first state additions:

* current integrity
* active / disabled state
* heat value
* electrical instability or interference value
* simple “needs attention” state where appropriate

This state should be visible both through inspection and through a high-level status view.

Definition of success:

* damage changes not just whether a module exists, but how it behaves

---

### 4. Basic Field Layer

Only implement a narrow and readable subset of the broader field design.

Recommended first field types:

* `heat` / `cooling`
* `electrical`

These are enough to prove:

* local environmental pressure
* component-specific response
* benefits of thoughtful ship layout
* need for direct player intervention

First-pass interpretations:

* heat damages components and the player when too high
* cooling reduces heat
* electrical interference reduces component reliability or responsiveness

Definition of success:

* players can clearly understand why a local area of the ship is dangerous or unstable

---

### 5. Repair And Stabilization

The player needs one immediate action to perform under pressure.

Possible first repair model:

* move near damaged component
* hold interact to repair
* repair consumes time and maybe a small abstract resource
* repair restores integrity and/or clears a disabled state

Possible first stabilization model:

* interact with reactor to reduce output instability
* interact with turret to clear jam / reset
* interact with shield emitter to restore proper operation

Definition of success:

* the player can meaningfully recover from system pressure instead of only dying once pressure starts

---

### 6. First Automation Hook

This is the first point where ARCH should become playable.

Do **not** implement the full system yet.

Recommended first version:

* one basic computer component
* one short editable script or one small hardcoded script slot
* one controllable target system, such as:
  * reactor output regulation
  * auto-disable turret on overheat
  * basic shield mode switch
  * simple engine cutoff on power deficit

The player should be able to feel:

* “this removed one repeated task from me”
* not:
* “the ship plays itself now”

Definition of success:

* one automated task makes the ship easier to handle, but leaves many others manual

---

### 7. Systems UI And Readability

This slice will fail if players cannot understand what is happening.

Minimum UI requirements:

* current local field readouts on the player
* danger indicators for heat / electrical pressure
* interact prompt when near a usable component
* component inspection panel
* high-level ship status summary

Optional but strongly recommended:

* ship overlay highlighting components in trouble
* simple field visualization when inspecting or debugging

Definition of success:

* a player can tell what is broken, why it is broken, and what they can do about it

---

## Recommended System Order

Build in this order:

1. Player embodiment and movement inside the ship
2. Component interaction framework
3. Per-module runtime state expansion
4. Basic heat and electrical field implementation
5. Local danger UI and component inspection
6. Manual repair / stabilization loop
7. One first-pass automation component and script path
8. Encounter tuning to create pressure for these systems
9. Return-to-editor flow updates that highlight what was learned
10. Lightweight polish and readability improvements

This order keeps the milestone focused on proving the new inner loop before broadening outward again.

---

## Technical Implementation Plan

### Phase 1: Player-On-Ship Representation

Deliverables:

* player entity or equivalent internal avatar
* ship-relative position tracking
* movement between walkable spaces or interaction nodes
* transition between normal ship control and on-foot / internal control mode

Suggested result:

* the player can leave the primary control position and move to another part of the ship during a mission

---

### Phase 2: Interaction Framework

Deliverables:

* interaction radius or adjacency checks
* prompt system for interactable components
* basic interaction dispatch per component type
* support for hold-to-interact where needed

Suggested result:

* standing near a reactor, turret, or damaged module allows a meaningful action

---

### Phase 3: Expanded Module State

Deliverables:

* per-module heat value
* per-module instability / interference state
* disabled or degraded state logic
* runtime updates that derive ship capability from live module state, not only destruction

Suggested result:

* components can remain present but become degraded, forcing decisions before full failure

---

### Phase 4: Basic Field Simulation

Deliverables:

* heat emitters from selected modules
* cooling emitters from selected modules
* electrical interference emitters from selected modules or damage states
* sampling of field values at player and component positions

Suggested result:

* modules and the player respond to nearby heat / electrical conditions in a readable way

---

### Phase 5: Repair And Stabilization Actions

Deliverables:

* repair action
* simple reactor stabilization action
* simple turret or shield reset action
* local danger consideration during interaction

Suggested result:

* staying alive sometimes means stepping away from ideal combat actions to fix or stabilize systems

---

### Phase 6: First ARCH Slice

Deliverables:

* one basic computer component
* one limited script model or predefined script slot
* one target system that can be automated
* minimal script editing or configuration UI

Suggested result:

* the player can offload one repetitive task and immediately feel the difference during play

---

### Phase 7: Encounter Pressure Pass

Deliverables:

* enemy behavior or encounter tuning that produces localized system stress
* damage outcomes that create repairable failures, not only destruction
* salvage or reward flow that still closes the loop cleanly

Suggested result:

* the mission reliably produces one or two “something is going wrong aboard ship” moments

---

### Phase 8: Return Loop Update

Deliverables:

* post-mission report expanded to include system stress outcomes
* editor UI hints showing what modules failed or ran hot
* progression hooks that imply why better layouts and more automation matter

Suggested result:

* returning to the editor feels like solving the ship’s next design problem, not merely spending scrap

---

## Suggested Minimal Controls

Gameplay ship-scale:

* `W` thrust
* `A` / `D` rotate
* `Space` fire
* `Tab` or similar to switch between ship control and internal control

Gameplay internal:

* `WASD` move within ship
* `F` interact
* `Hold F` repair / stabilize if needed
* `Esc` pause
* optional shortcut to return to cockpit

Editor:

* keep existing controls from the first slice

These controls can change later, but a small stable scheme is important for iteration and playtesting.

---

## Minimal UI Requirements

### Runtime UI

* current mission state
* power status as before
* local player field readout:
  * heat
  * electrical danger
* simple component prompt when in range
* inspection panel showing selected component state
* ship alert list or warning banner:
  * reactor hot
  * turret jammed
  * engine unstable
  * module damaged

### Editor UI

* current scrap / progression
* selected module and cost as before
* optional indicators or notes from the last run:
  * hottest module
  * first failed module
  * repair events performed
  * automated system used

Avoid overbuilding a giant systems dashboard unless it directly improves playtest clarity.

---

## What To Fake Or Simplify

To reach this demo faster, deliberately simplify several systems:

* player movement can be node-based or tile-based rather than full collision-rich movement
* only a subset of modules need interaction behavior
* heat can use simple local values rather than full global dissipation
* electrical can be a simple instability score rather than a full network model
* repair can use one abstract resource or even no resource at first
* the first ARCH slice can be a preset script selector instead of a full code editor
* field visualization can be developer-facing first, then player-facing later

If a system does not improve the feeling of “systems under pressure aboard the ship,” simplify it.

---

## Playtest Questions

The demo should answer these questions:

1. Does moving around the ship create real tension during a fight?
2. Do players understand when a module needs attention?
3. Do heat and electrical hazards feel readable and fair?
4. Does one automated helper noticeably reduce cognitive load?
5. Do ship layouts affect not only combat but also onboard maintenance?
6. Does the player come back from a run wanting to redesign around operational problems?

If the answer to most of these is “no”, the game is not yet proving its deeper identity.

---

## Definition Of Done

The systems demo is done when:

* the player can move within the ship during a mission
* the player can interact with at least a few critical components
* modules have runtime state beyond simple health
* heat and electrical pressure can affect modules and/or the player
* the player can manually repair or stabilize something under pressure
* one limited ARCH automation hook exists and is useful
* an encounter can create these problems in a repeatable, readable way
* the player can complete the mission and return to edit again
* the editor loop now implies deeper system design, not just hull expansion or extra weapons

---

## Recommended Next Doc

After this milestone document, the next useful planning document should be one of:

* `ARCH_SLICE.md`
* `SHIP_INTERIOR.md`
* `RUNTIME_SYSTEMS.md`

That follow-up document should define in detail:

* how the player is represented aboard the ship
* what modules expose interactable runtime state
* how the first playable ARCH automation path is implemented
* how fields are sampled and surfaced in the UI

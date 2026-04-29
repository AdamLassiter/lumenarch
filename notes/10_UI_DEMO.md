# UI_DEMO — Tenth Vertical Slice

## Goal

Build a dedicated UI-focused vertical slice for **LUMEN//ARCH** that proves the game can present its growing simulation clearly, stylishly, and interactively.

This slice should validate:

* the general in-sector UI is readable during combat, EVA, boarding, logistics, and atmosphere pressure
* component interaction is no longer primarily text-driven
* major stations open **full-screen or near-full-screen interactive panels**
* the player can understand and manipulate complex systems without reading dense walls of text
* the first **basic ARCH editor** can exist as a real UI tool rather than only presets and text summaries

This slice should prove the loop:

`Observe Situation -> Open Panel -> Adjust Graphical Controls -> Read Immediate Feedback -> Return To Play`

## Why This Slice Next

The current project already proves:

* ship editing and refit
* hostile encounters and travel
* onboard movement and boarding
* component interaction
* ARCH execution
* logistics and atmosphere

But the prototype still presents too much of that through verbose text blocks and overlapping status panes. The concept depends on the player becoming an operator and later an architect, and that requires interfaces that feel like operating machinery, not reading logs.

Before expanding the underlying mechanics further, the game needs a UI layer that can carry them cleanly.

## Demo Pitch

The player launches into a normal sector encounter. Instead of relying on dense stacked text for most actions, they use dedicated graphical panels:

* a cleaner general HUD during sector play
* a helm UI with visible throttle and steering state
* a turret UI with aim, fire, and lead cues
* a reactor UI with live graphs or gauges
* logistics panels showing inventories, routes, and processing visually
* a basic ARCH editor with line list, register targets, and validation feedback

The player should feel that every component is becoming a machine to operate, not only a tile with a text description.

Core loop for the demo:

`Launch -> Read Situation Quickly -> Open Station UI -> Adjust Controls -> Resume Play`

## In Scope

* improved sector/encounter HUD layout
* better spacing and hierarchy for alerts, status, cargo, and mission information
* reduction of overlapping panels and redundant text
* full-screen or near-full-screen station panels for:
  * cockpit / helm
  * turret
  * reactor
  * logistics
  * computer
* graphical controls such as:
  * sliders
  * toggles
  * dials
  * bars
  * directional indicators
  * inventory lists with icons and flow hints
* a basic ARCH editor:
  * line-based editing
  * opcode selection
  * register selection
  * constant editing
  * validation / parse feedback
* interaction flow that cleanly enters and exits these panels

## Explicitly Out Of Scope

* complete final art direction for every panel
* full freeform code-editor ergonomics
* full LUMEN editor
* all future component families
* localization
* final menu polish for all screens

## Core Design Rules

### 1. UI should make the simulation easier to act on

The UI must reduce cognitive load, not just restate raw numbers in prettier boxes.

### 2. Component panels should feel mechanically distinct

Reactor controls should not resemble logistics controls. Helm should not resemble ARCH editing. Shared scaffolding is fine, but interaction models should reflect the machinery.

### 3. General HUD should support active play

The player should be able to fly, board, fight, and monitor danger without the HUD becoming a wall of text.

### 4. ARCH editing should become real enough to be iterated in-game

This slice should not complete ARCH, but it should stop relying on only templates and hidden abstractions.

## First Systems To Prove

### 1. General Sector HUD Refresh

Deliver:

* clearer top-level status grouping
* less overlap
* less text duplication
* readable play-state separation between:
  * flight
  * EVA
  * boarding
  * station use

Definition of success:

* the player can understand the current situation at a glance

### 2. Helm UI

Deliver:

* throttle indicator
* turn/heading indicator
* optional steering wheel / helm visualization
* visible relationship between desired and current steering state

Definition of success:

* piloting feels like using a station, not only pushing keys

### 3. Turret UI

Deliver:

* local aim display
* desired vs actual turret angle feedback
* cooldown / weapon state
* target/lead cues where helpful

Definition of success:

* turret control becomes readable and trainable

### 4. Reactor UI

Deliver:

* reaction rate control
* turbine load control
* live readouts for:
  * heat
  * output
  * fuel
  * stability

Definition of success:

* the reactor becomes something the player can actually reason about visually

### 5. Logistics Panels

Deliver:

* inventory slots or structured lists
* transfer direction / source / destination clarity
* processor recipe state
* airlock/storage/fabrication relationship visibility

Definition of success:

* onboard flow is understandable without reading raw counters alone

### 6. Basic ARCH Editor

Deliver:

* line list for instructions
* add/remove/reorder flow
* opcode and operand editing
* parse / validation results
* basic saveable authored programs in a real UI

Definition of success:

* the player can meaningfully inspect and alter ship automation in-engine

## Relation To The Concept

This slice does not add entirely new game rules, but it is critical to delivering the concept because:

* the player’s role as operator depends on readable interfaces
* the later architect role depends on real programming and systems UI
* the distinction between direct control and guided autonomy needs presentation as much as mechanics

Without this slice, the game’s later complexity will remain harder to understand than to simulate.

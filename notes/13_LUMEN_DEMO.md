# LUMEN_DEMO — Thirteenth Vertical Slice

## Goal

Build the first full **late-automation language slice** for **LUMEN//ARCH** by polishing and expanding the programmable control stack:

* complete the ARCH instruction set and editor experience
* introduce proper LUMEN syntax and execution semantics
* make the distinction between direct control and optimization-layer control fully playable

This slice should validate:

* ARCH feels complete enough to support real authored control logic
* LUMEN feels like a distinct language layer rather than “ARCH but stranger”
* both systems can be edited through a polished in-game programming UI
* the player can evolve from manual operator to true systems architect

This slice should prove the loop:

`Inspect Ship Logic -> Edit ARCH / LUMEN Program -> Launch -> Audit Behavior -> Iterate`

## Why This Slice Later

LUMEN should not come before:

* strong UI support
* richer component variety
* a mature deterministic simulation base

By the time this slice is implemented, the project should already have:

* real station UIs
* diverse components worth automating
* hostile ships, boarding, atmosphere, and logistics pressure
* stronger deterministic guarantees

That is the right context for completing the higher-level programming layer.

## Demo Pitch

The player equips ships with both conventional ARCH computers and more advanced LUMEN-capable systems. In the editor, they can author proper programs using a stronger interface than the current template-driven flow. During missions, they can inspect what those programs are doing and redesign around the results.

ARCH remains the layer for:

* direct deterministic control
* register reads and writes
* explicit component commands

LUMEN becomes the layer for:

* optimization-oriented directives
* broader target selection
* controlled biasing of component behavior through a defined syntax

Core loop for the demo:

`Refit -> Edit Programs -> Launch -> Observe Control + Optimization -> Debug -> Rewrite`

## In Scope

* completion pass on the ARCH instruction set and editor flow
* polished in-game authoring UI for ARCH
* proper LUMEN language syntax
* target selection rules for LUMEN instructions
* runtime explainability for both ARCH and LUMEN
* save/load integration for richer authored programs
* at least one scenario where LUMEN is clearly useful but not interchangeable with ARCH

## Explicitly Out Of Scope

* final late-game content scale
* huge LUMEN standard library
* procedural code generation
* natural-language programming

## ARCH Direction For This Slice

This slice should treat ARCH as the mature direct-control language:

* assembly-style syntax
* register reads from current state
* writes to next-state command surfaces
* deterministic bounded execution
* proper editor support for:
  * line insertion/removal
  * operand editing
  * validation
  * error display
  * program naming
  * copying / templating / saving

The ARCH editor should feel like a real shipboard engineering tool.

## LUMEN Direction For This Slice

LUMEN should use a proper syntax centered on optimization instructions rather than direct component control.

### High-Level Language Shape

LUMEN instructions should read more like:

* target selection
* optimization application
* weighted preference or suppression

Examples of the intended style:

* `BUFF target heat_cooling weight`
* `NERF target power_draw weight`
* `BUFF target shield_strength weight`
* `NERF target instability weight`

The important point is that LUMEN should not directly say:

* set throttle to 0.8
* set reaction rate to 0.3

That remains ARCH territory.

Instead, LUMEN should target readable and writable system properties by **declared target register groups** and **optimization intents**.

### Register Targeting

LUMEN should operate on targetable register spaces such as:

* component family groups
* nearby components
* specific addressed modules
* filtered sets such as:
  * all turrets
  * nearby processors
  * current shield emitters
  * reactors within range

The language should define how targets are resolved, and those targets should be inspectable in UI.

### Runtime Semantics

LUMEN should:

* remain deterministic
* apply bounded optimization effects
* avoid becoming direct imperative control
* visibly complement ARCH rather than replace it

## Core Design Rules

### 1. ARCH and LUMEN must remain distinct

ARCH:

* direct
* explicit
* low-level
* command-oriented

LUMEN:

* indirect
* biasing / optimization-oriented
* field/range/system-aware
* higher-level

### 2. Editing experience matters as much as runtime

This slice should not only expand interpreter logic. It must also make programming legible and practical in-game.

### 3. Debuggability is mandatory

The player should be able to answer:

* what did ARCH write?
* what did LUMEN target?
* why did the ship behave that way?

## Relation To The Concept

This slice most directly fulfills the concept’s stated core:

> the shift from direct control to guided autonomy

By this point:

* the player should already understand manual systems
* ARCH should let them formalize direct machine control
* LUMEN should let them move beyond command into optimization and architecture

That is the clearest expression of the game’s intended long-term identity.

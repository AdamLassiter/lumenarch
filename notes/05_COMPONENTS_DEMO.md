# COMPONENTS_DEMO — Fifth Vertical Slice

## Goal

Build a **component interaction slice** for **LUMEN//ARCH** that proves ship systems are not only abstract statuses, but real stations with manual controls, distinct viewpoints, and shared command surfaces that later automation can target.

This milestone should validate:

* every major component feels like a place and an interface, not only a tile
* manual interaction is meaningfully different between component families
* the onboard player view feels ship-local by default
* cockpit, turret, reactor, and logistics interaction become readable, tactile jobs
* component-local UIs write into the same effective control surfaces that future ARCH logic will write into
* earlier slices are corrected where they were still using simplified or abstract interaction shortcuts

This slice should prove the **station interaction loop**:

`Move To Station -> Inspect State -> Adjust Controls -> Observe Response -> Reposition Or Handover`

## Why This Slice Next

The current project already proved:

* external flight and combat
* internal movement and intervention
* first logistics flow

But component interaction is still too abstract in several places:

* cockpit flight is not yet a true station UI
* turret operation is not yet a real local control mode
* reactor control is not yet exposed as a proper manual system
* logistics panels are still more status-like than operational

Before 04 ARCH becomes a real scripting slice, the game needs a stable rule:

> manual component operation and autonomous component operation must target the same control surfaces

This milestone establishes those surfaces.

## Demo Pitch

The player launches a ship and moves through its interior in a ship-local view. Each major component can be operated through a dedicated station interface:

* cockpit for helm control
* turret for aiming and firing
* reactor for reaction and turbine management
* logistics modules for transfer, storage, and processing oversight

The player should feel the difference between:

* being somewhere aboard the ship
* merely watching abstract HUD numbers

Core loop for the demo:

`Launch -> Walk To Station -> Operate Component -> Observe World / Ship Response -> Switch Stations`

## Scope

### In Scope

* ship-local onboard camera by default when not at a flight or turret station
* higher zoom for interior mode
* cockpit station with world-view flight control
* turret station with turret-local aiming and firing control
* reactor station with dedicated reactor simulation controls and readouts
* logistics station panels for:
  * storage
  * manipulators
  * processors
* component family UI scaffolding so subtypes can share common interaction patterns
* a command-surface model so manual interaction effectively writes component commands or writable registers
* inspection and interaction flow corrections for earlier slices

### Explicitly Out Of Scope

* full ARCH scripting runtime
* full LUMEN optimization layer
* complete content coverage for every future component subtype
* polished late-game station art or animation for every module
* fully continuous crew animation or inverse-kinematics interaction

This slice is about proving **component-local interaction grammar**, not finishing all content.

## Design Rule

Each interactable component should have:

* readable state
* writable controls
* a dedicated local UI or shared family UI
* a consistent mapping into deterministic runtime commands

Future ARCH should target those same writable controls.

## Camera And View Rules

### Default Interior View

When the player is moving around inside the ship:

* the camera is rotated so the ship is “up” on screen regardless of world rotation
* the camera is zoomed in relative to cockpit / flight view
* UI emphasizes nearby station and local hazards

This should become the default onboard presentation mode.

### Cockpit View

The cockpit should:

* switch to the current ship flight / world camera
* preserve the current external flight framing
* expose semi-diagetic helm feedback such as:
  * throttle position
  * steering / helm rotation
* support both keyboard and mouse control

Definition of success:

* the cockpit feels like piloting the ship rather than just toggling “flight mode”

### Turret View

The turret should:

* use the cockpit / flight zoom level
* rotate around the turret hardpoint’s local frame
* show actual vs desired turret angle
* support both mouse and keyboard control
* make firing feel like working a mounted system, not only pressing global fire

Definition of success:

* the player can clearly distinguish hull/world orientation from turret-local control

## Component Families To Prove

### 1. Cockpit

Manual UI should provide:

* throttle demand
* steering demand
* clear readout of actual movement state
* helm / throttle semi-diagetic cues

This should write the same effective flight demand state that future automation would write.

### 2. Turret

Manual UI should provide:

* desired aim angle
* fire intent
* actual vs desired angle indicator
* readouts for readiness, heat, or cooldown

This should write the same effective turret demand state that future automation would write.

### 3. Reactor

Manual UI should provide:

* reaction rate control
* turbine load control
* readouts for:
  * internal heat
  * power output
  * fuel consumption
  * stability if modeled separately

The simulation target is explicitly:

* higher reaction rate increases internal heat and fuel consumption
* higher turbine load reduces internal heat while increasing power output

Definition of success:

* reactor operation becomes an understandable balancing act rather than a generic “stabilize” action

### 4. Logistics

Logistics should expose family-specific panels:

* storage:
  * contents breakdown
  * fill state
* manipulator:
  * transfer source / target / held load
  * manual transfer commands or routing choice
* processor:
  * recipe list
  * current progress
  * input / output state

Definition of success:

* logistics feels like operating equipment, not reading a static inventory tooltip

### 5. Future-Proof Family Scaffolding

Even if not fully implemented yet, the slice should define clear expectations for:

* shields:
  * integrity
  * directional control where applicable
* detectors:
  * information-layer toggles
  * initial reuse of debug-view layers is acceptable
* drone stations:
  * current drone task selection
* memory banks:
  * stored data inspection

The point is not to fully ship these now, but to ensure the interaction architecture can support them cleanly.

## Corrections To Earlier Slices

This slice should explicitly correct simplifications from earlier milestones:

* interior movement currently exists, but the default onboard presentation should become ship-local rather than world-aligned
* turret use should stop being only a ship-level fire action and become a true station option
* reactor interaction should stop being only a generic stabilize action
* logistics inspection should become operational control where appropriate
* earlier “mode toggles” should begin collapsing into component-local control surfaces

## First Systems To Prove

### 1. Station Interaction Framework

Build a common framework for:

* station entry / exit
* component focus
* family-specific control panels
* shared command-write semantics

Definition of success:

* new station types can be added without rebuilding the whole interaction stack

### 2. Ship-Local Interior Camera

Build the default onboard view:

* ship-up orientation
* tighter zoom
* stable transition into and out of station views

Definition of success:

* walking the ship feels fundamentally different from flying the ship

### 3. Cockpit Station

Deliver:

* world-view flight camera
* helm and throttle controls
* keyboard and mouse input paths
* semi-diagetic cockpit feedback

Definition of success:

* cockpit becomes the canonical manual flight interface

### 4. Turret Station

Deliver:

* hardpoint-local aim control
* actual vs desired angle display
* keyboard and mouse aim paths
* fire control from station

Definition of success:

* turret operation is understandable as its own role

### 5. Reactor Station

Deliver:

* reactor simulation controls
* heat / power / fuel readouts
* explicit reaction-vs-turbine tradeoff

Definition of success:

* reactor operation feels like real system management

### 6. Logistics Stations

Deliver:

* storage contents panel
* manipulator control panel
* processor recipe and status panel

Definition of success:

* logistics modules gain meaningful manual operation

### 7. Shared Command Surface Model

Deliver:

* a deterministic runtime command layer for component-local writable state
* manual station UIs writing into that layer
* later ARCH compatibility by design

Definition of success:

* manual control paths are ready to become writable registers for 04

## Recommended Technical Shape

Use a layered structure:

* component runtime state
* component command state
* station UI/controller layer
* camera/view layer
* simulation consumption of command state

This should avoid coupling raw input directly to simulation side effects.

## UI Direction

Each component family should likely have:

* a common shell:
  * title
  * health / status
  * warnings
* a family-specific control cluster
* a compact local readout panel

Examples:

* reactors share one panel pattern across fission / fusion variants
* logistics modules share family layouts but different controls
* cockpit and turret are strongly bespoke

## Playtest Questions

This slice should answer:

1. Does onboard movement now feel like moving through ship-relative space?
2. Do cockpit and turret operation feel distinct enough?
3. Is reactor control understandable and interesting?
4. Does logistics gain a stronger manual identity?
5. Does the command-surface model feel like a strong base for ARCH?
6. Do the component UIs make the ship feel more like a machine than a tile map?

## Definition Of Done

The component demo is successful when:

* the default onboard view is ship-local and zoomed-in
* cockpit and turret have distinct playable station interfaces
* reactor has a real manual control surface and simulation response
* logistics modules expose meaningful manual control panels
* component-local controls effectively map into shared writable command surfaces
* the project is now structurally ready for 04 ARCH to automate those same surfaces

At that point, the game will have a much stronger bridge between:

* being physically aboard the ship
* manually operating its systems
* later handing those systems over to authored automation

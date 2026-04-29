# BREACH_DEMO — Ninth Vertical Slice

## Goal

Build the first atmosphere and decompression slice for **LUMEN//ARCH** that proves ships are enclosed machines with meaningful internal environmental state, not just traversable containers.

This slice should validate:

* ships have real sealed or venting interior volumes
* oxygen is simulated per tile inside ships
* hull breaches and airlock state can change those volumes
* the player can suffer or exploit decompression during boarding
* ship layout and stabilization start to matter in terms of compartment integrity, not only module health

This slice should prove the loop:

`Fight / Board -> Breach Or Open -> Vent / Stabilize -> Exploit Or Recover -> Extract -> Return`

## Why This Slice Next

The current implemented slices already prove:

* station-gated travel and relaunch
* modular ship editing
* component-local manual control
* authored ARCH runtime
* logistics flow
* real hostile ships
* EVA and boarding
* physical cargo extraction

But the original concept still lacks one of its most important environmental promises:

> ships are enclosed, fragile spaces where oxygen and exposure matter

Right now the prototype still lacks:

* per-tile oxygen inside ships
* meaningful sealed vs venting interior state
* decompression as a boarding and survival tool
* hull breaches that affect traversal pressure beyond simple damage
* architectural consequences for compartment shape and airlock placement

This slice should be the first proof that ship interiors are not just geometry, but environments.

## Chosen Direction

Decisions already made:

* oxygen should be simulated per tile within ships
* this slice focuses on oxygen and decompression, not a full fluid or chemistry sim
* boarding and hostile ships should be affected by the same atmosphere rules as the player ship
* airlocks and breaches should be first-class atmosphere boundaries
* the slice should stay systemic, not scripted around bespoke “decompression objective” moments

## Demo Pitch

The player launches into a standard encounter, damages or boards a hostile ship, and can now influence more than systems and cargo. Opening an airlock, damaging a hull edge, or failing to seal their own compartments has a real effect on onboard atmosphere.

Hostile ships can be weakened by being vented. The player’s own ship can become dangerous if left breached or poorly compartmentalized. Boarding is no longer only about reaching storage or controls, but about surviving and shaping the environment inside the ship.

Core loop for the demo:

`Board -> Breach / Seal -> Manage Oxygen -> Exploit Environment -> Return`

## In Scope

* per-tile oxygen on ship interiors
* detection of sealed vs venting spaces
* airlock-open and breach-driven venting
* first-pass player oxygen hazard state
* atmosphere interaction on both player and hostile ships
* hull breach state that feeds the atmosphere sim
* HUD/readouts for local oxygen and compartment stability
* mission/reporting consequences from exposure or venting

## Explicitly Out Of Scope

* full gas chemistry
* high-fidelity fluid simulation
* fire propagation
* detailed pressure-wave physics
* medical gameplay
* full NPC suffocation combat
* life-support production chains as a main feature of this slice

## Core Design Rules

### 1. Atmosphere is ship-local and tile-based

Oxygen should be stored and updated per interior tile, not only per room label or per module.

### 2. Compartments matter more than raw volume

The important question is not just “how much ship exists,” but “what is sealed together.” Layout and airlock placement should start to matter.

### 3. Hostile ships follow the same rules

The player should be able to exploit hostile venting because hostile ships are real ships, not special cases.

### 4. Breaches should create tactical pressure without requiring full realism

This slice should create meaningful gameplay consequences from decompression without attempting a full simulation-heavy survival game.

### 5. Atmosphere should reinforce boarding, not replace it

The purpose of this slice is to deepen boarding and ship stabilization, not to turn every encounter into passive environmental waiting.

## First Systems To Prove

### 1. Ship Interior Oxygen Grid

Deliver:

* oxygen amount per interior tile
* ship-local tile occupancy and enclosure state
* stable oxygen in sealed areas

Definition of success:

* ships have a meaningful internal atmosphere model

### 2. Breach And Airlock Venting

Deliver:

* tiles can vent to space
* airlock state affects venting
* breached hull sections affect nearby atmosphere

Definition of success:

* a ship can meaningfully lose atmosphere through real openings

### 3. Player Exposure Feedback

Deliver:

* player reads local oxygen
* low oxygen creates danger or performance pressure
* EVA and onboard low-atmosphere states are clearly distinguished

Definition of success:

* the player can tell when a boarded space is safe or dangerous

### 4. Hostile Atmosphere Exploitation

Deliver:

* hostile ships can be vented
* boarding route decisions can depend on atmosphere state
* decompression can become part of disabling or softening a hostile ship

Definition of success:

* atmosphere is a systemic tactical lever, not only a self-penalty

### 5. Return-Loop Consequences

Deliver:

* mission report reflects venting/stabilization outcomes
* redesign hints can point to bad compartmenting or poor airlock access
* hostile extraction can be affected by how the player managed breaches

Definition of success:

* atmosphere management matters after the encounter, not only during it

## Recommended Technical Shape

### Runtime Model

Expand encounter runtime with:

* ship-local atmosphere tile state
* enclosure / breach boundary state
* venting update systems
* player local oxygen sampling

### Ship Data

Each runtime ship should expose:

* interior walkable tiles
* solid / sealing boundaries
* airlock tiles or interfaces
* breached exterior openings

### Atmosphere Simulation

First-pass oxygen simulation should remain intentionally simple:

* oxygen quantity per tile
* equalization within connected compartments
* leak loss through exposed boundary tiles
* optional refill only from seeded starting atmosphere for this slice

### Player State

Add or revise runtime player data to include:

* local oxygen reading
* exposure danger timer or penalty state
* clear distinction between EVA vacuum and shipboard low-oxygen hazard

## Playtest Questions

* does decompression make ships feel more like real spaces?
* does the player understand when and why a compartment is venting?
* does boarding become more interesting when atmosphere matters?
* do breaches create tactical choices rather than only punishment?
* does ship layout start to imply better or worse compartment design?
* does this feel like a natural extension of the concept’s stabilization fantasy?

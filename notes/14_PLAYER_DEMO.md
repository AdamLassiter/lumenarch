# PLAYER_DEMO — Fourteenth Vertical Slice

## Goal

Build the first full **player embodiment and survival** slice for **LUMEN//ARCH** by turning the player into a more complete operational actor inside the expedition loop.

This slice should expand the earlier recovery/salvage direction into a broader **PLAYER_DEMO** focused on:

* the player’s tolerance to environmental hazards
* suit-based protection and specialization
* physical carrying of items and salvage
* repair and extraction work performed by the player body

This slice should validate:

* the player is meaningfully distinct from the ship
* moving through the ship and nearby space matters mechanically
* recovery, repair, and salvage become embodied activities
* suit choice changes how expeditions are approached
* the game’s core loop is not only about ship systems, but about what the player can personally survive and accomplish

## Candidate Next Slices

Before finalizing the next slice, there are three strong directions suggested by the concept and current codebase.

### Option 1 — Recovery / Salvage Slice

Focus:

* mission aftermath
* salvage hauling
* resource processing
* persistent expedition outcomes

Pros:

* directly strengthens the concept’s core loop
* builds naturally on existing logistics and mission-report systems
* gives automation more to do outside combat

Risks:

* can become too ship-centric
* does not by itself make the player body more meaningful

### Option 2 — Player Embodiment / Survival Slice

Focus:

* environmental tolerance
* manual repair and extraction
* suits and equipment
* carried items and personal mobility

Pros:

* directly supports the concept section on player embodiment
* makes ship interiors and EVA space more meaningful
* creates stronger manual-to-automation contrast
* turns recovery and repair into things the player actually does, not just things the ship reports

Risks:

* touches many systems at once
* needs care so it does not become fiddly survival micromanagement

### Option 3 — Faction / Encounter Identity Slice

Focus:

* stronger enemy identity
* faction-specific threats and rewards
* more distinct encounter patterns

Pros:

* helps the fiction a lot
* makes the world feel less generic
* gives more context to hostile ships and salvage

Risks:

* depends on stronger player and recovery systems first
* improves encounter flavor more than the underlying game loop

## Final Choice

The strongest next slice is:

## **Player Embodiment / Survival**, framed as **PLAYER_DEMO**

This is effectively a widened version of the earlier recovery/salvage idea.

Why this wins now:

* The concept explicitly says the player exists physically within the ship and performs manual actions.
* The project already has strong ship/system direction, but the player body is still too light relative to the game’s identity.
* Recovery and salvage become much more interesting once they are constrained by:
  * what the player can survive
  * what the player can carry
  * what the player can repair
  * what equipment the player chose to bring
* This slice cleanly supports the larger thesis:
  * the game begins with manual embodied work, and only later shifts toward broader orchestration

This makes the next slice not just “more salvage,” but “salvage, repair, and recovery as player work.”

## Demo Pitch

The player launches into a dangerous zone and must personally endure environmental hazards, move through damaged spaces, carry valuable material, repair critical modules, and sometimes step outside the ship in specialized gear to recover components or stabilize the mission.

The core loop for the slice becomes:

`Launch -> Endure -> Repair / Recover -> Carry / Extract -> Return -> Refit`

The player should feel that:

* some situations can be solved from a console
* some require physically going there
* some require the right suit
* some are only worth attempting if the player can survive long enough to do the work

## Why This Slice Fits The Concept

This slice most directly fulfills the “Player Embodiment” and “Core Gameplay Loop” sections of [docs/src/CONCEPT.md](/home/adaml/code/lumenarch/docs/src/CONCEPT.md).

It reinforces:

* manual repair
* manual salvage
* physical movement through ship interiors
* the importance of oxygen, heat, and danger
* the progression from engineer to operator to architect

It also gives stronger meaning to the world fiction:

* this is a graveyard of functioning machinery
* the player is not just commanding systems abstractly
* they are entering broken spaces and extracting value by hand

## Core Experience This Slice Should Prove

The player should be able to feel all of these in one expedition:

* “I can’t stay in this compartment long without protection.”
* “I need the welder suit to repair this section or strip parts from that wreck.”
* “I can grab the scrap, but I can only carry one thing at a time.”
* “The EVA suit lets me reach drifting salvage or hostile wreckage faster.”
* “The radiation suit lets me work in hazardous conditions the default suit can’t handle.”
* “Automation helped the ship survive, but I still had to go do the dangerous part myself.”

## In Scope

* player tolerance to environmental elements:
  * heat
  * electrical pressure
  * oxygen deprivation
* suit equipment model
* manually carried single-item state
* manual ship repair gameplay
* manual hostile-ship component extraction gameplay
* EVA movement differentiation through equipment
* recovery and salvage flow that uses the player body rather than only abstract payouts
* updated UI/readouts for player condition, equipped suit, and carried item

## Explicitly Out Of Scope

* final inventory complexity
* large RPG-style equipment trees
* broad crafting/economy systems
* full boarding-combat slice
* deep medical/injury simulation

## Equipment Direction

This slice should introduce a small, clear equipment model centered on utility and survival rather than loot complexity.

### Baseline Player State

The player should have:

* a default unspecialized state
* one equipped suit slot
* one carried item slot

The carried item slot can hold things like:

* scrap
* recovered material
* an unequipped suit
* an extracted component

This should remain simple and legible. The point is not inventory Tetris. The point is that the player can only do one significant carrying task at a time.

### Suit Types

The first suit set should be:

#### Radiation Suit

Purpose:

* improved tolerance to environmental hazards

Primary gameplay effects:

* protects against dangerous heat/electrical/low-oxygen exposure better than default gear
* allows longer work in compromised compartments or hazardous recovery spaces

#### Welder Suit

Purpose:

* manual repair and extraction specialist

Primary gameplay effects:

* required to repair damaged ship components
* required to extract recoverable components from hostile ships or wrecks

#### EVA Suit

Purpose:

* improved mobility in exterior / vacuum operations

Primary gameplay effects:

* increased EVA flight speed
* better ability to reach drifting salvage, distant breaches, or hostile wreckage quickly

## Player Hazard Model

The player should become meaningfully vulnerable to the same simulation pressures that already matter to ships and modules.

### Hazard Sources

Minimum supported hazards for this slice:

* heat exposure
* electrical pressure exposure
* oxygen deprivation

### Design Rules

* hazards should be readable and not hidden
* exposure should create pressure before instant failure
* suits should mitigate, not trivialize, danger
* player survival should influence what recovery tasks are practical

The important outcome is that the player’s route through the ship and nearby space becomes a real tactical decision.

## Recovery / Salvage Integration

This slice intentionally keeps the earlier recovery/salvage direction inside it.

Recovery should now be understood as:

* something the ship systems support
* something the player often performs physically
* something constrained by suits, survival, and carrying limits

Examples:

* carrying scrap from a wreck back to a storage module
* wearing the welder suit to strip a turret or reactor component from a hostile ship
* switching to the EVA suit to reach drifting salvage before abandoning the area
* using the radiation suit to traverse overheated or electrically dangerous sections of the player ship

## Manual Repair Direction

Repair should become a more explicit manual player action.

Target behaviors:

* damaged modules can be repaired by the player in person
* repair should require the proper suit
* repair should be shaped by environmental safety and access
* automated systems can support recovery, but should not remove the need for manual intervention early on

This is a strong expression of the concept’s early-game “Engineer” role.

## Component Extraction Direction

Hostile ships and wrecks should offer more than abstract salvage value.

Target behaviors:

* some modules can be extracted as meaningful recoverables
* extraction should require proximity and the welder suit
* extraction should compete with time, danger, and carrying limits
* extraction opportunities should help differentiate “clean victory” from “valuable recovery”

This is the clearest bridge between combat outcomes and player agency.

## Carrying Model

The player should be able to carry one thing at a time.

This should shape decisions such as:

* take scrap now or bring back an unequipped suit
* carry repair charge to a critical module or haul salvage to storage
* extract a component from a hostile ship or prioritize survival and retreat

The model should stay intentionally narrow:

* one carried object
* clear object identity
* visible dropoff / deposit behavior

## UI / Feedback Needs

This slice will need clearer player-facing information than currently exists.

Important displays:

* equipped suit
* carried item
* current hazard exposure
* whether the player is allowed to repair/extract a target
* whether a task is blocked by lacking the correct suit

The player should not have to guess:

* why they are taking damage
* why a repair action is unavailable
* why extraction failed

## Likely System Areas To Change

Likely touched areas:

* `src/gameplay/components/actors.rs`
* `src/gameplay/components/interactions.rs`
* `src/gameplay/components/logistics.rs`
* `src/gameplay/components/simulation.rs`
* `src/gameplay/helpers/interactions.rs`
* `src/gameplay/systems/control/`
* `src/gameplay/systems/interactions/`
* `src/gameplay/systems/simulation/atmosphere.rs`
* `src/gameplay/systems/simulation/fields.rs`
* `src/gameplay/systems/simulation/mission.rs`
* `src/gameplay/systems/ui/`
* `src/gameplay/spawn/scene/`
* `src/gameplay/spawn/ship/`
* `src/ship/mod.rs`
* `src/state/ui.rs`

Likely new or expanded concepts:

* equipped suit component/resource
* carried item component/resource
* player hazard tolerance state
* extractable hostile component state
* repair/extraction interaction variants
* EVA movement modifiers

## Design Rules For This Slice

### 1. Player embodiment must matter

The player should feel like a real entity with constraints, not only a camera focus.

### 2. Suits should change capability, not just numbers

Each suit should unlock or strongly alter what kinds of work the player can safely do.

### 3. Carrying limits should create decisions

The carrying model should be simple but meaningful.

### 4. Recovery should remain physical

Value should come from embodied action in dangerous spaces, not only from end-of-mission arithmetic.

### 5. Automation should complement player agency

ARCH and LUMEN should reduce burden, but the player must still be the one doing the dangerous, localized work in this slice.

## Slice Success Criteria

This slice succeeds if a player can finish an expedition and say:

* “I had to switch suits to finish the job.”
* “I could reach the wreck faster in EVA gear, but I needed the welder suit to extract anything useful.”
* “I brought back less salvage because I could only carry one thing at a time.”
* “That overheated compartment was survivable in one suit and dangerous in another.”
* “The ship’s automation kept systems stable, but I still had to physically go repair and recover.”

That is the clearest sign that **LUMEN//ARCH** is becoming a game about embodied systems work, not just remote control of a spaceship.

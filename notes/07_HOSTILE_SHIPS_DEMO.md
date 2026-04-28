# HOSTILE_SHIPS_DEMO — Seventh Vertical Slice

## Goal

Build the first enemy-ship combat slice for **LUMEN//ARCH**, replacing the current stationary turret-platform opposition with real hostile vessels.

This slice should validate:

* enemy encounters are ship-versus-ship rather than ship-versus-emplacement
* hostile targets can be composed from the same modular language as player ships
* enemy defeat creates more meaningful salvage and damage stories
* the current travel, systems, logistics, and ARCH foundations all become more relevant in combat

This slice should prove the loop:

`Dock -> Choose Node -> Fight Enemy Ships -> Disable / Destroy -> Recover Outcome -> Return`

## Why This Slice Next

The current prototype already proves:

* docked refit and sector travel selection
* a parameterized encounter runtime
* onboard manual station interaction
* logistics and processor flow
* early ARCH automation

But combat opposition is still too primitive. Static hostile turrets are useful as a harness, but they do not yet prove the actual game fantasy of assembling, damaging, disabling, and salvaging ships.

This slice should be the first proof that enemy vessels are part of the same game language as the player vessel.

## Chosen Direction

Decisions for this slice:

* focus on **hostile ship encounters**
* keep the current travel/docked loop and plug hostile ships into it
* use a **debug enemy ship editor** to author enemy layouts early
* persist authored enemy ships to `saves/enemy_ships.json`
* keep enemy AI intentionally simple in the first pass

## Demo Pitch

The player launches from station into a route node and encounters one or more hostile ships built from real ship modules. These hostile ships maneuver, aim, fire, take module damage, and eventually break apart into meaningful wreck outcomes.

The player should be able to tell:

* what kind of enemy ship they are facing
* where its vulnerable modules are
* whether it is still mobile, still dangerous, or effectively disabled
* what kind of salvage result they created

Core loop for the slice:

`Dock -> Choose Node -> Engage Hostile Ship -> Disable Key Systems -> Survive -> Return`

## In Scope

* hostile ship runtime entities built from modular ship definitions
* at least two enemy ship archetypes
* simple enemy control/AI
* enemy movement, aiming, and firing
* enemy module damage and disablement
* enemy defeat and wreck outcome
* encounter selection by enemy ship composition rather than only turret count
* debug enemy ship editor entry from the menu
* enemy ship persistence to `saves/enemy_ships.json`

## Explicitly Out Of Scope

* full faction simulation
* advanced enemy boarding or drones
* full symmetric AI using the entire player station model
* complete enemy ARCH programming
* procedural enemy ship generation
* polished final enemy ship UI/art presentation

## Core Design Rules

### 1. Enemy ships should use the same ship language

Enemy vessels should be represented as modular ship definitions rather than custom one-off enemy prefabs.

### 2. Defeat should mean more than deleting a target

Enemy ships should move through meaningful states:

* operational
* degraded
* disabled
* destroyed or abandoned wreck

### 3. AI should stay simple before it gets smart

The first pass should prioritize readable behavior:

* approach or hold range
* face the player
* fire when aligned
* stop performing well when engines / turrets / reactors are damaged

### 4. Enemy authoring should be cheap

We should be able to author and tweak enemy layouts quickly through a debug editor rather than hardcoding every hostile ship in Rust.

## First Systems To Prove

### 1. Shared Enemy Ship Definitions

Deliver:

* enemy ship definitions stored outside code
* load/save path at `saves/enemy_ships.json`
* at least two authored hostile ships

Definition of success:

* enemy ship content can be iterated without recompiling layout constants

### 2. Debug Enemy Ship Editor

Deliver:

* a debug menu item from the main menu or equivalent debug access path
* enemy ship selection / editing flow
* save back to `saves/enemy_ships.json`

Definition of success:

* enemy layouts can be authored in-engine using the same tile language as the player ship

### 3. Hostile Ship Runtime Spawning

Deliver:

* hostile ship root entities
* hostile runtime modules
* hostile movement / weapons / power approximations

Definition of success:

* encounters contain real ships rather than isolated turret points

### 4. Enemy Combat Readability

Deliver:

* visible hostile ship silhouettes
* visible turret/fire direction
* damage / disable feedback on hostile modules

Definition of success:

* the player can read which parts of an enemy ship are still dangerous

### 5. Node-Driven Encounter Composition

Deliver:

* route nodes reference enemy ship loadouts
* different node kinds can spawn different hostile ship mixes

Definition of success:

* node selection feels meaningfully different because enemy compositions differ

## Recommended Technical Shape

### Data

Add enemy content data alongside player ship persistence:

* `EnemyShipLibrary`
* `EnemyShipEntry`
* persisted at `saves/enemy_ships.json`

Each entry should include:

* display name
* ship definition
* threat tier
* optional behavior tag

### Runtime

Add hostile ship ECS types parallel to the player ship runtime:

* hostile ship root marker
* hostile ship movement / power / weapon state
* hostile allegiance / target selection state
* hostile ship AI state

### Encounter Composition

`EncounterSpec` should evolve to include hostile ship loadouts, for example:

* list of enemy library ids
* spawn offsets
* behavior tags

### Debug Authoring

The menu should include a debug path to enter an enemy-ship authoring flow.

This can begin as a thin variation of the current refit editor:

* choose an enemy entry
* edit its module layout
* save the library JSON

It does not need a full separate UX language yet, but it should be clearly marked as debug-only.

## Save Model

Persist:

* existing campaign save state as before
* enemy ship library separately in `saves/enemy_ships.json`

The enemy library should not be embedded into the campaign save; it should behave more like editable content than run-state progression.

## Playtest Questions

* does fighting real ships immediately feel more like the intended game?
* can the player identify enemy weak points visually?
* do different enemy ship layouts create distinct encounter texture?
* does the hostile-ship slice make current systems feel more justified?
* is the debug authoring path fast enough to support iteration?

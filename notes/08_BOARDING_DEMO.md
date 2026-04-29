# BOARDING_DEMO — Eighth Vertical Slice

## Goal

Build the first boarding and EVA slice for **LUMEN//ARCH** that proves encounters can continue beyond ship-to-ship gunfire and resolve through direct physical interaction with hostile vessels.

This slice should validate:

* the player can move continuously through world space rather than hopping tile-to-tile
* ships project an inertia field that converts the player between world-relative EVA and ship-relative onboard motion
* enemy ships appear in normal sector encounters, not just debug or authored edge cases
* hostile ships can be boarded and interacted with using the same station/component logic as the player ship
* cargo can be physically carried between space, hostile ships, and the player ship

This slice should prove the loop:

`Travel -> Engage Enemy Ship -> Disable / Approach -> EVA / Board -> Interact / Extract -> Return`

## Why This Slice Next

The current implemented slices already prove:

* station-gated travel and relaunch
* modular player ships
* real hostile ships
* onboard systems pressure
* logistics flow
* authored ARCH runtime
* component-local station interaction

But the original concept still lacks one of its most distinctive promises:

> the player physically inhabits and manipulates ships as spaces, not just as abstract combat entities

Right now the prototype still lacks:

* continuous personal motion
* EVA and ship-relative transition
* boarding in normal play
* direct reuse of enemy systems as spaces the player can enter
* physical carried cargo rather than only abstract salvage transfer

This slice should be the first proof that ships are traversable machines in the same world as the player.

## Chosen Direction

Decisions already made:

* player movement becomes continuous in this slice
* ship inertia fields define onboard vs EVA movement/camera behavior
* enemy ships should spawn in actual sector nodes
* enemy ships should be treated like player ships, not bespoke objective containers
* cargo should be physically carried by the player between ships and space

## Demo Pitch

The player launches into a normal sector encounter and faces one or more enemy ships rather than only turret points. Once an enemy ship is disabled or safely approached, the player can leave the cockpit, move continuously through their own ship, cross into EVA, drift or thrust toward the hostile ship, enter its inertia field, and board it.

Once aboard, there are no special “boarding objective” buttons. The enemy ship is simply another ship:

* its reactor can be managed or shut down through existing reactor controls
* its computers can be disabled or their ARCH state manipulated
* its cargo can be inspected and physically removed
* its logistics chain can be exploited or interrupted

The player can carry useful items back through space and deposit them into their own ship for return.

Core loop for the demo:

`Fight -> Board -> Interfere / Steal -> Extract -> Return`

## In Scope

* continuous player movement
* EVA state and world-relative player motion
* ship inertia fields and automatic frame-of-reference switching
* boarding-capable hostile ships in normal sector encounters
* enemy ships reusing the same station/component interaction rules as player ships
* carrying physical cargo by hand
* transferring physical cargo from enemy ship or space to player ship storage
* return-loop consequences based on what was actually extracted or altered

## Explicitly Out Of Scope

* humanoid enemy crew combat
* full ranged personal weapons
* ship capture / ownership transfer
* faction diplomacy around boarding
* decompression / oxygen simulation as the main feature of the slice
* multi-character boarding squads
* full stealth / alarm simulation

## Core Design Rules

### 1. The player is a continuous world actor

The player should no longer hop between discrete nodes for motion. They should have position, velocity, and collision in continuous space, even if many interactions still snap to nearby stations.

### 2. Inertia fields define onboard traversal

Each ship should project an inertia field around itself.

Inside that field:

* the player’s motion is interpreted relative to the ship
* the player camera rotates with the ship
* moving around feels like being aboard the vessel

Outside that field:

* the player is in world-relative EVA motion
* the camera is world-relative
* movement is not attached to any ship frame

### 3. Enemy ships should behave like real ships

Boarding should not introduce bespoke one-off objective widgets. Enemy ships should expose the same kinds of systems the player ship already does.

### 4. Cargo should exist physically

At least the first boarding rewards should come from physically carried cargo or cargo-like salvage bundles rather than only abstract mission payout.

### 5. Sector encounters must actually use ship boarding

This slice should update real sector nodes so boarding opportunities occur in standard travel flow, not only in a debug harness.

## First Systems To Prove

### 1. Continuous Player Locomotion

Deliver:

* continuous local/world player transform
* movement velocity and damping
* basic collision against ship geometry or walkable bounds

Definition of success:

* the player no longer moves tile-by-tile

### 2. EVA And Inertia Field Transition

Deliver:

* inertia field around ships
* automatic enter/exit detection
* world-relative EVA mode
* ship-relative onboard mode
* camera flips appropriately between those frames

Definition of success:

* moving between ship interior and open space feels like crossing a real spatial boundary

### 3. Enemy Ships In Real Sectors

Deliver:

* real sector nodes that spawn hostile ships
* reduced dependence on point-turret-only encounters
* tuning that makes hostile ships survivable and boardable

Definition of success:

* the player can encounter boardable ships through ordinary route selection

### 4. Boarding Via Existing Systems

Deliver:

* enemy interior entry
* interaction with hostile reactors, computers, storage, logistics, and turrets
* no bespoke objective-only actor design

Definition of success:

* boarding feels like reusing the game’s systemic rules rather than switching genres

### 5. Physical Cargo Carry Loop

Deliver:

* carried item state on the player
* pickups in space or aboard ships
* depositing carried cargo into storage on the player ship

Definition of success:

* loot extraction becomes a physical action, not just a mission result number

### 6. Return Consequences

Deliver:

* mission/reporting updates based on cargo brought home or systems sabotaged
* better reward when the player actually extracts value

Definition of success:

* boarding outcomes matter in the post-mission loop

## Recommended Technical Shape

### App / Runtime Model

Keep the current travel and encounter structure, but expand the encounter runtime with:

* continuous player actor state
* ship inertia field components
* carried item components/resources
* boarding-aware hostile ship composition

### Player Runtime State

Add or revise player runtime data to include:

* continuous world position
* velocity
* current reference frame:
  * `World`
  * `ShipLocal(player ship)`
  * `ShipLocal(hostile ship)`
* optional carried item payload

### Ship Runtime State

Each ship should expose:

* inertia field radius or shape
* traversable interior bounds
* airlock or boarding-relevant access points
* runtime components already used by the player ship

### Boarding Rules

The player should board by physically entering the hostile ship’s field and reaching accessible interior space, not by abstract “start boarding” UI flow.

### Cargo Model

First-pass physical cargo can be deliberately simple:

* one or two carryable resource bundle types
* player can carry one bundle at a time
* bundles can exist:
  * in space
  * in hostile storage
  * in player hands
  * in player storage

## Playtest Questions

* does continuous movement immediately improve the feel of ship interiors and EVA?
* does the inertia field transition read clearly without too much UI explanation?
* do enemy ships feel like spaces rather than targets?
* does boarding create naturally emergent goals without explicit objective markers?
* does carrying cargo back by hand make extraction more memorable than abstract salvage?
* do travel encounters feel richer once some can be resolved by boarding rather than only destruction?

# TRAVEL_DEMO — Sixth Vertical Slice

## Goal

Build the first **outer-loop travel slice** for **LUMEN//ARCH** that proves the game is more than a single encounter sandbox.

This slice should validate:

* the player chooses where to go next from a readable route structure
* stations become the intended safe hubs for refit and relaunch
* encounters are selected from a persistent travel layer rather than always launched directly
* mission outcomes feed back into a continuing route loop

This slice should prove the loop:

`Dock At Station -> Review Ship And Results -> Choose Next Node -> Jump -> Resolve Encounter -> Return -> Refit`

## Why This Slice Next

The current implemented slices already prove:

* ship editing and launch
* encounter combat/salvage loop
* onboard systems pressure
* logistics flow
* authored ARCH
* station-style component operation

But the original concept still has one major unproven promise:

> `Travel -> Engage/Explore -> Salvage -> Stabilize -> Repeat`

Right now the prototype still lacks:

* a real route-selection layer
* stations as the intended safe anchor
* persistent zone choice and completion state
* a reason to compare one encounter against another before launching

This slice should be the first proof that the game has a real campaign-shaped loop.

## Chosen Direction

Decisions already made:

* primary focus: **zone selection**
* editing rule: **station-gated**
* map shape: **small node graph**
* no misjump / in-transit event focus yet
* no full barter economy yet

## Demo Pitch

The player starts docked at a hub station with their current patchwork vessel. From the station they can review their latest mission, open the sector map, inspect a handful of nearby route nodes, and choose where to jump next.

Different nodes advertise different risk/reward profiles:

* safer salvage-rich fields
* more dangerous hostile holds
* unstable derelicts with harsher systems pressure

After the encounter, the player returns to the station, keeps the consequences, refits if desired, and chooses the next destination.

Core loop for the demo:

`Dock -> Inspect Route -> Choose Risk -> Launch -> Return With Outcome -> Refit -> Relaunch`

## In Scope

* `Docked` hub flow
* `SectorMap` route-selection flow
* small persistent node graph
* encounter launch from selected nodes
* station-gated refit/editor access
* persistent node completion / exhaustion state
* persistent scrap/progression return flow
* encounter parameterization by node type
* simple station services if helpful:
  * review report
  * refit/edit ship
  * optional repair-all-for-scrap

## Explicitly Out Of Scope

* full economy or barter system
* full station NPC life
* misjumps and in-transit events
* large procedural galaxy
* faction diplomacy
* full strategic layer
* LUMEN mechanics
* drones as the focus of the slice

## Core Design Rules

#### 1. Stations become the intended safe hub

Ship refit/editing should be a **docked activity**, not a generally available action from any point in the loop.

#### 2. Travel choice must be visible before launch

The player should be able to compare nodes by:

* type
* approximate risk
* approximate reward
* connectivity / reachability

#### 3. Encounters remain the existing strong core

This slice should reuse the existing runtime encounter foundation, but parameterize it per node rather than building a second gameplay mode.

#### 4. Outcomes must persist

Choosing a node must matter after the fight:

* node state changes
* rewards persist
* ship state persists
* the route graph remains coherent on return

## First Systems To Prove

#### 1. Docked Hub Flow

Deliver:

* station hub screen
* scrap total and latest mission summary
* access to refit/editor
* access to route selection

Definition of success:

* the player has a clear “home state” between encounters

#### 2. Small Route Graph

Deliver:

* one hub node
* 4–6 connected encounter nodes
* visible node type/risk/reward hints
* reachable-node highlighting

Definition of success:

* choosing the next encounter feels intentional rather than arbitrary

#### 3. Encounter Selection By Node

Deliver:

* `EncounterSpec` selected from a route node
* node-specific enemy/salvage/hazard flavor
* runtime launch from selected node

Definition of success:

* encounters feel like destinations, not one reusable undifferentiated arena

#### 4. Persistent Sector Progress

Deliver:

* node completion/exhaustion state
* current location
* saved seed and node graph
* return-to-station outcome application

Definition of success:

* the player can leave and come back to a persistent local route space

#### 5. Station-Gated Refit

Deliver:

* editor accessible from `Docked`
* editor exit returns to `Docked`
* player-facing flow no longer depends on “return to editor from anywhere”

Definition of success:

* refit reads as a station activity rather than a debug utility

## Recommended Technical Shape

#### App States

Expand the client flow into:

* `Menu`
* `Docked`
* `SectorMap`
* `Editing`
* `Encounter`

Use `Encounter` for the current gameplay runtime that is now parameterized by selected node data.

#### Persistent Resources

Add a persistent campaign/session layer:

* `SectorState`
* `DockedState`
* expanded progression resource for scrap plus route state

#### Route Model

First-pass route model:

* `SectorNode`
* `SectorNodeKind`
* `EncounterSpec`
* `TravelOutcome`

Recommended node kinds for this slice:

* `HubStation`
* `SalvageField`
* `HostileHold`
* `UnstableDerelict`

#### Encounter Handoff

Selected route node should produce an `EncounterSpec` that configures:

* hostile count/loadout
* salvage amount
* hazard flavor
* arena dressing
* reward multiplier hint

#### Save Model

Persist together:

* ship definition
* scrap/progression
* sector graph seed
* resolved node states
* current docked node
* last mission report

### Playtest Questions

* does the player understand the difference between docked, route selection, and encounter play?
* do node choices feel meaningfully different before launch?
* does returning to station make the game feel more like a campaign loop?
* does station-gated refit feel natural rather than restrictive?
* does the route graph make the next choice feel interesting after one or two runs?

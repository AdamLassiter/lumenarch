# PLAYABLE_DEMO — First Vertical Slice

## Goal

Build a **small but complete playable loop** for **LUMEN//ARCH** that proves the game is fun before we invest heavily in late-game systems, deep automation, or broad multiplayer support.

The first demo should validate:

* ship editing is meaningful
* ships are readable and controllable in play
* modules have clear gameplay roles
* combat creates pressure
* salvage creates reward
* returning to the editor creates a satisfying loop

## Demo Pitch

The player edits a small starter ship, launches into a single combat/salvage scenario, manually pilots and fights, scavenges a wreck, survives, and returns to edit again.

Core loop for the demo:

`Edit Ship -> Launch -> Fight / Salvage -> Return -> Edit Again`

## Demo Scope

### In Scope

* ship editor with place/remove/rotate tools
* one playable ship loaded from the editor
* one local encounter scene
* manual movement
* manual firing
* basic module health/damage
* a small salvage interaction
* a simple return-to-editor flow
* minimal UI for ship status

### Explicitly Out Of Scope

* full galaxy map
* travel between multiple sectors
* full ARCH interpreter gameplay integration
* full LUMEN mechanics
* full logistics simulation
* oxygen simulation as a fail state
* advanced heat chains
* drones
* boarding combat
* multiplayer gameplay
* persistence beyond simple local ship save/load unless needed

These can all come later. The point of this slice is to prove the game loop, not complete the design.

## Player Experience

### Start State

The player opens the ship editor and sees a small starter vessel.

They can:

* add or remove core modules
* rotate directional modules
* make simple tradeoffs in layout

### Launch State

The player starts a local encounter with that ship.

The encounter contains:

* one hostile target
* one salvageable wreck or debris cluster
* enough space to move and fight clearly

### In-Mission Goals

The player should be able to:

* pilot the ship
* manage one or two critical systems manually
* defeat or avoid a hostile threat
* collect salvage
* survive long enough to extract

### Return State

The player returns to the editor with salvage rewards and can improve the ship.

Even if the reward system is light at first, the loop should imply progression.

## First Demo Systems

### 1. Ship Data Backbone

The editor and runtime must share one canonical ship model.

This model should contain:

* ship name
* module list
* module positions
* module rotations
* module type ids

Optional for first slice:

* lightweight per-module runtime state

Why first:

* the playable slice depends on edited ships being usable immediately in runtime
* this reduces rework between editor and gameplay

### 2. Ship Spawn Pipeline

The edited ship must spawn into a play scene from saved or in-memory editor data.

This includes:

* creating module entities from ship data
* creating derived ship runtime state
* calculating movement-relevant aggregates
* placing visuals correctly

Definition of success:

* the ship seen in the editor matches the ship used in gameplay

### 3. Manual Flight

Manual control should come before automation.

Minimum movement feature set:

* forward thrust
* rotation left/right
* inertial drift or at least weighty motion
* camera follow

This is where the game starts to feel real.

### 4. Minimal Ship Systems

Only implement the smallest set needed to make combat and layout decisions matter:

* `reactor`: generates power
* `battery`: buffers power
* `engine`: contributes thrust
* `turret`: fires projectiles
* `hull` / `hull_corner`: structural body
* `core`: mission-critical ship center
* `cockpit`: control anchor
* `cargo`: salvage capacity hook
* `airlock`: salvage interaction point if needed

For the first demo, each module should have a clear runtime role even if simplified.

### 5. Combat

Combat should be small and readable.

Minimum combat systems:

* hostile target AI
* projectile spawning
* projectile hits
* module damage
* ship destruction or retreat condition

Enemy scope:

* one small drone ship or turret platform

The enemy does not need sophisticated AI. It only needs to create real pressure.

### 6. Salvage

The first salvage interaction can be simple.

Possible first version:

* after the enemy is disabled or a wreck is approached, player presses interact near salvage
* salvage becomes a resource reward or a recoverable module reward

This should be enough to imply the future progression loop without requiring full logistics.

### 7. Return And Iterate

After the encounter:

* the player returns to the editor
* rewards are applied
* the ship can be changed
* another run can begin

This is the moment that proves the game’s meta-loop.

## Recommended System Order

Build in this order:

1. Canonical ship data model shared by editor and runtime
2. Save/load or handoff path from editor ship to runtime ship
3. Runtime scene that spawns the edited ship
4. Manual movement and camera follow
5. Reactor/battery/engine power flow
6. Turret firing and basic projectile hits
7. Module damage and ship failure rules
8. One hostile encounter
9. Salvage interaction and reward
10. Return-to-editor flow
11. Lightweight polish and readability UI

This order keeps the critical path focused on player feel.

## Technical Implementation Plan

### Phase 1: Shared Ship Runtime

Deliverables:

* shared ship model type
* conversion from editor state to runtime state
* spawn system for ship modules in combat scene

Suggested result:

* pressing a launch button instantiates the exact edited ship in a playable test arena

### Phase 2: Core Movement And Power

Deliverables:

* ship thrust/turn controls
* basic power generation and draw
* movement affected by installed engines

Suggested result:

* editing engine placement changes how the ship feels in play

### Phase 3: Weapons And Damage

Deliverables:

* one player weapon
* one enemy weapon
* damage routed to modules
* ship death / disable state

Suggested result:

* losing modules changes combat outcomes

### Phase 4: Encounter And Salvage

Deliverables:

* one encounter scene
* one hostile target
* one salvage source
* interaction to collect rewards

Suggested result:

* the player completes a full mission loop

### Phase 5: Return Flow And Progression Stub

Deliverables:

* end-of-encounter summary
* rewards applied to editor state
* ship can be changed for the next run

Suggested result:

* repeated runs make sense and imply longer-term progression

## Suggested Minimal Controls

Editor:

* left click place
* right click remove
* `Q` / `E` rotate
* toolbox selection
* launch button

Gameplay:

* `W` thrust
* `A` / `D` rotate
* mouse aim or facing-based firing
* left click or `Space` to fire
* `F` interact / salvage
* `Esc` pause or exit

These can change, but a tiny stable control set helps playtesting.

## Minimal UI Requirements

The first demo only needs enough UI to make the state understandable.

Editor UI:

* toolbox
* selected part + rotation
* launch button

Gameplay UI:

* current hull/integrity
* power status
* maybe weapon cooldown
* mission status or extraction prompt

Avoid overbuilding diagnostic UI beyond what helps playtesting.

## What To Fake Or Simplify

To reach the demo faster, it is good to deliberately fake several systems:

* salvage can be a direct reward instead of a full transport pipeline
* power can be simplified to total supply vs total demand
* weapon targeting can be straightforward
* enemy AI can be state-machine simple
* return flow can be an instant scene swap
* rewards can be abstract “scrap” without a full economy

If a system does not directly improve the first 10 minutes of play, simplify it.

## Playtest Questions

The demo should answer these questions:

1. Is building or changing the ship immediately interesting?
2. Does manual control feel readable and satisfying?
3. Do different module layouts change decisions in a noticeable way?
4. Is combat tense without being chaotic?
5. Does salvage feel worth the risk?
6. Does returning to the editor make the player want “one more run”?

If the answer to most of these is “no”, more systems will not fix the problem.

## Definition Of Done

The first playable demo is done when:

* the player can edit a ship layout
* that exact layout is used in gameplay
* the ship can move and fight
* at least one hostile threat can damage or destroy the ship
* at least one salvage interaction exists
* the player can finish the mission and return to edit again
* module choices meaningfully affect success
* the loop can be repeated without developer intervention

## Recommended Next Doc

After this milestone document, the next useful planning document should be:

* `RUNTIME_SLICE.md` or similar

That document should define:

* the shared ship data model
* how editor state becomes runtime state
* the exact Bevy entities/resources/systems for the first playable mission

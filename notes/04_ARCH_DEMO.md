# ARCH_DEMO — Fourth Vertical Slice

## Goal

Build the first **playable authored automation slice** for **LUMEN//ARCH**, but do it on top of a real register-backed component interaction model rather than inventing separate “automation-only” control paths.

This milestone should validate:

* ARCH feels like a real playable system rather than a preset toggle
* authored logic writes to the same command surfaces the player uses manually
* scripting changes ship behavior under pressure in a readable way
* the player benefits from designing control logic, not only ship layout
* the same ship can transition between:
  * direct manual operation
  * assisted operation
  * guided autonomy

The first three slices proved:

* `Edit Ship -> Launch -> Fight / Salvage -> Return -> Edit Again`
* `Detect Problem -> Move / Interact -> Stabilize -> Resume Fighting`
* `Acquire Resource -> Route It -> Process It -> Use It -> Improve Layout / Automation`

The next slice to implement is [05_COMPONENTS_DEMO.md](/home/adaml/code/lumenarch/notes/05_COMPONENTS_DEMO.md), because it provides the missing manual-control surfaces that ARCH must write into.

Once that exists, this slice should prove the **autonomy loop**:

`Observe State -> Write Control Logic -> Let Ship Act -> Audit Result -> Rewrite`

## Why This Slice Still Matters

Starting from [docs/src/CONCEPT.md](/home/adaml/code/lumenarch/docs/src/CONCEPT.md), the most distinctive promise of the game is still:

> the shift from direct control to guided autonomy

But after the latest planning correction, that shift should be implemented as:

* manual UI writes component command registers
* ARCH writes those same component command registers
* the simulation consumes those staged writes deterministically

That keeps the game coherent and avoids building separate manual and autonomous control architectures.

## Dependency On 05 Components Demo

This slice now explicitly depends on [05_COMPONENTS_DEMO.md](/home/adaml/code/lumenarch/notes/05_COMPONENTS_DEMO.md).

That slice should establish:

* ship-local onboard view by default
* cockpit and turret control stations
* reactor, logistics, and future-component panels
* per-component manual UIs that effectively write command registers
* consistent semantics between:
  * player input
  * station UI
  * staged writes
  * runtime simulation

ARCH should not bypass those surfaces.

## Revised Demo Pitch

The player launches a small working ship with one or two real computer modules. During the mission, they can manually operate components through dedicated station UIs, or rely on authored ARCH programs that write into the same underlying control surfaces.

The player should feel the difference between:

* a ship with no control architecture
* a ship with only manual operation
* a ship with well-authored assistance logic

Core loop for the demo:

`Edit Ship -> Configure Stations / ARCH -> Launch -> Observe Behavior -> Adjust Script -> Relaunch`

## Revised Scope

### In Scope

* real ARCH program representation
* bounded deterministic script execution each fixed tick
* register read/write model for a narrow curated set of systems
* one or two computer modules hosting programs
* script editor or constrained program editor in the editor flow
* runtime ARCH status UI
* authored automation targets built on top of the manual control surfaces from 05:
  * reactor support
  * logistics routing support
  * turret or combat-support behavior
* mission scenarios that reveal the difference between poor and good automation
* return-to-editor reporting about script effectiveness

### Explicitly Out Of Scope

* full final ARCH instruction set
* arbitrary user-authored labels and full assembly IDE polish
* LUMEN optimization layer
* broad dynamic component UI coverage beyond what 05 establishes
* strategic sector map
* drone swarms

## Key Architectural Rule

ARCH should not directly “do gameplay.”

Instead:

* components expose readable state through registers
* components expose writable command registers
* manual station UIs manipulate those writable registers
* ARCH programs manipulate those same writable registers
* the simulation resolves command intent into actual component behavior

This rule should hold for:

* cockpit flight demand
* turret desired angle / fire enable
* reactor reaction rate and turbine load
* logistics priorities and recipes

## First Systems To Prove

### 1. Narrow But Real ARCH Runtime

Implement the smallest real version of ARCH that still feels authentic:

* deterministic fixed-tick execution
* bounded instruction budget
* current-state reads
* next-state writes
* no unbounded loops
* explicit failure / invalid-op reporting

This can be a subset of [docs/src/ARCHLANG.md](/home/adaml/code/lumenarch/docs/src/ARCHLANG.md), not the full final language.

### 2. Register Surface Based On Manual Stations

The first writable register set should mirror the first manual station UIs:

* cockpit demand registers
* turret desired angle / fire intent
* reactor control registers
* logistics mode / routing / recipe controls

Definition of success:

* if the player can set it manually at a station, ARCH can write the same control path

### 3. Computer Modules As Real Hosts

Computers should:

* host one program
* expose execution budget / speed
* report execution state
* be damageable and operationally important

### 4. Script Authoring Workflow

Recommended first version:

* constrained line-list editor
* starter templates:
  * `ReactorGuard`
  * `LogisticsFeed`
  * `TurretAssist`
  * `BalancedOps`
* editable constants and enabled lines

### 5. Runtime Explainability

Minimum runtime explainability:

* current program name
* current writes
* last successful branch or action
* blocked reason
* over-budget / invalid instruction state

### 6. Encounter Designed For Automation Value

The mission must create workload broad enough that authored automation is valuable, but still readable.

A good first scenario:

* intermittent hostile turret pressure
* reactor or electrical spikes
* post-fight salvage routing pressure
* moments where the player chooses whether to stay at a station or trust automation

## Updated Design Question

The most important question for this slice is now:

* does authored ARCH feel like an extension of the component-control model, rather than a detached scripting minigame?

If not, the architecture is wrong even if the interpreter works.

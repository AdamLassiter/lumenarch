# BETTER_COMPONENTS_DEMO — Eleventh Vertical Slice

## Goal

Expand the component roster of **LUMEN//ARCH** from single baseline examples into richer families with meaningful mechanical variation.

This slice should validate:

* component **types** and **variants** create real design choices
* shipbuilding becomes about selecting different mechanical behaviors, not only adding more of the same tiles
* automation becomes more interesting because different components demand different control strategies
* the ship editor and runtime both support a broader but still coherent component language

This slice should prove the loop:

`Choose Variant -> Build Around Constraints -> Operate / Automate -> Learn Tradeoffs -> Refit`

## Why This Slice Next

The current prototype proves that components exist and can be operated, but many families still only have one representative behavior. The concept promises evolving patchwork vessels built from scavenged machinery, and that fantasy needs meaningful hardware diversity.

This slice should focus on making each major component family more expressive and mechanically differentiated.

## Demo Pitch

The player can now build ships using multiple variants within each major system family. Some are simpler, cheaper, and easier to automate. Others are stronger, faster, or more specialized, but demand new logistics, new control strategies, or better ARCH.

The player should feel that they are no longer merely placing “a reactor” or “a turret,” but choosing between different machine philosophies with different operational consequences.

Core loop for the demo:

`Refit -> Choose Better Parts -> Launch -> Experience New Behaviors -> Adjust Layout / Automation -> Return`

## In Scope

* component variants for major families
* differing costs, strengths, and automation requirements
* runtime behaviors that materially differ between variants
* editor support for selecting and placing these variants
* UI updates for new controls/readouts
* logistics expansion where required by new component rules

## Explicitly Out Of Scope

* full final balance across every component
* massive tech tree or research system
* final salvage rarity/economy integration
* every possible future component subtype

## Component Family Expansions

### 1. Turrets

Deliver:

* **Hitscan laser turret**
  * base option
  * simpler to automate
  * instant hit
  * lower per-shot impact
* **Projectile ballistic turret**
  * more expensive upgrade
  * harder to automate
  * stronger damage
  * requires ammunition
  * ammunition must be stored nearby in ammo storage / suitable logistics containers

Validation:

* turret choice affects both combat feel and automation complexity

### 2. Reactors

Deliver:

* **Fission reactor**
  * base option
  * relatively simple control behavior
  * automatable with simple PID-style logic
* **Fusion reactor**
  * more advanced option
  * more interdependent control variables
  * stronger payoff
  * still automatable, but demands better logic
* fuel is separated from the reactor itself and stored in neighboring fuel storage

Validation:

* reactor choice changes both logistics and control problems

### 3. Cockpits / Helms

Deliver:

* improved cockpit / helm variants that offer superior control quality
* better handling, visibility, or control authority depending on implementation direction

Validation:

* helm quality matters as more than a cosmetic replacement

### 4. Batteries / Capacitors

Deliver:

* batteries with:
  * total capacity
  * maximum charge rate
* **capacitors**
  * lower total capacity
  * much higher charge/discharge rate

Validation:

* energy storage becomes about response profile as well as total reserve

### 5. Cores

Deliver:

* better cores with larger maximum supported ship size
* larger inertia shields
* clearer relationship between core quality and vessel scale

Validation:

* the core becomes a meaningful architectural limiter and upgrade path

### 6. Fabricators

Deliver:

* slower and faster fabricator variants
* ability to turn scrap into:
  * ammunition
  * fuel

Validation:

* logistics and salvage can feed richer onboard production loops

### 7. Shields

Deliver:

* **radial shield generator**
  * simpler
  * broader protection
  * lower peak effectiveness
* **directional shield generator**
  * stronger
  * more focused
  * intended to require meaningful automation

Validation:

* shielding becomes a control/optimization problem, not only a passive buffer

## Core Design Rules

### 1. Variants must differ mechanically, not only numerically

Each variant should change:

* how the player lays out the ship
* how the player interacts with the station
* how ARCH would automate it
* what kinds of supporting logistics are needed

### 2. Better parts should create new burdens

Upgrades should not only be “strictly stronger.” They should often demand:

* better fuel routing
* better ammunition routing
* more precise control
* more reliable automation
* larger or more specialized layouts

### 3. The editor and UI must keep up

This slice is large, but it should still remain usable:

* clear labels
* clear variant identity
* understandable station panels

## Relation To The Concept

This slice pushes directly toward the concept by making the ship truly patchwork and evolutionary. The player should begin to move from a simple functional craft toward a machine built from specialized salvaged subsystems, each with operational consequences.

It also strengthens later ARCH and LUMEN work, because those systems become far more interesting once the machinery they target is mechanically diverse.

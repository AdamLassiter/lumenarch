# LUMEN.md — LUMEN Processor Specification

## Overview

**L.U.M.E.N.** processors are rare alien-derived optimisation systems that operate alongside A.R.C.H. computers.

Unlike ARCH computers, which directly command components by writing to their control registers, LUMEN processors alter the performance characteristics of nearby components through **BUFF** and **NERF** instructions.

LUMEN does not replace ARCH. Instead, it modifies the behaviour of components that are already operating, allowing advanced ships to become more efficient, more resilient, or more disruptive.

In practical terms:

* **ARCH controls components**
* **LUMEN biases components**
* **ARCH says what to do**
* **LUMEN makes nearby systems better or worse at doing it**

\---

## Core Design Goals

LUMEN should:

* Provide late-game scaling without requiring massive scripts
* Encourage hybrid ARCH/LUMEN ship designs
* Allow indirect interaction with hostile vessels
* Feel alien, powerful, and slightly unfair
* Avoid becoming a full replacement for ordinary automation

\---

## Execution Model

A LUMEN processor executes a short ARCH-like script once per compute cycle.

It supports ordinary ARCH utility, math, comparison, logic, and flow-control instructions, but its defining feature is access to two special instructions:

```arch
BUFF <REGISTER> <CONDITION>
NERF <REGISTER> <CONDITION>
```

The processor evaluates its script deterministically each tick.

As with ARCH:

* Reads come from the current state
* Writes apply to the next state
* Backward jumps halt execution until the next tick
* Numeric literals are read-only
* General purpose registers may be used for intermediate values

\---

## LUMEN Field Range

Each LUMEN processor emits a **LUMEN influence field**.

BUFF and NERF instructions only affect components within this field.

The field may have properties such as:

* Range
* Falloff
* Maximum affected components
* Optimisation budget
* Deoptimisation budget

Higher-tier LUMEN processors may have:

* Larger range
* Faster compute cycles
* Larger BUFF/NERF budgets
* Better falloff
* More script space

\---

## Register Targeting

LUMEN instructions target components by register identity.

A component is eligible for BUFF or NERF if it has a register matching the first argument’s:

```text
Category + Type + Property + Channel
```

Example:

```arch
BUFF RRP9 GP01
```

This targets nearby components with:

```text
RRP9
```

Meaning:

* `R` = Reactor
* `R` = RBMK fission reactor
* `P` = Power output
* `9` = Channel 9

If one or more nearby components expose that register, LUMEN may apply a performance modifier associated with that property.

\---

## Register Prefixes

LUMEN uses the same register naming scheme as ARCH.

There is no special output prefix.

Correct:

```arch
BUFF RRP9 GP01
NERF WTF0 GP07
```

Incorrect:

```arch
BUFF ORRP9 GP01
NERF OWTF0 GP07
```

\---

## BUFF Instruction

```arch
BUFF <REGISTER> <CONDITION>
```

BUFF attempts to improve nearby matching components when `CONDITION` is non-zero.

The target register determines what kind of property the LUMEN processor is trying to improve.

Examples:

```arch
BUFF RRP9 GP01
BUFF SRI3 GP02
BUFF WTD4 GP03
BUFF ECF1 GP04
```

Possible BUFF interpretations:

|Target Register|Effect|
|-|-|
|Reactor power output|Increase power output efficiency|
|Reactor stability|Improve stability|
|Shield integrity|Increase effective shield integrity|
|Shield field strength|Improve field strength|
|Weapon damage|Increase effective damage|
|Weapon cooldown|Reduce cooldown|
|Engine force output|Increase thrust efficiency|
|Engine heat|Reduce heat generation|
|Fabricator output|Improve production rate|

BUFF does not necessarily write a new visible value directly into the register. Instead, it applies an optimisation modifier to the component property represented by that register.

\---

## NERF Instruction

```arch
NERF <REGISTER> <CONDITION>
```

NERF attempts to worsen nearby matching components when `CONDITION` is non-zero.

The target register determines what property is being disrupted.

Examples:

```arch
NERF WTF0 GP07
NERF SRI2 GP08
NERF RRS5 GP09
NERF ECF1 GP10
```

Possible NERF interpretations:

|Target Register|Effect|
|-|-|
|Weapon fire command|Reduce firing reliability|
|Weapon damage|Reduce effective damage|
|Weapon cooldown|Increase cooldown|
|Shield integrity|Weaken shield integrity|
|Shield field strength|Reduce field strength|
|Reactor stability|Reduce stability|
|Reactor power output|Reduce effective power output|
|Engine force output|Reduce thrust|
|Detector lock quality|Reduce sensor accuracy|

NERF can affect hostile vessels if they are within the LUMEN field.

Enemy registers are read-only from the player’s perspective, but LUMEN does not need to write to them directly. It applies an external modifier to matching components.

\---

## Conditions

The second argument to BUFF or NERF is a condition value.

If the condition is:

* `0` → no effect
* Non-zero → effect is active

Example:

```arch
GT VPD GP00 GP01
BUFF RRP9 GP01
```

If vessel power draw exceeds the comparison threshold, `GP01` becomes `1`, activating the BUFF.

\---

## Budget System

Each LUMEN processor has two separate budgets:

* **BUFF budget**
* **NERF budget**

Every active BUFF or NERF consumes budget.

The cost may depend on:

* Number of matching components
* Distance from processor
* Strength of effect
* Component tier
* Property being modified
* Whether the target is friendly or hostile

If budget is insufficient, effects may be:

* Reduced in strength
* Applied only to the nearest matching components
* Distributed proportionally
* Ignored after budget exhaustion

\---

## Suggested Budget Resolution

When resolving BUFF/NERF effects:

1. Evaluate all active BUFF and NERF instructions.
2. Identify matching components within LUMEN range.
3. Calculate base effect strength.
4. Apply distance falloff.
5. Allocate budget.
6. Apply final modifiers to affected components.

If multiple LUMEN processors affect the same component, modifiers may stack with diminishing returns.

\---

## Falloff

LUMEN field falloff may use the same broad model as other fields:

* None
* Linear
* Inverse-square

Example:

|Falloff|Behaviour|
|-|-|
|None|Full effect throughout range|
|Linear|Effect weakens evenly with distance|
|Inverse-square|Strong nearby, very weak at range|

High-tier LUMEN processors may have gentler falloff.

\---

## Friendly and Hostile Targeting

LUMEN does not inherently distinguish friendly from hostile systems.

It targets by:

* Register identity
* Channel
* Physical proximity
* Condition

This enables both powerful and dangerous behaviour.

Example risk:

```arch
BUFF WTF0 1
```

If an enemy turret on channel 0 is inside range, it may also be buffed.

Players can avoid accidental effects by:

* Choosing unusual channels
* Restricting LUMEN range
* Using detector logic to gate conditions
* Designing ship layouts carefully

\---

## Channel Warfare

Because LUMEN targets register/channel combinations, channels become tactically important.

Common channels may be dangerous.

For example, if many cheap weapons default to channel `0`, a player may deliberately avoid using channel `0` on their own weapons and carry a hostile-disruption LUMEN script:

```arch
GT WTF0 0 GP07
NERF WTF0 GP07
```

This checks whether any turret on channel `0` is firing nearby, then disrupts such turrets.

Since the player has no weapons on channel `0`, this is likely to affect enemies instead.

\---

## Example: Emergency Reactor Buff

When ship power draw exceeds 80% of generation, buff reactor output on channel 9.

```arch
MUL VPG 0.8 GP00      # 80% of vessel power generation
GT VPD GP00 GP01      # true if draw exceeds 80% generation
BUFF RRP9 GP01        # improve RBMK reactor power output on channel 9
```

\---

## Example: Sneaky Enemy Turret Disruption

The ship intentionally has no weapons on channel 0.

The LUMEN processor has a large field and can influence nearby hostile vessels.

```arch
# No friendly weapons use channel 0
# Enemy vessels often leave turrets on default channel 0

GT WTF0 0 GP07        # detect nearby firing turrets on channel 0
NERF WTF0 GP07        # disrupt those turrets if they are firing
```

\---

## Example: Heat-Aware Weapon Buff

Buff weapon damage only while ship heat is under control.

```arch
LT VHT 600 GP00       # true if vessel heat is below 600
BUFF WTD2 GP00        # improve weapon damage on turret channel 2
```

\---

## Example: Shield Emergency Stabilisation

Buff shield integrity when it falls below a dangerous threshold.

```arch
LT SRI3 40 GP00       # true if shield integrity is below 40
BUFF SRI3 GP00        # reinforce shield integrity on channel 3
```

\---

## Example: Reactor Safety NERF

If a reactor is too hot, nerf its own reaction rate to suppress runaway behaviour.

```arch
GT RRH4 850 GP00      # reactor heat too high
NERF RRF4 GP00        # suppress fission rate on channel 4
```

This is not as precise as ARCH directly lowering `RRF4`, but it may work even when ordinary control is unavailable or insufficient.

\---

## Example: Sensor Interference

Disrupt nearby hostile sensors on a common detector channel.

```arch
GT DSH0 0 GP00        # hostile contact detected on ship detector channel 0
NERF DSL0 GP00        # reduce lock quality of detectors on channel 0
```

\---

## Example: Emergency Mobility Boost

When hostile contact is close, buff engine force output.

```arch
LT DSN0 300 GP00      # target within 300 units
BUFF ECF1 GP00        # improve force output of engine channel 1
```

\---

## Example: Power Conservation

If power reserve is low, nerf high-consumption weapons.

```arch
LT VPR 100 GP00       # low power reserve
NERF WTP2 GP00        # reduce effective power draw / weapon demand on channel 2
```

Depending on implementation, nerfing a power draw register could either:

* Reduce component power consumption
* Starve the component, reducing effectiveness
* Increase instability if overused

For clarity, the recommended interpretation is:

> BUFF on a cost register reduces the cost.  
> NERF on a cost register increases the cost or reduces access to that resource.

\---

## Property Interpretation Rules

LUMEN effects depend on the type of property being targeted.

### Output Properties

Examples:

* Reactor power output
* Engine force output
* Weapon damage
* Fabricator output

```arch
BUFF <output> condition
```

Improves output.

```arch
NERF <output> condition
```

Reduces output.

\---

### Cost Properties

Examples:

* Power draw
* Heat generation
* Fuel consumption

```arch
BUFF <cost> condition
```

Reduces cost.

```arch
NERF <cost> condition
```

Increases cost or worsens efficiency.

\---

### Stability / Integrity Properties

Examples:

* Reactor stability
* Shield integrity
* Component health

```arch
BUFF <stability> condition
```

Improves resilience.

```arch
NERF <stability> condition
```

Weakens resilience.

\---

### Command Properties

Examples:

* Weapon fire command
* Engine throttle command
* Shield facing command

```arch
BUFF <command> condition
```

Improves responsiveness or reliability.

```arch
NERF <command> condition
```

Adds delay, drift, noise, or unreliability.

\---

## Design Constraints

LUMEN should not become simple remote hacking.

It should not directly assign enemy control values.

Instead, it should apply probabilistic, efficiency-based, or modifier-based effects.

Recommended constraints:

* Cannot directly write hostile registers
* Cannot force enemies to fire, jump, vent oxygen, etc.
* Can make systems worse at obeying their own commands
* Can improve friendly systems already under control
* Should be powerful but not perfectly predictable

\---

## Debugging and UI

Because LUMEN effects are indirect, the UI should clearly show:

* Active BUFF/NERF instructions
* Matching target registers
* Number of affected components
* Budget used
* Estimated effect strength
* Friendly/hostile/unknown affected targets
* Falloff losses

Example UI readout:

```text
BUFF RRP9
Condition: Active
Targets: 1 friendly reactor
Budget: 34 / 50
Effect: +12% effective power output
```

Example hostile readout:

```text
NERF WTF0
Condition: Active
Targets: 3 unknown turret systems
Budget: 41 / 60
Effect: -18% firing reliability
```

\---

## LUMEN Processor Progression

Possible LUMEN processor tiers:

### Fragment Core

* Short range
* Small BUFF budget
* No NERF capability

### Eidolon Lens

* Medium range
* Separate BUFF and NERF budgets
* Moderate script size

### LUMEN Heart

* Long range
* High budgets
* Reduced falloff
* Can affect multiple vessels reliably

### Quiet Singularity

* Extremely rare
* Large range
* Strong budget recovery
* Dangerous instability effects if overloaded

\---

## Narrative Role

LUMEN processors are not ordinary computers.

They are fragments of an alien optimisation system built by the Eidolon Remnants.

They do not understand machinery as commands and outputs. They understand systems as pressures, tendencies, and outcomes.

To humans, LUMEN appears to “bless” or “curse” components.

To engineers, it is a modifier layer.

To Continuants, it is alien witchcraft with a debugging panel.

\---

## Summary

LUMEN processors extend ARCH automation with indirect optimisation.

They:

* Use the same register language
* Add `BUFF` and `NERF`
* Target nearby components by register/channel
* Consume optimisation/deoptimisation budget
* Can affect friendly or hostile systems
* Enable late-game ship-wide balancing and electronic warfare

LUMEN is not about direct control.

It is about making systems more or less able to become what they already want to be.

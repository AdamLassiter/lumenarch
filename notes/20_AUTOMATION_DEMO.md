# 20_AUTOMATION_DEMO

## Goal

Add a first real family of **automation-support components** that give ARCH programs better awareness of ship, crew, and threat state without bypassing the existing register-driven control model.

This slice should sit between [15_ARCH_COMPLETE_DEMO.md](/home/adaml/code/lumenarch/notes/15_ARCH_COMPLETE_DEMO.md) and the broader ship/system simulation work already in place.

The point is not to make ARCH "smarter" by hidden magic. The point is to give players new physical modules they can install, power, position, and script against.

## Why This Slice Now

The game already has:

- real component-local control surfaces
- channel-driven ARCH workflows
- richer logistics and component families
- multiple kinds of runtime state worth reacting to

But there is still a missing layer in the automation fantasy:

- ARCH can issue commands
- ARCH can read some local state
- ARCH cannot yet build rich *sensor architectures*

Right now, many interesting automation ideas are either impossible or too brittle because the ship lacks dedicated observer modules.

This slice should make it possible to build things like:

- auto-close-range boarding alarms
- directional turret wake-up logic
- retreat / threat-recognition behaviors
- damage-control escalation when a region of the ship is getting hit
- "stand down when no hostiles are nearby" logic
- salvage and patrol behavior that responds to actual detected conditions

## Demo Pitch

The player installs a few specialized detector modules and assigns them channels.

During missions, ARCH computers can now read:

- whether life signs are nearby
- where the nearest one is
- how far away it is
- whether another ship is nearby and in what direction
- whether the player's ship is currently taking damage and from which side/region
- other compact machine-readable signals that help existing systems cooperate

The player should feel the difference between:

- a ship that can only react when the player notices a problem
- a ship with coarse automation triggers
- a ship with layered sensor coverage and deliberate scripted responses

## Core Design Direction

These components should be treated as **sensor modules**, not abstract UI features.

That means:

- they occupy physical ship space
- they have variants and tradeoffs
- they may have power draw or fragility implications
- they expose registers like any other module
- ARCH reads their outputs through channels

The player's automation capability should therefore become part of ship architecture:

- do you spend space on better detection?
- do you centralize sensing or distribute it?
- do you power a fragile high-tier detector near the front line?
- do you accept low-fidelity binary signals or invest in better directional/range info?

## Primary Component Families

## 1. Life Sign Detectors

Purpose:

- detect nearby living actors for anti-boarding, EVA support, rescue, or encounter awareness logic

Recommended tiers:

### Basic Life Sign Detector

Outputs:

- any friendly life sign nearby: yes/no
- any hostile life sign nearby: yes/no

Characteristics:

- short range
- cheap
- low power draw
- coarse signal only

Useful ARCH patterns:

- trigger boarding alarm
- lock/unlock interior doors
- enable crew-safe or crew-unsafe system states

### Directional Life Sign Detector

Outputs:

- friendly nearby: yes/no
- hostile nearby: yes/no
- direction of nearest friendly
- direction of nearest hostile

Characteristics:

- medium range
- higher cost
- enough precision for directional reactions

Direction format should stay compact and deterministic, for example:

- nearest target x offset sign / quantized x
- nearest target y offset sign / quantized y
- or a small direction index such as front/right/rear/left plus diagonals

Useful ARCH patterns:

- rotate defenses or orient crew response
- trigger side-specific lockdown logic
- send player-facing status summaries

### Survey Life Sign Detector

Outputs:

- all directional outputs above
- distance to nearest friendly
- distance to nearest hostile

Characteristics:

- largest range
- highest cost and power draw
- best for deliberate sensor ships or advanced automation cores

Useful ARCH patterns:

- threat-distance-based alert levels
- boarding defense staging
- EVA safety thresholds

## 2. Ship Detectors

Purpose:

- detect other ships as objects of tactical and navigational concern, independent of crew presence

These should be parallel to the life sign detectors in progression structure.

### Basic Ship Detector

Outputs:

- any ship nearby: yes/no
- any hostile ship nearby: yes/no

Useful ARCH patterns:

- power up combat systems only when needed
- switch logistics/fabrication to battle-safe mode

### Directional Ship Detector

Outputs:

- nearby / hostile yes-no
- direction of nearest ship
- direction of nearest hostile ship

Useful ARCH patterns:

- face shields or turrets toward contact direction
- trigger movement/turn behaviors

### Survey Ship Detector

Outputs:

- all directional outputs above
- distance to nearest ship
- distance to nearest hostile ship

Useful ARCH patterns:

- open/close engagement envelopes
- start charge-up logic before direct visual/manual confirmation
- retreat or advance thresholds

## 3. Ship Damage Detectors

Purpose:

- tell ARCH not just that the ship is damaged in the abstract, but that recent incoming damage or structural stress is happening somewhere meaningful

These should tie directly into the existing damage/integrity model.

Recommended progression:

### Damage Alarm Node

Outputs:

- ship currently taking damage: yes/no
- any module critically damaged: yes/no

Useful ARCH patterns:

- emergency power routing
- disable risky systems during cascading failures
- page the player to a control station

### Directional Damage Detector

Outputs:

- current incoming damage yes/no
- direction / side of most recent significant damage
- critical-damage-present yes/no

Useful ARCH patterns:

- aim directional shields
- rotate ship/turrets
- decide which side to protect or withdraw

### Structural Surveyor

Outputs:

- current incoming damage
- direction / side of strongest recent damage
- quantized damage intensity
- distance or region index to the most damaged detected module cluster

Useful ARCH patterns:

- damage-control triage
- reroute behavior around failing ship regions
- create meaningful advanced automation for shield/retreat logic

## Additional Automation-Aiding Components Worth Including

The slice should include at least one or two more sensor/control-support modules beyond the obvious detector trio, so the ship feels like it is gaining an actual automation toolkit.

## 4. Power State Monitor

Purpose:

- expose cleaner machine-readable summaries of ship power stress than forcing every script to infer them indirectly

Potential outputs:

- power deficit yes/no
- reactor brownout risk yes/no
- battery reserve low yes/no
- capacitor reserve low yes/no

Why it helps:

- ties together reactors, batteries, capacitors, engines, and weapons
- enables sensible fallback/priority scripts

This should not replace existing register surfaces; it should aggregate them into a monitor component for simpler automation setups.

## 5. Heat / Hazard Monitor

Purpose:

- provide local or ship-wide warning signals about dangerous thermal conditions

Potential outputs:

- overheat nearby yes/no
- critical overheat yes/no
- direction of hottest nearby detected region

Why it helps:

- supports damage prevention before catastrophic failure
- ties into fields/heat work already present in the simulation

## 6. Logistics Demand Beacon

Purpose:

- expose simple automation-friendly summaries of whether nearby logistics systems are starved or blocked

Potential outputs:

- processor input needed
- output storage full
- ammo demand nearby
- fuel demand nearby
- repair-material demand nearby

Why it helps:

- ties drones, manipulators, processors, storage, fabricators, and weapons/reactors together
- makes ARCH-based logistics orchestration more legible without needing every script to poll many stations manually

This component should be especially valuable if the ship already has drone/manipulator-rich logistics.

## Unifying Design Rules

## 1. Better sensors should mean better information, not free action

These modules should output facts and summaries. They should not directly aim turrets, move drones, or toggle systems on their own.

## 2. Variants should improve fidelity, not just raw range

The low/mid/high progression should feel meaningful because:

- low tier gives binary triggers
- mid tier adds directional reasoning
- high tier adds distance / intensity / prioritization detail

## 3. Sensor outputs should stay compact

ARCH should receive information in a bounded, deterministic, scripting-friendly way.

Avoid dumping giant contact lists. Prefer:

- presence flags
- nearest-contact direction
- nearest-contact distance
- side / region identifiers
- quantized severity bands

## 4. Physical placement should matter where possible

At least some detectors should benefit from position, range, or facing assumptions rather than acting as perfect global awareness by default.

The exact model can start simple:

- radius around the module
- ship-centered but range-limited
- front-biased or local-region-biased scans for some variants

But placement should matter enough that "sensor architecture" becomes part of ship design.

## 5. The modules should be useful even for simple scripts

Good first-pass automation should be possible with short ARCH programs:

- "if hostile ship nearby, enable combat posture"
- "if hostile life sign inboard, sound alarm and close airlock"
- "if damage from starboard, bias shield / maneuver response"
- "if power deficit, disable nonessential fabrication"

## Register / Channel Surface Plan

Each detector family should expose a coherent set of outputs so variants layer naturally.

Example pattern:

- `detected_any`
- `detected_hostile`
- `nearest_dir_x`
- `nearest_dir_y`
- `nearest_distance`
- `severity`

Not every variant exposes every register:

- low tier may only give the binary flags
- mid tier adds direction
- high tier adds distance/intensity

This preserves family consistency while making higher tiers genuinely richer.

## Interaction With Existing Systems

These modules should help tie together current systems rather than creating isolated new minigames.

### Turrets / Shields / Helm

- ship detectors and damage detectors enable combat posture logic
- directional info supports shield orientation and movement response

### Reactors / Batteries / Capacitors

- power monitors help build graceful load-shedding scripts

### Logistics / Drones / Fabricators

- logistics demand beacons help advanced supply ships script production and delivery priorities

### Boarding / EVA / Internal Security

- life sign detectors enable automated anti-boarding, EVA pickup support, and safer internal state management

### ARCH Identity

This slice should reinforce that ARCH is about engineering observation plus bounded control, not omniscient AI.

## Risks / Questions

### 1. Over-aggregation risk

If monitors become too abstract, they may trivialize the more detailed component-level register model.

Response:

- keep monitors concise and purpose-built
- let advanced players still benefit from reading lower-level registers directly

### 2. Detection semantics risk

"Nearby" and "hostile" must be simulation-meaningful and deterministic.

Questions to settle:

- does "hostile" mean opposing ship ownership, boarded enemy actor presence, or active threat state?
- are cloaked/hidden/future entities out of scope for now?
- are detectors ship-local only, or do they operate in encounter/world space too?

### 3. UI overload risk

Too many new detector modules could clutter the editor.

Response:

- group them as a coherent sensor family
- keep variants visually and textually clear

## Success Criteria

- the player can build noticeably better ARCH behavior by adding detector modules
- low-tier detector builds are useful early, not merely stepping stones
- higher-tier detectors make more sophisticated scripts possible without hidden AI assistance
- existing component families feel more connected because the ship can now observe itself and its surroundings better
- sensor placement and variant choice feel like real ship-design decisions

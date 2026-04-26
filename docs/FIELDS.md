# 📄 FIELDS — Field Systems & Environmental Simulation

---

## 🧠 Overview

Fields are the primary mechanism by which components, entities, and the environment influence one another in **LUMEN//ARCH**.

> **Fields define how energy, matter, and influence propagate through space.**

Fields are:

* Spatially defined
* Continuously evaluated
* Additive in nature
* Interpreted differently by each component or entity

---

# 📐 Field Representation

---

## 🧩 Field Definition

Each field instance is defined as:

```text
Field = {
  type,
  shape,
  origin,
  intensity,
  falloff,
  lifetime
}
```

---

## 🔷 Field Shapes

Fields are emitted as simple geometric volumes:

| Shape     | Description                      |
| --------- | -------------------------------- |
| Circle    | Radius-based, omnidirectional    |
| Arc       | Directional sector               |
| Rectangle | Directional projection           |
| Line      | Thin beam (degenerate rectangle) |

---

## 🎯 Intersection Rule

> World-space fields affect a component or entity if its **center point lies within the field shape**.

This ensures:

* Predictable interactions
* Efficient computation
* Clear visualisation

---

## 📉 Falloff Types

| Type           | Description             |
| -------------- | ----------------------- |
| None           | Uniform intensity       |
| Linear         | Decreases with distance |
| Inverse-square | Rapid drop-off          |

---

# ⏳ Field Lifetime

---

## Categories

### 🔗 Attached Fields

* Exist only while component is active
* Example: reactor heat

---

### 🌫 Persistent Fields

* Remain after source is gone
* Decay over time
* Example: explosion heat cloud

---

### 🔁 Continuous Fields

* Recomputed every tick
* Example: engine thrust, beam weapons

---

# 🔁 Field Evaluation Model

---

Fields are evaluated continuously during gameplay.

Conceptually:

```text
1. Components emit fields
2. Fields exist in world space (continuous coordinates)
3. Entities/components sample fields at their position
4. Effects applied based on field intensity
```

---

## 📍 Coordinate Space

* Ships: grid-based internally
* World: continuous position + rotation
* Fields: always defined in world space

---

# ⚖️ Field Stacking

---

## 🧮 Additive Model

Fields of the same type combine linearly:

```text
Total Field = Σ (all contributing fields)
```

---

### Example

```text
+100 Heat (reactor)
-50 Heat (radiator)
= 50 Net Heat
```

---

## 🧊 Clamping

Some effects may clamp values:

* Minimum / maximum thresholds
* Component-specific limits

---

# 🌊 Flood-Fill Systems

---

Some systems are not explicit fields, but **propagated volumes**.

---

## 🫧 Oxygen

* Simulated per ship tile
* Flood-fills enclosed spaces
* Lost through hull breaches
* Recomputed when topology changes

---

## 🔥 Ambient Heat Dissipation

* Simulated per ship tile
* Heat equalises across connected spaces
* Moves away from sources over time

---

## 📌 Rule

> Flood-fill systems operate only within enclosed ship volumes and are sampled from the tile underneath an entity or component center.

---

# 🧩 Field Types

---

# ❤️ Helpful / Harmful Fields

---

## 🔥 Heat / Cold

* Scalar field (positive/negative)

### Sources:

* Reactors
* Weapons
* Engines
* Radiators (negative)

### Effects:

| Target     | Effect                     |
| ---------- | -------------------------- |
| Components | Damage, reduced efficiency |
| Player     | Burning damage             |
| Drones     | Damage                     |

---

## ☢️ Radiation

### Sources:

* Fission reactors
* Damaged systems

### Effects:

| Target     | Effect          |
| ---------- | --------------- |
| Components | Efficiency loss |
| Player     | Damage          |
| Drones     | Efficiency loss |

---

## 🌬 Oxygen (Flood-Fill)

### Sources:

* Life support systems

### Effects:

| Target     | Effect                |
| ---------- | --------------------- |
| Player     | Required for survival |
| Components | No effect             |
| Drones     | No effect             |

### Sampling Rule

* Each enclosed vessel maintains an internal tile lattice
* Oxygen values live on tiles, not free-floating field instances
* Components and actors sample the tile containing their center point

---

## 🛡 Shield

### Nature:

* Threshold-based protective field

### Behaviour:

* Blocks external harmful fields
* Does NOT block:

  * Internal fields
  * Friendly emissions

---

## ⚡ Electrical

---

### Purpose:

Represents electrical instability, EMP effects, and system interference.

---

### Sources:

* Damaged reactors
* EMP weapons
* Overloaded systems
* Electrical arcs

---

### Effects:

| Target     | Effect                                     |
| ---------- | ------------------------------------------ |
| Components | Reduced reliability, instability           |
| ARCH       | Potential signal noise (optional advanced) |
| LUMEN      | Reduced optimisation effectiveness         |
| Drones     | Task disruption                            |
| Player     | Minor damage or stun                       |

---

### Gameplay Role:

* Disrupts automation-heavy builds
* Creates non-destructive system pressure
* Enables electronic warfare mechanics

---

# 🌍 Physics Fields

---

## 🚀 Thrust / Force (Vector Field)

---

### Sources:

* Engines
* Explosions

---

### Effects:

| Target     | Effect                |
| ---------- | --------------------- |
| Ships      | Movement              |
| Components | None                  |
| Entities   | Physical displacement |

---

### Notes:

* Directional
* May affect nearby ships

---

## 🌌 Gravity (Special Case)

---

### Sources:

* Environmental anomalies
* Massive objects

---

### Effects:

* Alters trajectories
* Pulls objects

---

# 🧠 Utility Fields

---

## 🧠 ARCH Field

* Defines range of programmable control
* Allows reading/writing to components

---

## 🌌 LUMEN Field

* Defines optimisation range
* Enables BUFF / NERF application

---

## 🤖 Drone Control Field

* Defines operational radius for drones
* Limits task assignment range

---

## 📌 Note

These are **non-physical fields**:

* Do not interact with environment
* Only define system reach

---

# 🧱 Tile Volumes

Internal ship atmosphere and ambient heat are simulated as dense tile volumes rather than ordinary world fields.

## Rules

* Tile volumes exist in vessel-local grid coordinates
* Emitters inject heat or oxygen into one or more tiles
* Hull changes rebuild enclosure topology
* Components and actors sample tile values by center-point lookup
* World-space effects such as beams, thrust, and radiation remain ordinary fields

---

# ⚔️ Field Interaction Rules

---

## General Principles

* Fields do not inherently define effects
* Components interpret fields individually

---

## Examples

| Field      | Typical Effect  |
| ---------- | --------------- |
| Heat       | Damage          |
| Radiation  | Efficiency loss |
| Electrical | Disruption      |
| Shield     | Protection      |

---

## Shield Interaction

* Blocks external harmful fields
* Ignores friendly sources
* Threshold-based mitigation

---

# 🧍 Entity Interaction

---

## Player

| Field      | Effect           |
| ---------- | ---------------- |
| Heat       | Damage           |
| Radiation  | Damage           |
| Oxygen     | Required         |
| Electrical | Minor disruption |

---

## Drones

| Field      | Effect          |
| ---------- | --------------- |
| Heat       | Damage          |
| Radiation  | Efficiency loss |
| Electrical | Task disruption |
| Oxygen     | No effect       |

---

## Components

| Field      | Effect          |
| ---------- | --------------- |
| Heat       | Damage          |
| Radiation  | Efficiency loss |
| Electrical | Instability     |

---

# 🌌 Environmental Fields

---

Environment may include:

* Radiation zones
* Heat clouds
* Electrical storms
* Gravity anomalies

---

## Implementation

> Environmental fields may be implemented as special “null vessels” emitting fields.

---

# 👁 Field Visualisation & UI

---

## Player Feedback

* Field intensity readouts:

  * Heat
  * Radiation
  * Oxygen
  * Electrical

---

## Indicators

* Safe / warning / critical thresholds
* Visual overlays
* HUD alerts

---

## Component Inspection

* Shows:

  * Local field values
  * Damage rates
  * Efficiency modifiers

---

## Ship Overview System (Optional Component)

Provides:

* Aggregate field mapping
* Field source locations
* Overlay visualisation

---

# 🧪 Examples

---

## 🔥 Reactor Heat Field

* Emits circular heat field
* High intensity near core
* Decreases with distance

---

## ❄️ Radiator Cooling

* Emits negative heat
* Balances reactor output

---

## ⚡ Electrical Arc

* Line-shaped field
* Short-lived
* High disruption

---

## 🚀 Engine Plume

* Rectangular field behind engine
* Applies force + heat

---

## 🛡 Shield Dome

* Circular field
* Threshold mitigation

---

# 🎯 Design Summary

---

Fields in **LUMEN//ARCH**:

* Provide a unified environmental simulation layer
* Enable emergent system interactions
* Tie together:

  * Combat
  * Logistics
  * Automation
  * Player survival

---

## Core Principles

* **Simple rules, complex outcomes**
* **Additive interactions**
* **Component-defined responses**
* **Spatial gameplay relevance**

---

## Final Insight

> **Fields are the invisible systems that make everything else meaningful.**

They turn:

* Ships into ecosystems
* Combat into system interactions
* Automation into something that must adapt to reality

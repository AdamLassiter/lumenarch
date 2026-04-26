# 📄 LOGISTICS — Ship Logistics & Resource Systems

---

## 🧠 Overview

Logistics in **LUMEN//ARCH** is a **physical, spatial, and contested system**.

> **Resources do not teleport. They are moved, transformed, reserved, and consumed.**

All logistics emerges from three foundational component types:

* **Storage** — holds resources
* **Manipulators** — move resources
* **Processors** — transform resources

Later in the game, **drones extend and scale this system**, but do not replace its underlying rules.

---

# 📦 Resource Model

---

## 🔀 Resource Categories

---

### 💰 1. Barter Resources

Used primarily for trade and value extraction.

| Resource   | Description           |
| ---------- | --------------------- |
| Scrap      | Raw wreckage          |
| Components | Intact modules        |
| Metals     | Refined base material |
| Alloys     | Higher-tier material  |

---

### ⚙️ 2. Operational Resources

Consumed by ship systems.

| Resource        | Use                 |
| --------------- | ------------------- |
| Fuel            | Reactors            |
| Ammo            | Weapons             |
| Oxygen          | Life support        |
| Coolant         | Heat management     |
| Repair Material | Component repair    |
| Drone Equipment | Enables drone roles |

---

# 🏗 Core Logistics Components

---

# 📦 Storage Components (`LS`)

---

## Subtypes

These are content variants, not distinct register prefixes:

* Scrap storage
* Metal storage
* Fuel storage
* Oxygen tank
* Ammo storage
* General storage

---

## Registers

| Register | Access | Description     |
| -------- | ------ | --------------- |
| `LSF0`   | Read   | Fill level (%)  |
| `LSC0`   | Read   | Capacity        |
| `LSM0`   | Read   | Stored mass     |
| `LST0`   | Read   | Resource type   |
| `LSR0`   | Read   | Reserved amount |
| `LSI0`   | Read   | Input rate      |
| `LSO0`   | Read   | Output rate     |

---

## 🔒 Reservation System

All resource usage follows:

1. **Reserve** → `LSR`
2. **Transport**
3. **Consume**

Prevents:

* Double allocation
* Race conditions
* Deadlocks (in well-designed systems)

---

# 🦾 Manipulators (`LM`)

---

## Subtypes

These are content variants, not distinct register prefixes:

* Grabber arm
* Conveyor (optional upgrade)
* Loader

---

## Function

Manipulators:

* Move resources between adjacent components
* Operate within a fixed range (field)
* Are the **early-game logistics backbone**

---

## Registers

| Register | Access | Description      |
| -------- | ------ | ---------------- |
| `LME0`   | Write  | Extend           |
| `LMR0`   | Write  | Retract          |
| `LMG0`   | Write  | Grip (0/1)       |
| `LMD0`   | Write  | Target direction |
| `LML0`   | Read   | Load held        |
| `LMS0`   | Read   | Status           |
| `LMP0`   | Read   | Power usage      |

---

## Example: Simple Grab Cycle

```arch
# Extend arm
MOV 1 LME0

# Grip resource
MOV 1 LMG0

# Retract
MOV 1 LMR0
```

---

# 🏭 Processing Components (`LP`)

---

## Subtypes

These are content variants, not distinct register prefixes:

* Scrapper
* Refiner
* Fuel processor
* Ammo fabricator
* Equipment fabricator

---

## Function

Processors:

* Consume input resources
* Produce output resources
* Require power and adjacency/logistics

---

## Registers

| Register | Access | Description    |
| -------- | ------ | -------------- |
| `LPI0`   | Read   | Input rate     |
| `LPO0`   | Read   | Output rate    |
| `LPP0`   | Read   | Power usage    |
| `LPS0`   | Read   | Status         |
| `LPY0`   | Write  | Recipe ID      |
| `LPQ0`   | Read   | Craft progress |

---

## Example Recipes

| Recipe          | Input            | Output |
| --------------- | ---------------- | ------ |
| Scrap → Metals  | Scrap            | Metals |
| Metals → Ammo   | Metals           | Ammo   |
| Metals + Fuel   | Fuel Rods        |        |
| Metals → Repair | Repair Materials |        |

---

## Example: Enable Scrap Processing

```arch
# If scrap storage > 20%, run scrapper
GT LSF0 20 GP00
MOV GP00 LPS0
```

---

# 🔁 Early-Game Logistics Flow

---

## Example Chain

```text
Wreck → Scrap Storage → Grabber → Scrapper → Metal Storage
```

---

## Key Constraints

* Range-limited manipulators
* Physical adjacency matters
* Throughput limited by:

  * Arm speed
  * Storage capacity
  * Processor rate

---

# 🤖 Late-Game: Drone Logistics

---

## 🧠 Core Idea

> Drones are mobile manipulators with autonomy.

They:

* Extend logistics beyond adjacency
* Operate at range
* Handle complex routing

---

# 🏭 Drone Station (`DRS`)

---

## Function

* Spawns and manages drones
* Assigns tasks
* Handles coordination

---

## Registers

| Register | Access | Description   |
| -------- | ------ | ------------- |
| `DRC0`   | Write  | Mode          |
| `DRN0`   | Read   | Active drones |
| `DRM0`   | Read   | Max drones    |
| `DRP0`   | Read   | Power usage   |
| `DRR0`   | Read   | Range         |
| `DRL0`   | Read   | Active tasks  |
| `DRQ0`   | Read   | Task queue    |
| `DRS0`   | Read   | Status        |

---

# 🎛 Drone Modes

---

| Mode | Function  |
| ---- | --------- |
| `0`  | Idle      |
| `1`  | Repair    |
| `2`  | Salvage   |
| `3`  | Logistics |
| `4`  | Combat    |
| `5`  | Return    |

---

# 📦 Drone Logistics Model

---

## Task Lifecycle

---

### 1. Task Creation

A task is generated when:

* A processor needs input
* Storage is overfilled
* Salvage is available

---

### 2. Reservation

System locks:

* Source storage
* Resource quantity
* Target component

---

### 3. Assignment

* Drone station assigns drone
* Drone reserves required equipment

---

### 4. Execution

Drone:

* Moves to source
* Collects resource
* Moves to destination
* Delivers resource

---

### 5. Completion

* Locks released
* Drone returns or takes new task

---

# 🔒 Task Locking System

---

Each task locks:

* **Source** (storage)
* **Target** (component)
* **Resources**
* **Tiles (optional)**

Prevents:

* Multiple drones targeting same resource
* Overconsumption
* Conflicting assignments

---

# 🧠 Example: Logistics Automation

---

## Enable Drones When Storage High

```arch
GT LSF0 80 GP00
MOV GP00 DRC0
```

---

## Prioritise Repair Over Logistics

```arch
LT VHP 70 GP00

JEQ GP00 1 :REPAIR
MOV 3 DRC0
JMP :END

:REPAIR MOV 1 DRC0

:END NOP
```

---

# ⚙️ Hybrid Systems

---

## Early Game

* Manual + grabbers
* Slow, predictable

## Mid Game

* Mixed manipulators + drones
* Semi-automated

## Late Game

* Drone-dominated logistics
* Fully automated pipelines

---

# 🔥 Emergent Behaviour

---

## Efficient Ship

* Balanced storage
* Smooth flow
* Minimal idle time

---

## Inefficient Ship

* Resource starvation
* Bottlenecks
* Idle drones
* Deadlocks

---

# 🎯 Design Outcomes

---

This system ensures:

* **Ship layout matters**
* **Automation is physical**
* **Scaling requires planning**
* **Drones enhance, not replace systems**

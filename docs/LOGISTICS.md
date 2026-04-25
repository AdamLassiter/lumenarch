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

# 📦 Storage Components (`ST`)

---

## Subtypes

| Code | Type            |
| ---- | --------------- |
| `SS` | Scrap Storage   |
| `SM` | Metal Storage   |
| `SF` | Fuel Storage    |
| `SO` | Oxygen Tank     |
| `SA` | Ammo Storage    |
| `SG` | General Storage |

---

## Registers

| Register | Access | Description     |
| -------- | ------ | --------------- |
| `STF0`   | Read   | Fill level (%)  |
| `STC0`   | Read   | Capacity        |
| `STM0`   | Read   | Stored mass     |
| `STT0`   | Read   | Resource type   |
| `STR0`   | Read   | Reserved amount |
| `STI0`   | Read   | Input rate      |
| `STO0`   | Read   | Output rate     |

---

## 🔒 Reservation System

All resource usage follows:

1. **Reserve** → `STR`
2. **Transport**
3. **Consume**

Prevents:

* Double allocation
* Race conditions
* Deadlocks (in well-designed systems)

---

# 🦾 Manipulators (`MA`)

---

## Subtypes

| Code | Type                        |
| ---- | --------------------------- |
| `MG` | Grabber Arm                 |
| `MC` | Conveyor (optional upgrade) |
| `ML` | Loader                      |

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
| `MAE0`   | Write  | Extend           |
| `MAR0`   | Write  | Retract          |
| `MAG0`   | Write  | Grip (0/1)       |
| `MAT0`   | Write  | Target direction |
| `MAL0`   | Read   | Load held        |
| `MAS0`   | Read   | Status           |
| `MAP0`   | Read   | Power usage      |

---

## Example: Simple Grab Cycle

```arch
# Extend arm
MOV 1 MAE0

# Grip resource
MOV 1 MAG0

# Retract
MOV 1 MAR0
```

---

# 🏭 Processing Components (`PR`)

---

## Subtypes

| Code | Type                 |
| ---- | -------------------- |
| `PS` | Scrapper             |
| `PR` | Refiner              |
| `PF` | Fuel Processor       |
| `PA` | Ammo Fabricator      |
| `PP` | Equipment Fabricator |

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
| `PRI0`   | Read   | Input rate     |
| `PRO0`   | Read   | Output rate    |
| `PRP0`   | Read   | Power usage    |
| `PRS0`   | Read   | Status         |
| `PRT0`   | Write  | Recipe ID      |
| `PRC0`   | Read   | Craft progress |

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
GT STF0 20 GP00
MOV GP00 PRS0
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
GT STF0 80 GP00
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

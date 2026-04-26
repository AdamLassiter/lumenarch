# 📄 ARCHLANG — *ARCH Systems Specification*

---

## 🧠 Overview

**A.R.C.H. (Adaptive Runtime Control Heuristic)** is a deterministic, assembly-style scripting system used to automate ship components.

Programs:

* Execute fully each tick
* Read from the **current state**
* Write to the **next state**
* Are bounded in time (no infinite loops)

---

# ⚙️ Instruction Set

---

## 🧾 Instruction Format

```arch
[:LABEL] OPC ARG0 ARG1 ARG2 # comment
```

* Labels: `:XXXX` (max 4 chars)
* Comments: `#`
* Arguments: registers or numeric literals

---

## 📊 Instruction Summary

| Category   | Instructions                                            |
| ---------- | ------------------------------------------------------- |
| Utility    | `NOP`, `MOV`, `CLP`, `ABS`, `NEG`                       |
| Math       | `ADD`, `SUB`, `MUL`, `DIV`, `MOD`, `POW`, `SQRT`, `LOG` |
| Trig       | `SIN`, `COS`, `TAN`, `ASIN`, `ACOS`, `ATAN`, `ATN2`     |
| Comparison | `GT`, `GTE`, `LT`, `LTE`, `EQ`, `NEQ`                   |
| Logic      | `AND`, `OR`, `XOR`, `NOT`                               |
| Flow       | `JMP`, `JEQ`, `JNE`, `JGT`, `JGE`, `JLT`, `JLE`         |
| Helpers    | `MIN`, `MAX`, `AVG`, `SGN`, `LERP`                      |

---

## 🔧 Utility Instructions

| Op    | Args | Description    |
| ----- | ---: | -------------- |
| `NOP` |    0 | No operation   |
| `MOV` |    2 | Copy value     |
| `CLP` |    4 | Clamp value    |
| `ABS` |    2 | Absolute value |
| `NEG` |    2 | Negate         |

---

## ➕ Math Instructions

| Op                                | Args |
| --------------------------------- | ---: |
| `ADD`, `SUB`, `MUL`, `DIV`, `MOD` |    3 |
| `POW`                             |    3 |
| `SQRT`, `LOG`                     |    2 |

---

## 🔺 Trigonometry

| Op                     | Args |
| ---------------------- | ---: |
| `SIN`, `COS`, `TAN`    |    2 |
| `ASIN`, `ACOS`, `ATAN` |    2 |
| `ATN2`                 |    3 |

---

## ⚖️ Comparison

Outputs `1` or `0`.

| Op                                    | Args |
| ------------------------------------- | ---: |
| `GT`, `GTE`, `LT`, `LTE`, `EQ`, `NEQ` |    3 |

---

## 🔀 Logic

| Op                 | Args |
| ------------------ | ---: |
| `AND`, `OR`, `XOR` |    3 |
| `NOT`              |    2 |

---

## 🔁 Flow Control

| Op                                       | Args | Notes        |
| ---------------------------------------- | ---: | ------------ |
| `JMP`                                    |    1 | Always jumps |
| `JEQ`, `JNE`, `JGT`, `JGE`, `JLT`, `JLE` |    3 | Conditional  |

Backward jump → execution halts for current tick.

---

# 🧮 Registers

---

## 🟢 General Purpose

```
GP00 … GP99
```

* Read/write
* Temporary storage

---

## 🚀 Ship Registers (`V`)

| Register     | Description              |
| ------------ | ------------------------ |
| `VXP`, `VYP` | Position                 |
| `VXV`, `VYV` | Velocity                 |
| `VRT`        | Rotation                 |
| `VAV`        | Angular velocity         |
| `VMS`        | Mass                     |
| `VTH`        | Thrust capacity          |
| `VPD`        | Power draw               |
| `VPG`        | Power generation         |
| `VPR`        | Power reserve            |
| `VHT`        | Avg heat                 |
| `VOX`        | Oxygen                   |
| `VHP`        | Avg integrity            |
| `VJM`        | Jump trigger (write)     |
| `VJD`        | Jump destination (write) |

---

# 🔌 Register Addressing Convention

```
[Category][Type][Property][Channel]
```

Example:

```
WTR0
```

* `W` = Weapon
* `T` = Turret
* `R` = Rotation
* `0` = Channel

---

## 🔢 Channels

```
0 … 9
```

* Each component must have a unique channel
* Registers are namespaced by channel

---

# 🧩 Component Categories

---

## 🔫 Weapons (`W`)

### Subtypes

* `WT` Turret
* `WL` Laser
* `WM` Missile
* `WR` Railgun

### Common Registers

| Code | Meaning         |
| ---- | --------------- |
| `A`  | Actual angle    |
| `R`  | Requested angle |
| `F`  | Fire command    |
| `P`  | Power usage     |
| `H`  | Heat            |
| `N`  | Range           |
| `C`  | Cooldown        |
| `D`  | Damage          |

---

## 🛡 Shields (`S`)

### Subtypes

* `SR` Redirectable
* `SD` Directional
* `SB` Bubble

### Registers

| Code | Meaning        |
| ---- | -------------- |
| `A`  | Current angle  |
| `C`  | Command angle  |
| `W`  | Width          |
| `F`  | Field strength |
| `I`  | Integrity      |
| `P`  | Power          |
| `H`  | Heat           |

---

## ⚛ Reactors (`R`)

### Subtypes

* `RR` Fission (RBMK)
* `RT` Fusion (Toroidal)
* `RA` Antimatter

### Registers

| Code | Meaning                      |
| ---- | ---------------------------- |
| `F`  | Fission rate / reaction rate |
| `T`  | Turbine load                 |
| `P`  | Power output                 |
| `H`  | Heat                         |
| `R`  | Radiation                    |
| `S`  | Stability                    |
| `E`  | Efficiency                   |

---

## 🚀 Engines (`E`)

### Subtypes

* `EC` Chemical
* `EI` Ion
* `EP` Plasma

### Registers

| Code | Meaning             |
| ---- | ------------------- |
| `A`  | Actual thrust angle |
| `C`  | Command angle       |
| `T`  | Throttle            |
| `P`  | Power usage         |
| `H`  | Internal heat       |
| `X`  | External heat       |
| `F`  | Force output        |

---

## 📡 Detectors (`D`)

### Subtypes

* `DS` Ship detector
* `DV` Vitals detector
* `DA` Anomaly detector

### Registers

| Code     | Meaning      |
| -------- | ------------ |
| `X`, `Y` | Position     |
| `N`      | Range        |
| `A`      | Angle        |
| `V`      | Velocity     |
| `H`      | Hostility    |
| `L`      | Lock quality |
| `O`      | Oxygen       |
| `R`      | Radiation    |

---

## 📦 Logistics (`L`)

### Subtypes

* `LS` Storage
* `LM` Manipulator
* `LP` Processor

### Registers

| Code | Meaning           |
| ---- | ----------------- |
| `F`  | Fill level        |
| `C`  | Capacity          |
| `M`  | Stored mass       |
| `T`  | Resource type     |
| `R`  | Reserved amount   |
| `I`  | Input rate        |
| `O`  | Output rate       |
| `E`  | Extend command    |
| `G`  | Grip command      |
| `L`  | Load held         |
| `S`  | Status            |
| `D`  | Target direction  |
| `Y`  | Recipe id         |
| `Q`  | Craft progress    |
| `P`  | Power usage       |

---

## 🤖 Drones (`DR`)

### Registers

| Code | Meaning        |
| ---- | -------------- |
| `C`  | Command (mode) |
| `N`  | Active drones  |
| `M`  | Max drones     |
| `R`  | Control range  |
| `L`  | Active tasks   |
| `Q`  | Task queue     |
| `T`  | Target ID      |
| `S`  | Status         |
| `P`  | Power usage    |
| `H`  | Health         |

---

## 🧠 Memory Banks (`MB`)

```
MB00 … MB99
```

Persistent storage between ticks.

---

# 🧩 Upgrade Examples

---

## Direct Upgrades

* Reactor → higher output, less heat
* Engine → more thrust
* Shield → higher integrity

---

## Tradeoffs / Variety

* High power / high heat reactor
* Low power / ultra-stable reactor
* Wide vs focused shields
* Fast vs efficient engines
* Long-range vs high-damage weapons

---

# 🧠 Register Naming Conventions

---

## Shared Meanings

| Code | Meaning                                  |
| ---- | ---------------------------------------- |
| `A`  | Actual value                             |
| `C`  | Command value                            |
| `P`  | Power                                    |
| `H`  | Heat                                     |
| `F`  | Output / Force / Fire (context-specific) |
| `N`  | Range                                    |
| `S`  | Status                                   |
| `I`  | Integrity / Input                        |
| `T`  | Throttle / Target                        |
| `R`  | Requested / Rate                         |

---

## Context-Specific Differences

| Code | Weapons   | Engines  | Shields        |
| ---- | --------- | -------- | -------------- |
| `F`  | Fire      | Force    | Field strength |
| `T`  | Targeting | Throttle | —              |
| `A`  | Angle     | Angle    | Angle          |

---

# 🧪 Example Scripts

---

## 🔫 Auto-Turret

```arch
JEQ DSH0 0 :SAFE
JLE DSN0 WTN0 :FIRE

:SAFE MOV 0 WTF0
JMP :END

:FIRE MOV DSA0 WTR0
MOV 1 WTF0

:END NOP
```

---

## ⚛ Reactor Stabiliser

```arch
JGT RRH0 900 :COOL
JLT VPR 100 :BOOST

MOV 50 RRF0
MOV 50 RRT0
JMP :END

:BOOST MOV 80 RRF0
MOV 70 RRT0
JMP :END

:COOL MOV 20 RRF0
MOV 90 RRT0

:END NOP
```

---

## 🛡 Shield Auto-Redirect

```arch
JEQ DSH0 0 :IDLE

MOV DSA0 SRC0
MOV 60 SRW0
MOV 100 SRF0
JMP :END

:IDLE MOV 0 SRC0
MOV 360 SRW0
MOV 40 SRF0

:END NOP
```

---

## 🌡 Heat Safety System

```arch
JGT VHT 700 :STOP

MOV 1 WTF0
JMP :END

:STOP MOV 0 WTF0

:END NOP
```

---

## 🚀 Engine Control

```arch
MOV GP00 ECC0

JGT ECH0 600 :LOW
MOV 100 ECT0
JMP :END

:LOW MOV 30 ECT0

:END NOP
```

---

## 🧠 Memory Example

```arch
# store last known target range
MOV DSN0 MB00
```

---

# 🎯 Design Principles

* Deterministic, predictable execution
* Limited complexity → encourages creativity
* Physical + logical systems intertwined
* Clear separation:

  * **Read (state)**
  * **Write (command)**

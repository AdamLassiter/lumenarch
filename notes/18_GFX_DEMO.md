# 18_GFX_DEMO

## Goal

Give **LUMEN//ARCH** a first strong layer of systemic, readable visual effects without changing the underlying deterministic simulation model.

This slice should make the ship feel alive:

- reactors should visibly pulse
- engines should visibly burn
- EVA movement should feel propulsive
- welding should feel active and legible
- sectors should stop feeling like flat colored voids

The focus is not “cinematic spectacle” in the abstract. The focus is readable, game-state-driven effects that reinforce what the simulation is already doing.

## Why This Slice Next

The game now has enough simulation depth that many actions are mechanically meaningful but visually understated.

Examples:

- reactor output and heat are important, but reactors still look mostly static
- engines move the ship, but their thrust is visually weak
- EVA suit movement changes handling, but doesn’t yet read as a different traversal mode
- welding/extraction are important player verbs, but lack convincing feedback
- sector space is mechanically varied, but the background is still simple and repetitive

This slice turns existing mechanics into stronger feedback.

## Demo Pitch

The same ship and encounters should feel much more expressive with no new core mechanics:

- the reactor glows and pulses according to state
- engines throw flame and impulse plumes
- EVA suit thrust reads instantly from the avatar
- welding emits sparks and a progress bar while work is happening
- the backdrop becomes a seeded spacefield with stars, haze, and galaxy structures that vary by sector

The player should feel that the world is not only simulated but *visible*.

## Scope

### 1. Component Overlay Effects

Add lightweight, state-driven overlays to existing components:

- **reactor**
  - pulsing green glow
  - pulse amplitude tied to reaction rate / heat / output
- **engines**
  - rear flame plume while thrusting
  - plume intensity tied to engine power / thrust demand
- **player EVA suit**
  - small back-thruster flames while accelerating in EVA
- **welder repair / extraction**
  - sparks while active
  - repair/extraction progress bar

These should be built as overlays or effect children on existing world entities rather than replacing the current sprites.

### 2. Deterministic-Enough Presentation Rules

These effects are presentation-only and do **not** need rollback authority.

Rules:

- effect spawning/toggling should derive from already-authoritative runtime state
- no gameplay outcome depends on the effect
- timing may use local presentation time, but the decision to show an effect should come from real simulation state

### 3. Procedural Space Backdrop

Replace the current solid sector/encounter background treatment with a shader-driven starfield / galaxy backdrop.

Requirements:

- seeded pseudorandom star distribution
- large-scale haze / galaxy shapes
- subtle parallax or depth variation if affordable
- parameters can vary by sector or encounter

This should support authored variation such as:

- cold sparse sectors
- bright dust-heavy salvage fields
- stormlike electrical haze
- quieter calibration/test regions

### 4. Sector / Encounter Background Parameters

Add a small authored parameter surface for backdrop variation.

Likely examples:

- background seed
- star density
- nebula density
- haze tint
- galaxy arc strength
- dust brightness
- scroll/parallax strength

These should live in authored data rather than hardcoded per-screen constants.

## Architecture Direction

### Presentation Layer

Preferred structure:

- new gameplay/presentation helpers or systems for effect sync
- one or more reusable material/shader assets
- minimal new ECS state, mostly markers and effect-child entities

Likely areas:

- `src/gameplay/spawn/`
- `src/gameplay/systems/visual*`
- `assets/shaders/`
- `src/state/sector.rs` or adjacent authored config for backdrop parameters

### Worldspace Rule

These effects should live in real worldspace where appropriate:

- engine flames attach to engine modules
- reactor glow attaches to reactor modules
- suit thrust attaches to player actors
- sparks attach to the current interaction site

The backdrop should remain presentation-only and camera-driven, not part of gameplay collision or field logic.

## Risks / Design Gaps

- particle-heavy effects can get noisy quickly, so readability matters more than count
- engine/reaction visuals should reflect module orientation correctly
- progress bars for repair/extraction need to read well over bright or dark ship art
- shader parameter ownership needs to be decided cleanly:
  - sector-level
  - node-level
  - encounter-level

## Success Criteria

- the player can identify reactor stress, thrust, EVA propulsion, and welding activity at a glance
- repair/extraction progress is visible without reading text
- the backdrop gives sectors stronger mood and identity
- no gameplay logic depends on shader/effect state

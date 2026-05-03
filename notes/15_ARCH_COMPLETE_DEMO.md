# ARCH_COMPLETE_DEMO — Fifteenth Vertical Slice

## Goal

Complete the first fully authored, text-driven **ARCH** workflow for **LUMEN//ARCH**.

This slice should turn the current partially-structured programming tools into a real programmable engineering loop:

`Inspect Registers -> Edit Text Program -> Parse / Validate -> Launch / Run -> Diagnose Errors Or Behavior -> Rewrite`

It should also complete the missing component-channel model that both `ARCHLANG` and `LUMENLANG` already assume.

This slice validates that:

* component channels are a real gameplay concept rather than doc-only fiction
* players can author programs as text, not only through template/cycling UI
* invalid programs fail clearly and recoverably
* component UIs expose the real register surface in a readable way
* ARCH feels like a complete direct-control language layer
* the same text-authoring foundation can later support richer LUMEN editing without duplicating tooling

## Candidate Directions Considered

Before finalizing the slice, there are three obvious directions:

### Option 1 — Channel Surface Slice

Focus only on:

* per-component channels
* register naming visibility
* better component/control UIs

Why it is attractive:

* closes an important spec/implementation gap quickly
* improves readability of both manual control and future programming

Why it is not enough:

* leaves program authoring still stuck in template-style editing
* does not complete the core “write code in-game” promise

### Option 2 — Text Programming Slice

Focus only on:

* text editor
* parser
* validation
* runtime error display

Why it is attractive:

* gets the game closer to a real code-authoring identity

Why it is not enough:

* without channels and register-readable component UIs, the text layer is still disconnected from how components are understood and addressed in play

### Option 3 — Complete ARCH Implementation Slice

Combine:

* component channels
* register-readable UIs
* in-game ARCH/LUMEN text editing
* parsing and error handling
* runtime explainability

Why this is the right choice now:

* the current implementation gap is systemic, not isolated
* channels, readable registers, and text authoring all depend on the same canonical control surface
* this produces a coherent milestone instead of another partial programming pass

## Finalized Direction

This slice should be implemented as **ARCH_COMPLETE_DEMO**.

That means:

* ARCH becomes a complete playable direct-control programming layer
* LUMEN gains the shared text-authoring/parsing foundation it will also need
* channels become a real player-facing part of ship design and mission-time interaction

## Why This Slice Now

The project already has:

* register-backed runtime control surfaces
* multiplayer-safe deterministic simulation
* component-local UIs
* basic structured ARCH/LUMEN editing
* runtime automation execution

But it still lacks the pieces that make the programming stack feel finished:

* no real per-component channel workflow
* no readable register names at the point of interaction
* no text editing workflow while using computers
* no parser-driven authoring loop
* no strong invalid-program UX

Without these, the game still demonstrates automation, but not the “ritualistic engineering through code” identity described in the concept.

## Demo Pitch

The player outfits a ship with channel-assigned components, boards their vessel, inspects real register surfaces at runtime, and authors or repairs ARCH programs directly through a shipboard processor interface.

While operating a computer:

* they can open a compact text editor
* edit a bounded number of lines
* parse the result into an ARCH or LUMEN program
* see line-level errors and invalid operands
* return to the ship and immediately observe the consequences

Manual and automated control should now feel like two views over the same system:

* component UIs show real register names and values
* channels are visible and configurable
* programs address the same surfaces the player reads manually

## In Scope

### Channels As Real Ship State

* every placed channel-capable component gets a configurable channel
* channels can be edited:
  * in the ship editor
  * during encounters while interacting with the component
* channel values are visible in component UIs
* channel assignment is saved with ship definitions
* enemy editor keeps effectively unlimited access to channels and parts, but uses the same underlying model

### Register Legibility

* component-specific UIs show register names next to values
* readable/writable distinction is visible where practical
* ARCH-facing register names match the actual authoring language surface
* the player can inspect a component UI and understand what program-visible data/control exists

### Text Authoring UI

* a generic bounded multi-line text editor using the existing textbox foundation
* visible cursor, focus, navigation, selection basics, copy/cut/paste semantics
* used while interacting with:
  * ARCH computers
  * LUMEN processors
* supports a fixed number of lines in v1
* supports line-focused editing, not a full freeform IDE

### Text Parsing And Validation

* parser from text to ARCH programs
* parser from text to LUMEN programs
* deterministic validation and error reporting
* structured errors for:
  * unknown opcodes
  * malformed labels
  * wrong argument counts
  * invalid registers
  * invalid channels
  * invalid literals
  * unsupported constructs

### Runtime Error Handling

* invalid authored programs do not crash or silently rewrite themselves
* parse errors block program activation and remain visible to the player
* runtime execution errors remain surfaced clearly
* the player can distinguish:
  * parse failure
  * validation failure
  * runtime halt/error

### Shared Editing Foundation For ARCH And LUMEN

* one editor shell
* two parsers / language modes
* language-specific help, validation, and summaries
* enough shared structure that future programming improvements do not fork the UI again

## Explicitly Out Of Scope

* full final freeform code IDE polish
* syntax highlighting beyond simple state/error emphasis
* unlimited file-sized programs
* networked collaborative editing
* late-game LUMEN feature completion beyond what is needed to support text parsing and error handling

## Design Gaps This Slice Must Resolve

### 1. Channel Semantics Are Specified But Not Implemented

`ARCHLANG` and `LUMENLANG` already assume channels exist and matter.

This slice must answer:

* which component kinds are channel-addressable
* whether channel uniqueness is enforced globally, per family, or not at all
* whether channels are only `0..9` as in docs, or a wider internal space
* how duplicate channels are presented if allowed

Recommended v1 rule:

* channels are `0..9`
* many component families may share a channel intentionally
* the UI warns about shared channels but does not forbid them

That matches the concept’s tactical use of channels more naturally than forced uniqueness.

### 2. Register Surface Needs Canonical Naming

Current runtime/UI state is richer than the original early ARCH slice, but not all surfaces are yet presented with canonical author-facing names.

This slice must define:

* the exact register names shown in component UIs
* which names are read-only vs writable
* which names appear in ARCH vs LUMEN
* which current runtime variables are intentionally not exposed

### 3. Text Editing Needs A Deterministic Program Model Boundary

The editor should not mutate runtime execution state line by line as the player types.

This slice should define a clean boundary:

* text buffer is local editor state
* parse/validate produces structured program state
* only successful apply/commit updates the authored program on the component/module

That avoids partially valid text corrupting saved programs or causing unstable runtime behavior.

### 4. Runtime Editing Scope Must Be Explicit

The request includes editing while interacting with an ARCH or LUMEN processor in-game.

This slice should define:

* whether in-mission edits are allowed live
* whether they apply instantly or only when “committed”
* whether editing pauses processor execution while the draft is invalid

Recommended v1 rule:

* the player may draft edits live
* the old compiled program remains active until the new text parses and is explicitly committed
* commit failure keeps the old active program and shows errors

### 5. Error UX Needs To Be Engineering-Facing, Not Just Dev-Facing

The player needs actionable errors, not only logs or enum dumps.

Error UI should answer:

* what line failed
* what token/register/opcode was invalid
* what the parser expected
* whether the previous valid program is still active

### 6. Rollback / Multiplayer Implications Must Be Deliberate

Programming changes during synchronized play are gameplay-relevant state.

This slice must explicitly decide:

* whether in-mission program edits are host-only in current multiplayer
* whether they are synchronized immediately through rollback-owned actions
* whether player-editor ship programming remains host-authored during non-encounter phases

Recommended v1 rule:

* docked/editor program changes remain host-authored synchronized state
* in-encounter live computer edits are local authoring drafts until committed
* commit of a live program change is a synchronized action if allowed in multiplayer

If that is too large for one implementation pass, the slice should still design the path and constrain the first shipped behavior clearly.

## Relation To Existing Slices

This slice is the direct successor to the earlier ARCH and LUMEN slices:

* [04_ARCH_DEMO.md](/home/adaml/code/lumenarch/notes/04_ARCH_DEMO.md)
* [13_LUMEN_DEMO.md](/home/adaml/code/lumenarch/notes/13_LUMEN_DEMO.md)

Those slices established:

* programs as ship content
* deterministic execution
* starter editing flows
* runtime explainability basics

This slice completes the missing authoring, channel, and validation loop that makes the stack feel finished rather than prototype-shaped.

## Success Criteria

The slice is successful when:

* the player can assign and inspect channels in both editor and gameplay
* the player can read canonical register names directly in component UIs
* the player can open an ARCH/LUMEN processor and edit a bounded text program
* the text parses into a real program representation
* invalid programs fail with clear, line-level feedback
* valid programs can be committed and immediately observed in runtime behavior
* ARCH feels like a complete direct-control language layer rather than a template system with hidden structure

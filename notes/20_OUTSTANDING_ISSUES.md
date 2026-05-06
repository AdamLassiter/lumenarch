# 20_OUTSTANDING_ISSUES

This is not a copy of every TODO item. It is the shorter list of:

* real cross-cutting gaps still visible in the codebase
* places where notes disagree with current implementation
* issues worth keeping in view during the next iteration

## High-Value Remaining Work

### 1. Tuning And Playtest Debt

Multiple slices are implemented but still explicitly waiting on live tuning rather than code invention:

* `02_SYSTEMS_DEMO_TODO`
* `03_LOGISTICS_DEMO_TODO`
* `05_COMPONENTS_DEMO_TODO`
* `06_TRAVEL_DEMO_TODO`
* `07_HOSTILE_SHIPS_DEMO_TODO`
* `08_BOARDING_DEMO_TODO`
* `09_BREACH_DEMO_TODO`
* `10_UI_DEMO_TODO`
* `13_LUMEN_DEMO_TODO`

This is now one of the biggest real risks in the repo: the prototype has more implemented systems than it has validated game feel.

### 2. Multiplayer / Rollback Cleanup Is Still In Progress

`12_MULTIPLAYER_DEMO_TODO` is correct that the architecture works but is not fully closed out.

The main remaining gaps are:

* deterministic player-editor mutations still need fuller rollback-native treatment
* some gameplay paths still assume local presentation ownership more than ideal
* regression coverage is better than before, but still narrow relative to the total gameplay surface

### 3. Programming Docs Need A Real Alignment Pass

`15_ARCH_COMPLETE_DEMO_TODO` still reflects real work.

The docs exist under `docs/src/`, but they still need an implementation-alignment pass for:

* actual ARCH parser/editor behavior
* actual LUMEN syntax and limits
* explicit documentation of deferred features and current editor workflow

### 4. Content Pressure Is Behind Mechanics

Several gameplay systems now exist without enough scenario content to prove their value:

* `14_PLAYER_DEMO_TODO`: suit-specific pressure scenarios
* `16_STATIONS_DEMO_TODO`: more station variety and deeper docked consequences
* `17_ROGUE_CONTINUANTS_DEMO_TODO`: more enemy identity range and comms outcomes

The codebase is ahead of the authored content here.

## Active Bugs / Risks Worth Keeping Visible

These remain credible current issues after the sweep:

* `notes/99_PROBLEMS.md`: atmospheric decompression direction still likely needs correction toward space-facing leakage rather than a tile corner
* `notes/99_PROBLEMS.md`: docked/editor ship preview transform retention still needs confirmation

## Codebase Cleanup Follow-Up

This sweep removed disconnected older-revision editor/program UI code and several dead helper paths, but some intentionally forward-looking code remains unused.

That remaining unused code mostly falls into three categories:

* component/runtime fields reserved for balancing or future systemic use
* presentation helpers that are not yet reintroduced after UI reshaping
* small utility methods in progression/station/frontend state that are plausible future hooks

Those should be revisited opportunistically, not blindly deleted.

## Suggested Next Pass

If the goal is highest-value progress rather than more cleanup, the strongest next pass is:

1. run a focused playtest/tuning round across travel/combat/boarding/logistics
2. fix the known decompression and preview-state issues if still reproducible
3. update `docs/src/ARCHLANG.md` and `docs/src/LUMENLANG.md`
4. expand behavior-level regression tests around multiplayer/editor/program flows

# AGENTS.md

## Summary

LUMEN//ARCH is a top-down multiplayer 2D sci-fi systems game about building a patchwork ship, surviving inside it, and gradually moving from direct hands-on control to guided autonomy.

## Project Constraints

* Rust 2024, Bevy 0.18, `bevy_ggrs`/`ggrs`, `cordic`, and `fixed`.
* Rollback simulation must remain deterministic. Gameplay state that affects clients belongs in rollback-aware resources/components; pure presentation UI can stay client-side.
* Simulation math should use fixed-point helpers/types. Keep `f32`/Bevy transform work at presentation boundaries unless the surrounding code already does otherwise.
* Ship infrastructure is physical and strict: power, oxygen ducts, and typed resource pipes should not silently fall back to global pools.
* Ship systems should expose ARCH-readable/writeable registers when they add meaningful automation or monitoring surface.

## Documentation and Notes

* [docs](docs/src) contains high-level game direction.
* [notes](notes) records vertical slices and TODOs. Add/update notes when implementing a new slice.

## Code Organization

* Prefer flat, state-oriented modules like `gameplay`, `docked`, `sector_map`, `lobby`, and `editor`.
* Keep component definitions, one-shot spawning, runtime systems, and UI presentation separated when a file grows large.
* Every registered Bevy system/run condition should have a function-level comment explaining what it does and why it exists in the schedule.
* Any `ParamSet<(...)>` use must include a nearby `SAFETY:` comment explaining why the branches cannot double-mutably access the same entity/component.

## Testing

* [sim_tests.rs](src/sim_tests.rs) uses the deterministic app flow for larger session/regression coverage.
* Use focused unit tests for deterministic graph/planner/parser behavior where possible.

## Extra Tools

* `cargo fmt` and `cargo test` are the default verification path.
* `cargo bevy-check` can help diagnose Bevy B0001/B0002 query conflicts.

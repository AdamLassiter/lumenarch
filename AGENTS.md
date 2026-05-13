# AGENTS.md

## Summary

LUMEN//ARCH is a top-down multiplayer 2D sci-fi systems game about building a patchwork ship, surviving inside it, and gradually moving from direct hands-on control to guided autonomy.

## Technical Stack

* Rust latest (currently 1.95) and Bevy latest (currently 0.18) - this is 'greenfield' and new features are a nice convenience
* Rollback netcode through `bevy_ggrs` and `ggrs` - the game simulation is deterministic except player input
* Fixed-point math through `cordic` and `fixed` - keep game simulations consistent on different systems without floating-point math concerns
* UI presentation is not subject to rollback as it is purely a client-side concern, but the states that may drive it across multiple connected clients should be handled with rollback in mind.

## Documentation and Notes

* The [docs](docs/src) directory contains high-level descriptions for the direction of the game
* The [notes](notes) directory contains implementation details for vertical slices as they have been implemented, along with associated TODO lists
  * New vertical slices should be documented here, and TODO files updated once tasks have been completed

## Code Style and Cross-cutting Concerns

* Source files should rarely exceed 1000 lines - this might indicate a refactor is due
  * Flat-and-wide is the preferred style
    * Top-level modules generally covering different high-level states the game can be in
    * Components, one-shot spawing and runtime systems are the next heirarchy to consider splitting refactors over (see [gameplay](src/gameplay) for an example)
* Use of `ParamSet<(...)>` in queries is convenient compared to large sets of `Without<T>` labels, but use of these should be considered comparable to `RefCell` or dubious `.unwrap()` calls and warrants `// SAFETY ...` comments for mutability guarantees

## Testing

* [sim-tests.rs](src/sim_tests.rs) leverage the deterministic nature of the game and can test for larger

## Gameplay Implementation

* 'Components' on the ship should expose read/write or readonly 'registers' for the 'ARCH' computer system in-game to interact with
  * This enables the player to automate and monitor ship systems

## Extra Tools

* The usual suite of `cargo check`, `cargo fmt` and `cargo test` are available but not required for every small change
* `cargo bevy-check` can be used to check for Bevy B0001 and B0002 error candidates and will provide diagnostics on possible fixes

# 17_ROGUE_CONTINUANTS_DEMO TODO

## Current Status

This slice has started, but is not complete.

Implemented in this pass:

- hostile enemy entries now carry faction/name/captain/comms metadata
- seeded hostile ships are framed as rogue Continuant crews
- encounter setup now forwards opposition identity into mission framing
- contract launch and mission debrief text can refer to named opposition

## Completed

- [x] Add basic faction and ship-identity metadata to enemy ship entries
- [x] Add captain/comms identity for seeded rogue hostile ships
- [x] Surface opposition identity in encounter mission framing
- [x] Surface rogue opposition in contract-backed debrief text

## Remaining

- [ ] Add non-rogue hostile identity variants, especially clearer Null Swarm authored encounters
- [ ] Add richer encounter comms timing instead of static single-line framing
- [ ] Add station reaction or reputation consequences tied to specific rogue crews
- [ ] Add surrender / retreat / distress outcomes
- [ ] Add more than two named hostile archetypes and more authored personality range
- [ ] Add explicit regression tests covering hostile identity propagation into encounter and debrief UI

## Notes

- This slice currently uses authored metadata, not full hostile human actor simulation.
- The current implementation is meant to give the world social texture quickly while staying compatible with the existing rollback combat loop.

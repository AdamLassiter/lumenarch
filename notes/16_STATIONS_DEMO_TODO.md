# 16_STATIONS_DEMO TODO

## Current Status

This slice is substantially implemented.

The repo now has:

- station authored data in `saves/stations.json`
- a loaded station catalog
- a docked hub with shipyard / quartermaster / contracts / lore surfaces
- host-synchronized contract accept / launch actions
- progression-backed station/contact/lore/contract memory
- contract-specific launch/debrief framing

## Completed

- [x] Add authored station data and create a default `needle_rest` station
- [x] Add shared station-related campaign state to `DemoProgression`
- [x] Associate the starting hub node with a station id
- [x] Rework the docked UI into a station hub with multiple surfaces
- [x] Add contract browsing to the docked flow
- [x] Route contract acceptance through rollback meta ops
- [x] Route contract launch through rollback meta ops
- [x] Apply contract rewards and lore unlocks on mission return
- [x] Persist station/campaign memory through the existing campaign save

## Remaining

- [ ] Add more than one inhabited station and route-specific station variety
- [ ] Give quartermaster actions deeper systemic meaning beyond repair/readout
- [ ] Replace the generic sector-map-first loop more thoroughly with station-issued work
- [ ] Add explicit docked debrief acknowledgement / inbox flow
- [ ] Add more mission artifacts or recovered logs that unlock lore outside contracts
- [ ] Add stronger regression coverage for:
  - host/client synchronized contract accept
  - host/client synchronized contract launch
  - lore/contact persistence through save/load

## Notes

- Surface selection and lore browsing are intentionally local presentation state.
- Progression-affecting docked actions remain host-authored and rollback-backed.
- This slice is intentionally UI-first; explorable station interiors remain out of scope.

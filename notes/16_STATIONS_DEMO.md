# 16_STATIONS_DEMO

## Goal

Make docked play feel like arriving somewhere inhabited and useful, rather than merely pausing between encounters to open a refit screen.

This slice keeps stations UI-first. We do not add free-roam station interiors. Instead, the docked screen becomes a shared hub surface with authored contacts, contracts, station flavor, and lore that all sit on top of the existing rollback-backed campaign state.

## What This Slice Covers

- Authored station data in `saves/stations.json`
- Station identity:
  - name
  - faction
  - short flavor text
  - contacts
  - services
  - lore entries
  - contracts
- A docked hub loop with four surfaces:
  - shipyard
  - quartermaster
  - contract board
  - lore terminal (archives)
- Host-authored synchronized contract actions:
  - accept contract
  - launch contract
- Persistent campaign memory for:
  - known stations
  - unlocked contacts
  - unlocked lore
  - completed contracts
  - active contract
- Contract-backed launch and contract-specific debrief text

## Data Model

The station layer is authored in `saves/stations.json` and loaded into a shared station catalog.

Important types:

- `FactionId`
- `StationDefinition`
- `StationContact`
- `StationService`
- `StationContract`
- `LoreEntry`
- `StationCatalog`

Shared progression state now remembers station/campaign knowledge through `DemoProgression`, which remains rollback-owned and save-backed through the campaign save.

## Multiplayer Model

The inhabited station slice follows the existing docked synchronization rule:

- host may issue progression-changing docked actions
- clients observe the shared station state
- local lore browsing and surface selection remain presentation-side

The synchronized actions are routed through rollback meta ops rather than direct docked UI mutation.

## User Experience

Docking at a hub should now answer:

- where am I?
- who runs this place?
- who is talking to me?
- what job am I taking?
- why does this mission matter to the station?

The station screen now uses a single hub view with multiple surfaces instead of treating docked play as only a refit/repair launcher.

## Relationship To The Existing Loop

This slice does not replace the existing sector graph or encounter runtime. Instead, it wraps them:

- station contracts target existing sector nodes
- launching a contract still enters the existing encounter loop
- mission return still lands back in the docked phase
- debrief text now reflects station and contract framing

## Follow-On

This slice deliberately sets up `17_ROGUE_CONTINUANTS_DEMO`.

Once stations have contacts and contracts, hostile encounters can start feeling like conflicts inside a lived world rather than abstract combat problems. That next slice extends the same authored framing outward into enemy ships, comms, and consequences.

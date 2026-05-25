# 23_DOCKED_SPACEPORT_DEMO

## Goal

Replace the docked menu with a walkable station or spaceport scene where the player ship is physically docked.

Docked play should feel like arriving somewhere inhabited. Players should be able to leave or stand near their ship, move through a station interior, focus an NPC or relevant ship component, press `E`, and open a local overlay for dialogue or service actions.

## What This Slice Covers

- Station layout authoring for spaceports and station interiors
- Airlock-based docking between the player ship and the station
- NPC interactables placed in the docked station scene
- Yarn-backed NPC dialogue through `bevy_yarnspinner`
- Engineer or repair NPC services for existing repair and refit actions
- Contract, archive, and other dialogue NPCs
- Moving `Open sector map` to the ship cockpit or helm while docked

## Data Model

The station catalog remains the source of authored station data.

This slice extends station data in `saves/stations.json` with:

- station layout tiles
- a docking anchor
- NPC definitions
- NPC grid positions and facing
- Yarn dialogue start nodes
- optional service actions that dialogue choices can trigger

Spaceports are not full simulated ships in this slice. They are authored walkable docked layouts with interactable NPCs and service points.

## Editor Model

Station and spaceport authoring should use a new station editor mode that reuses the existing grid editor patterns.

This keeps enemy ship editing separate while allowing station authors to place:

- floor and wall tiles
- docking anchor
- NPCs
- service markers or NPC service metadata

The player ship editor must also validate that the ship has at least one `Airlock`, because Airlock modules are the first docking connector model.

## Multiplayer Model

NPC dialogue overlays are local per player.

Players may talk to NPCs independently without changing rollback state. Shared effects still flow through rollback-safe meta commands, such as:

- repair ship
- open refit editor
- accept contract
- launch contract
- open sector map

This keeps dialogue presentation flexible while preserving deterministic shared game state for meaningful actions.

## Relationship To 16_STATIONS_DEMO

This slice supersedes the UI-first station hub from `16_STATIONS_DEMO`.

The existing station concepts remain:

- station identity
- contacts
- contracts
- lore
- repair and refit services
- campaign progression

The presentation changes from a tabbed docked menu to a physical walkable docked scene. Existing contracts, lore, repair, and refit behavior should be preserved but reached through NPC dialogue or cockpit/helm interaction instead of direct docked menu buttons.

## Assumptions

- Airlock is the required docking connector for now.
- No new `DockingPort` module is added in this slice.
- Spaceports do not simulate oxygen, power, pipes, weapons, or damage as ships.
- `bevy_yarnspinner = "0.8"` is used for Bevy 0.18 compatibility.
- Dialogue state is local unless a selected option explicitly emits a rollback meta command.

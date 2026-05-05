# 17_ROGUE_CONTINUANTS_DEMO

## Goal

Make at least some hostile encounters feel like confrontations with other scavengers and rival Continuant cells, not only abstract automated threats.

This slice does **not** add full hostile onboard human crew simulation. It starts with identity, comms, and consequences.

## What This Slice Covers

- Named hostile ship identity
- Captain/cell identity
- Faction framing
- Encounter comms flavor
- Contract and debrief framing that distinguishes rogue Continuants from anonymous machine hostility

## Chosen Shape

The first pass treats enemy ships as crewed characters through authored metadata rather than through new hostile actor simulation.

Important surfaces:

- station contract briefing
- encounter top-banner / contact framing
- mission return / debrief text

## Data Model

Enemy ship entries can now carry authored identity fields such as:

- faction
- ship name
- captain name
- intro comms
- outro comms
- crewed flag

This gives the game enough structure to distinguish:

- Continuant stations
- Rogue Continuants in the field
- Null Swarms and other non-human opposition

## Relationship To `STATIONS_DEMO`

Stations and Rogue Continuants are intentionally staged together:

- stations provide named contacts and reasons to take work
- rogue Continuants provide named opposition and social consequences

Without the station slice, rogue ships risk feeling like decorative labels. Without the rogue slice, stations risk handing out jobs into an empty-feeling world.

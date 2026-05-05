# 19_DRONES_DEMO TODO

## Goal

Implement the first logistics-focused drone layer as an extension of the current storage/manipulator/processor simulation.

## Phase 1 — Data And Control Surface

- [x] Add a `Drone Station` component/module kind or variant strategy.
- [x] Define first-pass drone runtime state:
  - idle
  - assigned
  - traveling to source
  - picking up
  - traveling to destination
  - delivering
- [x] Define the drone station control/readout surface.
- [x] Add ARCH-facing register/readout support aligned with the logistics doc.

## Phase 2 — Drone Entities

- [x] Add drone ECS entities with:
  - world/local position
  - simple movement
  - current task
  - carried resource payload
- [x] Spawn and clean up drones with the owning drone station.
- [x] Add visible drone art/markers for debugging and readability.

## Phase 3 — Endpoint And Task Model

- [x] Define valid drone endpoints:
  - storage
  - processor
  - airlock
  - other relevant logistics endpoints
- [x] Reuse the reservation model rather than inventing a parallel resource system.
- [x] Add first-pass task assignment rules.

## Phase 4 — Runtime Logistics Integration

- [x] Move intake salvage to storage by drone.
- [x] Move storage resources to processors by drone.
- [x] Move processor outputs back to storage by drone.
- [x] Keep manipulator and drone systems interoperable instead of mutually exclusive.

## Phase 5 — Constraints And Balance

- [x] Add operational radius / field constraint.
- [x] Add power usage and throughput tuning if needed.
- [x] Decide whether drone count is tied to station variant, equipment, or simple station cap.

## Phase 6 — UI And Feedback

- [x] Add drone station component UI.
- [x] Show active drone count, task summaries, and idle/bottleneck state.
- [x] Add debug or status text so players can tell what each drone is trying to do.

## Phase 7 — Regression Coverage

- [x] Add tests for reservation safety with multiple drones.
- [x] Add tests for deterministic task assignment.
- [x] Add tests for repeated encounter entry/exit / station spawn cleanup.

## Test Questions

- [x] Does a drone-equipped ship actually unblock layouts that manipulator-only ships struggle with?
- [x] Can players understand where drones are helping and where they are bottlenecked?
- [x] Do drones preserve the physical/logistical feel instead of making resources feel magical?

# 19_DRONES_DEMO TODO

## Goal

Implement the first logistics-focused drone layer as an extension of the current storage/manipulator/processor simulation.

## Phase 1 — Data And Control Surface

- [ ] Add a `Drone Station` component/module kind or variant strategy.
- [ ] Define first-pass drone runtime state:
  - idle
  - assigned
  - traveling to source
  - picking up
  - traveling to destination
  - delivering
- [ ] Define the drone station control/readout surface.
- [ ] Add ARCH-facing register/readout support aligned with the logistics doc.

## Phase 2 — Drone Entities

- [ ] Add drone ECS entities with:
  - world/local position
  - simple movement
  - current task
  - carried resource payload
- [ ] Spawn and clean up drones with the owning drone station.
- [ ] Add visible drone art/markers for debugging and readability.

## Phase 3 — Endpoint And Task Model

- [ ] Define valid drone endpoints:
  - storage
  - processor
  - airlock
  - other relevant logistics endpoints
- [ ] Reuse the reservation model rather than inventing a parallel resource system.
- [ ] Add first-pass task assignment rules.

## Phase 4 — Runtime Logistics Integration

- [ ] Move intake salvage to storage by drone.
- [ ] Move storage resources to processors by drone.
- [ ] Move processor outputs back to storage by drone.
- [ ] Keep manipulator and drone systems interoperable instead of mutually exclusive.

## Phase 5 — Constraints And Balance

- [ ] Add operational radius / field constraint.
- [ ] Add power usage and throughput tuning if needed.
- [ ] Decide whether drone count is tied to station variant, equipment, or simple station cap.

## Phase 6 — UI And Feedback

- [ ] Add drone station component UI.
- [ ] Show active drone count, task summaries, and idle/bottleneck state.
- [ ] Add debug or status text so players can tell what each drone is trying to do.

## Phase 7 — Regression Coverage

- [ ] Add tests for reservation safety with multiple drones.
- [ ] Add tests for deterministic task assignment.
- [ ] Add tests for repeated encounter entry/exit / station spawn cleanup.

## Test Questions

- [ ] Does a drone-equipped ship actually unblock layouts that manipulator-only ships struggle with?
- [ ] Can players understand where drones are helping and where they are bottlenecked?
- [ ] Do drones preserve the physical/logistical feel instead of making resources feel magical?

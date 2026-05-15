Place all ship tile sprites in this directory.

Art contract:

* Every tile sprite is `32x32` pixels.
* The client loads tiles from `assets/tiles/<kind>.png`.
* Rotation is provided per tile in the ship snapshot as `rotation_quadrants`.
* Rotation values are clockwise quarter-turns:
  * `0` = default/up-facing art
  * `1` = rotate 90 degrees clockwise
  * `2` = rotate 180 degrees
  * `3` = rotate 270 degrees clockwise
* Tiles that care about interior/exterior direction, such as hull edges, hull corners, airlocks, engines, cockpits, and turrets, should be painted with a clear "forward/outward" orientation in their default sprite.

Current tile set and intended purpose:

* `assets/tiles/core.png`
  Central command or structural heart of the ship. This is the anchor tile for the vessel and can read as a generic systems nexus.
* `assets/tiles/interior.png`
  Walkable internal ship space. This should read as deck plating or corridor floor rather than a major subsystem.
* `assets/tiles/cockpit.png`
  Piloting or bridge tile. Its facing should clearly indicate the "front" of the ship.
* `assets/tiles/computer.png`
  Computer or memory/control tile. This is the physical host for ARCH programs and should read as a console, rack, or control bank.
* `assets/tiles/processor.png`
  Logistics processing tile. This should read as an industrial refiner, fabricator, or salvage-processing machine.
* `assets/tiles/reactor.png`
  Main power generation tile. This should visually read as hazardous, industrial, or energy-dense.
* `assets/tiles/engine.png`
  Main propulsion tile. Default art should point thrust/exhaust outward so rotation can align it with the ship stern or side thrusters.
* `assets/tiles/cargo.png`
  Storage or logistics tile. This can read as crates, racks, or a cargo bay floor module.
* `assets/tiles/battery.png`
  Power storage or capacitor tile. Visually distinct from the reactor, more contained and less active.
* `assets/tiles/airlock.png`
  Exterior access tile for boarding, EVA, or docking. Default art should clearly indicate which side opens to space.
* `assets/tiles/hardpoint.png`
  Exterior weapon mount tile. Default art should have an obvious front direction for cosmetic rotation.
* `assets/tiles/turret.png`
  Rotating turret to affix to a hardpoint. Default art should have an obvious direction.
* `assets/tiles/hull.png`
  Straight hull edge tile used to define the exterior shell. Default art should be a straight boundary with exterior on one side and interior on the opposite side.
* `assets/tiles/hull_inner_corner.png`
  Concave hull corner used where interior space cuts into the hull line. Default art should clearly read as an inward corner.
* `assets/tiles/hull_outer_corner.png`
  Convex hull corner used where two exterior hull edges meet. Default art should clearly read as an outward corner.

Planned 21_TUBES_DEMO engineering-underlay sprites:

* `assets/tiles/floor.png`
  Walkable deck/floor foundation tile used on the underlay layer.
* `assets/tiles/wire.png`
  Power wiring route tile. It should read clearly as electrical infrastructure beneath components.
* `assets/tiles/duct_oxygen.png`
  Oxygen duct route tile for moving oxygen supply into breathable rooms.
* `assets/tiles/pipe_raw_salvage.png`
  Raw salvage pipe route tile.
* `assets/tiles/pipe_repair_charge.png`
  Repair charge pipe route tile.
* `assets/tiles/pipe_fuel.png`
  Fuel pipe route tile for reactor fuel delivery.
* `assets/tiles/pipe_ammunition.png`
  Ammunition pipe route tile for weapon supply.
* `assets/tiles/pipe_oxygen.png`
  Oxygen resource pipe route tile for moving stored oxygen before it becomes atmosphere.
* Route tiles may also provide adjacency-selected variants with `_end`, `_straight`, `_corner`, `_tee`, and `_cross` suffixes. The editor/runtime picks these according to connected cardinal neighbors of the same route type; `_end` is used for exactly one connection, while isolated routes use the base sprite.
* `assets/tiles/service_link.png`
  Gameplay-only cosmetic service link drawn dynamically from components to connected service-port route tiles. This is not placeable and has no collision.
* `assets/tiles/junction_box.png`
  Programmable electrical junction box. It should read as a closeable routing/control component.
* `assets/tiles/valve.png`
  Programmable pipe or duct valve. It should have a clear open/closed control identity.
* `assets/tiles/interior_wall.png`
  Interior wall overlay tile that blocks walking and atmosphere flow without acting like a major system component.
* `assets/tiles/o2_generator.png`
  Oxygen generator component that produces oxygen resource for ducts or storage.
* `assets/tiles/o2_canister_storage.png`
  Oxygen storage component for canisters or tanks.

Variant sprite expectations:

* The runtime currently falls back to family sprites such as `reactor.png` and `battery.png` for upgraded variants.
* Better-components variants should eventually receive dedicated art with the same `32x32` contract when available.
* Expected upgraded sprite names:
  * `assets/tiles/core_expanded.png`
  * `assets/tiles/cockpit_advanced_helm.png`
  * `assets/tiles/reactor_fission.png`
  * `assets/tiles/reactor_fusion.png`
  * `assets/tiles/cargo_fuel_tank.png`
  * `assets/tiles/cargo_ammo_rack.png`
  * `assets/tiles/battery_capacitor.png`
  * `assets/tiles/processor_fabricator_fast.png`
  * `assets/tiles/turret_laser.png`
  * `assets/tiles/turret_ballistic.png`
  * `assets/tiles/shield_radial.png`
  * `assets/tiles/shield_directional.png`
  * `assets/tiles/cargo_raw_salvage.png`
  * `assets/tiles/cargo_repair_charge.png`
  * `assets/tiles/cargo_o2_canister.png`

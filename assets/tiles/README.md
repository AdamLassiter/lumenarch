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

Place all ship tile sprites under the category that matches how the editor presents them.

Art contract:

* Every tile sprite is `32x32` pixels.
* Rotation is provided per tile in the ship snapshot as `rotation_quadrants`.
* Rotation values are clockwise quarter-turns:
  * `0` = default/up-facing art
  * `1` = rotate 90 degrees clockwise
  * `2` = rotate 180 degrees
  * `3` = rotate 270 degrees clockwise
* Tiles that care about interior/exterior direction, such as hull edges, hull corners, airlocks, engines, cockpits, and turrets, should be painted with a clear "forward/outward" orientation in their default sprite.

Asset categories:

* `assets/tiles/hull/`
  Structural hull foundation art and exterior fixture component art used by the Hull editor layer:
  * `hull.png`
  * `hull_inner_corner.png`
  * `hull_outer_corner.png`
  * `airlock.png`
  * `engine.png`
  * `hardpoint.png`
  * `turret.png`
* `assets/tiles/logistics/`
  Deck, route, and logistics-link art used by the Logistics editor layer or logistics presentation:
  * `floor.png`
  * `wire.png`
  * `duct_oxygen.png`
  * `pipe_raw_salvage.png`
  * `pipe_repair_charge.png`
  * `pipe_fuel.png`
  * `pipe_ammunition.png`
  * `pipe_oxygen.png`
  * route variants with `_end`, `_straight`, `_corner`, `_tee`, and `_cross` suffixes
  * `service_link.png`
* `assets/tiles/components/`
  Ship and station component art used by the Components editor layer and runtime module rendering:
  * `core.png`
  * `interior.png`
  * `cockpit.png`
  * `computer.png`
  * `processor.png`
  * `reactor.png`
  * `cargo.png`
  * `cargo_fuel_tank.png`
  * `cargo_ammo_rack.png`
  * `cargo_raw_salvage.png`
  * `cargo_repair_charge.png`
  * `cargo_o2_canister.png`
  * `battery.png`
  * `junction_box.png`
  * `valve.png`
  * `interior_wall.png`
  * `o2_generator.png`
  * `o2_canister_storage.png`

Variant sprite expectations:

* The runtime currently falls back to family sprites such as `reactor.png` and `battery.png` for upgraded variants.
* Better component variants should eventually receive dedicated art with the same `32x32` contract when available.
* Expected upgraded sprite names belong under the same category as their base component, for example:
  * `core_expanded.png`
  * `cockpit_advanced_helm.png`
  * `reactor_fission.png`
  * `reactor_fusion.png`
  * `battery_capacitor.png`
  * `processor_fabricator_fast.png`
  * `assets/tiles/hull/turret_laser.png`
  * `assets/tiles/hull/turret_ballistic.png`
  * `shield_radial.png`
  * `shield_directional.png`

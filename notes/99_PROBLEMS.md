# PROBLEMS — Known issues and bugs in need of fixing

* [ ] Components other than the turret may need similar animated overlayed parts, e.g. airlock doors
  * [ ] Some components may need some simple shader effects, like the engines, reactors, fires on the ship (once implemented)
* [ ] Player movement needs to be continuous rather than discrete
* [ ] Simulation stability testing for single player/multi player
* [ ] Components need proper full-screen (or at least most-of-screen) UIs
* [ ] Need enemy ships rather than just single-point enemy turrets
* [ ] Ship turn speed is too high and too immediate - should have an angular inertia calculation and should be driven by retrothrusters or side-mounted engines
* [ ] Ship forward speed should be faster
* [ ] No ability to write ARCH programs
* [ ] Power usage of components should consider if the component is in use - primarily engines and turrets
* [ ] tiles/README.md is missing docs for some tiles
* [ ] Reactors don't generate enough power
* [ ] Shots should be drawn over the top of the ship
* [ ] Magic numbers and constants in code should be pulled out into a config file, which is loaded as one or more ECS resources
* [ ] The UI is very verbose, and certain panes overlap one another
* [ ] Ship builder and sector map should support scroll zoom in/out and scrollwheel click dragging to move about the map

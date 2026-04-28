# PROBLEMS — Known issues and bugs in need of fixing

* [ ] Components other than the turret may need similar animated overlayed parts, e.g. airlock doors
  * [ ] Some components may need some simple shader effects, like the engines, reactors, fires on the ship (once implemented)
* [ ] Player movement needs to be continuous rather than discrete
* [x] Magic numbers and constants in code should be pulled out into a config file, which is loaded as one or more ECS resources
  * [ ] Ship turn speed is too high and too immediate - should have an angular inertia calculation and should be driven by retrothrusters or side-mounted engines
  * [ ] Ship forward speed should be faster
  * [ ] Reactors don't generate enough power
* [ ] Simulation stability testing for single player/multi player
* [ ] Components need proper full-screen (or at least most-of-screen) UIs
* [ ] No editor for reading/writing ARCH programs
* [ ] The UI is very verbose, and certain panes overlap one another

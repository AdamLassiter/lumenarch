# PROBLEMS — Known issues and bugs in need of fixing

* [ ] Components other than the turret may need similar animated overlayed parts, e.g. airlock doors
  * [ ] Some components may need some simple shader effects, like the engines, reactors, fires on the ship (once implemented)
* [ ] Rebalance some numbers
  * [ ] Ship turn speed is too high and too immediate - should have an angular inertia calculation and should be driven by retrothrusters or side-mounted engines
  * [ ] Ship forward speed should be faster
  * [ ] Reactors don't generate enough power
* [ ] Simulation stability testing for single player/multi player
* [ ] Components need proper full-screen (or at least most-of-screen) UIs
* [ ] No editor for reading/writing ARCH programs
* [ ] The UI is very verbose, and certain panes overlap one another
* [ ] Enemy ships still don't spawn, only stationary turrets
* [ ] Make the inertia around ships a field emitted by the ship's core
  * [ ] On leaving the editor (or once at the start of combat), calculate the minimum size of the field needed to fit the whole of the ship, with a maximum size determined by the quality of the core. Since we only have a basic ship core, this can just be a size configured in the balance config
* [ ] Players should move faster while aboard a ship than in space
* [ ] Scrap and other items need more effort - one suit per player, one carried item per player
  * [ ] Needs some design for types of suits and what items will exist
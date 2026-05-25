# PROBLEMS — Known issues and bugs in need of fixing

* [ ] Components other than the turret may need similar animated overlayed parts, e.g. airlock doors
  * [ ] Some components may need some simple shader effects, like the fires on the ship (once implemented)
* [ ] Rebalance some numbers
  * [ ] Ship turn speed is too high and too immediate - should have an angular inertia calculation and should be driven by retrothrusters or side-mounted engines
  * [ ] Ship forward speed should be faster
  * [ ] Reactors don't generate enough power
* [ ] Simulation stability testing for single player/multi player
* [ ] Components need proper thematic UIs
* [ ] Players should move faster while aboard a ship than in space
* [ ] Scrap and other items need more effort - one suit per player, one carried item per player
  * [ ] Needs some design for types of suits and what items will exist
* [ ] Some components are missing unique sprites
* [ ] Sometimes, navigating to a sector takes you to the wrong sector (probably always a previously-selected selector - i.e. select A, select B, travel -> arrive at sector A - even allowing reasonable time between clicking between sectors, so more of a state issue than a race condition)
* [ ] Atmospheric decompression pulls players towards the top-left corner of the ship node that is leaking oxygen, rather than the void of space outside of the ship
* [ ] Ship preview while docked still appears in random positions - this seems to be retaining the position and rotation while in gameplay
  * [ ] The same (at least some amount of the rotation) also applies to the refit ship editor

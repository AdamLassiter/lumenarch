Actor sprite expectations

The player/lobby profile flow can load these optional actor sprites:

- `player_default.png`
- `player_radiation.png`
- `player_welder.png`
- `player_eva.png`
- `station_engineer.png`
- `station_contracts.png`
- `station_archives.png`

Usage notes:

- Sprites are tinted in-game using the player's chosen lobby color.
- Keep silhouettes readable when colorized.
- A square-ish footprint around 16x16 to 24x24 pixels works well with the current encounter scale.
- If a file is missing, the game falls back to a colored simple sprite so development can continue without blocking on art.

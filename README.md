# Connected Textures for Horizon

By tariksabanuz. A native mod (`.hnmod`) for [Horizon](https://github.com/horizon-rust/horizon), the Rust
rewrite of Minecraft Java 26.3: glass, glass panes, bookshelves and sandstone connect to the same block beside
them.

- **Glass and stained glass** (all 16 colours and tinted glass): the frame between two blocks of the same glass
  goes away on every face, so a window is one pane with one frame round it. An inner corner keeps its frame pixel
  where the block across the corner is missing.
- **Glass panes**: the same, on the sides of the pane, between panes of the same colour.
- **Bookshelves, cut and chiseled sandstone** (red too): joined left and right, into one long shelf or band.
- **Sandstone and red sandstone**: a side with the same block above shows the sandstone bottom (its cracked pattern without the band), so a wall or a
  pillar has no band between its blocks; the top row keeps its band.
- A face whose neighbour's face is hidden (a block in front of it) does not join, so corners and steps stay framed.
- Options > Mods > Connected Textures turns it on and off and each part on its own; the world is meshed again at
  once. The settings are kept in `config/connected_textures.txt`.

## How it works

It hooks Horizon's section mesher (`Mesher::mesh_reusing`). After Horizon has made a section's quads, each face of
a connecting block is cut, in its sprite's own space, into the thin strips along its edges (a sixteenth of the
sprite: one pixel of a 16x16 texture), its corners and its middle. A strip along an edge that joins a neighbour
shows the texture just inside the frame instead of the frame. Sandstone sides switch to the sandstone bottom sprite, and the tops and bottoms of stacked glass panes are left out.
Light, ambient occlusion and colour are carried over smoothly.

No texture is shipped: it works on the textures your game already has, and a resource pack that changes glass or
bookshelves is joined the same way.

## Not (yet) included

- OptiFine-format CTM resource packs (`optifine/ctm/*.properties` with their own tile sets), random and repeat
  textures, and emissive textures are not read: only the built-in blocks above connect.
- Distant terrain (Horizon's LOD) is drawn without the joins.

## Credits and copyright

Inspired by what Continuity (by PepperCode1) does for Fabric. This is a clean-room mod for Horizon: it contains
none of Continuity's code or textures, and uses only your own game's textures at runtime. The icon is my own.
MIT licence.

## Build

```sh
python3 tools/package.py          # target/connected_textures-<version>.hnmod
cd test && rustc --edition 2024 --test lib.rs -o t && ./t
python3 check/build.py            # type check of the hooks against stand-ins
```

Install the `.hnmod` in the Horizon launcher (Performance build).

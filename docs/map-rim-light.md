# Map rim light

Reversing doc for `map-rim-light`. Its `batches.jsonl` note points here.

A rim light for characters that the map configures. The batch names 2 classes and 8
fields. Both classes are new in `16.20`.

```
MapLightRegions                        fa8824c0   232 B
    +8    DefaultRenderData
    +184  TextureRenderDataList
    +200  TextureWidth
    +202  TextureHeight
    +204  RimLightProperties     c74f953c   Embed RimLightMapProperties

RimLightMapProperties                  9c39a518   28 B
    +0    Enabled                02f3b39e   Bool, default false
    +4    rotation               21ac415f   Vec3, default (90, 0, 0)
    +16   Contrast               2a856173   F32, default 0.5
    +20   SunLightBlendWeight    b2a597b2   F32, default 0.0
    +24   SunShadowBlendWeight   7f337738   F32, default 1.0

LightRegionRenderData                  f15a89f3   128 B, 112 B before 16.20
    +112  RimLightColor          783ad838   Vec3, default (1, 1, 1)

(unnamed)                              80f31f46   32 B, pointer 8881ee77 of SkinMeshDataProperties
    +0    ed8edf74                          F32, default 0.1
    +4    dba24ad6                          F32, default 1.0
    +8    RimOffset
    +12   ModelHeight
    +16   ShadowGradientRenderData  bdc69b11   Embed ShadowGradientRenderData

ShadowGradientRenderData               bdc69b11   16 B
    +0    ShadowFactor           8a74fffa   F32, default 0.3
    +4    GradientStart          4815183f   F32, default 0.65
    +8    GradientEnd            57d3720a   F32, default 0.8
```

Offsets and sizes are from the registration functions of the 16.21 PBE Windows client,
build `8255794`. The dumps of `16.20.8248524` and `16.21.8264391` have the same classes.

`RimLightColor` was a field of `SkinMeshDataProperties` from `16.16.8042660` to
`16.19.8230722`. In 16.20 the field is on `LightRegionRenderData`.

## What the client does

Read from build `8255794` and from the compiled shaders of `16.21.8264391`. Not tested in
game.

- `MapLightRegions` is a `MapGraphicsFeature`. If `RimLightProperties.Enabled` is true,
  the feature adds the shader define `USE_MAP_RIM_LIGHT = 1`.
- When the feature binds its shader constants and `Enabled` is true, it writes two
  constants of the `MantisSharedMapParameters` buffer:
  - `RIM_LIGHT_DIR_WS = (0, 0, -1) * R`, where `R` is the rotation matrix of `rotation`
    (Euler angles in degrees, sines from a table with 0.1 degree steps). The default
    `(90, 0, 0)` gives `(0, 1, 0)`, which is straight up.
  - `RIM_LIGHT_PARAMS = (Contrast, SunLightBlendWeight, SunShadowBlendWeight)`.
- 20,808 distinct compiled pixel shaders read `RIM_LIGHT_PARAMS`. Each of them has the
  character per-draw buffer and reads `RimLightColor` of the light regions. No vertex
  shader reads it.

The pixel shader, from the disassembly of two variants:

```
rimColor = Blend(RimLightColor)                      // same region blend as the sun color
if any(rimColor > 0):
    c     = min(Contrast, 0.99999)
    ramp  = saturate((dot(N, RIM_LIGHT_DIR_WS) - c / 2) / (1 - c))
    ramp *= lerp(1, sunShadow, SunShadowBlendWeight)
    ramp *= smoothstep(0.4, 0.6, height)
    color = lerp(rimColor, Blend(CharacterSunLightColor), SunLightBlendWeight)
    color *= albedo with its HSV value set to 1
    lit  += color * ramp                             // before the tone map
```

- `N` is the world normal. `RIM_LIGHT_DIR_WS` is the direction from the surface to the
  light.
- `Contrast` narrows the ramp around `dot = 0.5`: 0 gives `saturate(dot)`, 0.5 gives a
  ramp from 0.25 to 0.75, and a value near 1 gives a step at 0.5.
- `sunShadow` is the shadow term of the character sun light, after the shadow gradient
  below.
- `height` is a vertex output. In the one vertex shader that was read, it is the bind-pose
  `POSITION.y` divided by `ModelHeight`.

The shadow gradient, in the same pixel shaders (`SHADOW_GRADIENT` of `CharacterPerDrawPS`):

```
s         = min(LIGHT_GRID_TEXTURE[layer 6].r, 1)
t         = smoothstep(GradientStart, GradientEnd, saturate(height))
sunShadow = s + t * (1 - s) * (1 - ShadowFactor)
```

Below `GradientStart` the shadow is the sampled value. Above `GradientEnd` the darkness of
the shadow is multiplied by `ShadowFactor`.

## Evidence

| hash | name | on | what fixes it | tier |
|---|---|---|---|---|
| 783ad838 | RimLightColor | LightRegionRenderData | member of the shader struct `LightRegionRenderData` at offset 112, the offset of the field | 1 |
| bdc69b11 | ShadowGradientRenderData | class | struct type of `SHADOW_GRADIENT` in `CharacterPerDrawPS`; 3 floats at 0, 4, 8 in both | 1 |
| bdc69b11 | ShadowGradientRenderData | 80f31f46 | the field hash is the class hash | 1 |
| 8a74fffa | ShadowFactor | ShadowGradientRenderData | struct member at offset 0 | 1 |
| 4815183f | GradientStart | ShadowGradientRenderData | struct member at offset 4 | 1 |
| 57d3720a | GradientEnd | ShadowGradientRenderData | struct member at offset 8 | 1 |
| c74f953c | RimLightProperties | MapLightRegions | role list; the client reads this value into `RIM_LIGHT_*` | 3 |
| 9c39a518 | RimLightMapProperties | class | pattern `RimLight<X>Properties`, registration order | 3 |
| b2a597b2 | SunLightBlendWeight | RimLightMapProperties | pattern `<X>BlendWeight`; the shader blends toward `CharacterSunLightColor` by this value | 3 |
| 7f337738 | SunShadowBlendWeight | RimLightMapProperties | pattern `<X>BlendWeight`; the shader blends toward the sun shadow by this value | 3 |

**Shader reflection.** The compiled dx11 shaders keep their reflection data. The five
tier 1 names are identifiers of that data in retail `16.20.8248524` and in PBE
`16.21.8264391`: `RimLightColor` is in 631 chunks of the retail shader cache, and
`ShadowGradientRenderData`, `ShadowFactor`, `GradientStart` and `GradientEnd` are in 654.
The method is that of [shader-reflection](shader-reflection.md).

**The reader came first.** The client copies +16, +20 and +24 of the embedded value into
`RIM_LIGHT_PARAMS` in that order, and the shader gives each component one role. The word
lists were written from those roles.

**Runs.** One list of 347 words gives 120,753 stems of zero, one or two words.

| pattern | targets | expected chance hits | matches |
|---|---|---|---|
| `<stem>BlendWeight` | b2a597b2, 7f337738 | 0.00006 | `SunLightBlendWeight`, `SunShadowBlendWeight` |
| `RimLight<stem>Properties` | 9c39a518 | 0.00003 | `RimLightMapProperties` |

`RimLightProperties` and `RimLightColor` matched in an earlier list of 49,650 names
(0.00008 expected against 7 targets).

**Registration order.** The client registers classes in near-alphabetical order of their
names. The registration function of 9c39a518 is between those of `RegionEntityTemplate`
and `RotateMapLightUpdater`. The registration function of bdc69b11 is between those of
`SHData` and `SkinMeshDataProperties_MaterialOverride`. Both names sort in those places.

**Field names.** `RimLightProperties` is the class name without `Map`. `SunLightBlendWeight`
has the casing of the sibling `SunLightColor`; `SunlightBlendWeight` is the same hash.

## Ruled out

- No name of the batch is a string in the 16.21 PBE Windows client. The client has the
  strings `USE_MAP_RIM_LIGHT`, `RIM_LIGHT_DIR_WS` and `RIM_LIGHT_PARAMS`.
- A search of four-word names from the same 347 words (108 expected chance hits against
  the class, the two floats and 3069f601) gave no second name with a meaning for any of
  them, and no name for 3069f601.

## What is left

| hash | what is known |
|---|---|
| 80f31f46 | value class of `RimOffset`, `ModelHeight` and the shadow gradient, on pointer 8881ee77 of `SkinMeshDataProperties` and `SkinMeshDataProperties_MaterialOverride` |
| ed8edf74, dba24ad6 | `F32` fields of 80f31f46, defaults 0.1 and 1.0 |
| 3069f601 | child of `LightRegionRenderData` with `Probe` and `CharacterProbe`, 176 B |

## Shipped data

No bin in PBE 16.21 has an object of `MapLightRegions`, `LightRegionGeComponentDef`,
9c39a518, 80f31f46 or bdc69b11. The same search has 203 matches for `MapSunProperties`.

## Status

All 10 rows `status=pending`, `pr=-`. The names are fit to submit. No shipped bin has an
instance of any of them, so an upstream PR must state that.

# Shader reflection attestation

The reversing doc for `shader-reflection` and `shader-reflection-macro`. Their
`batches.jsonl` notes point here. The plan is `docs/plans/shader-reflection.md`;
the method lives in `scripts/shader_reflect.py`, which reproduces everything
below from a League install.

Compiled shaders ship with their reflection metadata intact, so the probe set is
identifiers Riot wrote rather than names a wordlist assembled. Same shape as
`exe-strings`, and disjoint from it: reflection names are compared by `strcmp`
at effect-build time and exist only in data, never as exe strings.

## Corpus

Retail 16.15.8024387, Windows, dx11 only.

| wad | chunks | DXBC containers |
|---|---|---|
| `ShaderCache.dx11.wad.client` | 2,176 | 71,776 |
| `Bootstrap.windows.wad.client` | 469 | 5,452 |

Checked across **all 455 wads** in the install: compiled-shader paths resolve in
those two and nowhere else. 2,154 of ShaderCache's chunks resolve against
CDragon's `.game` table, leaving 22 unresolved paths. Every one of Bootstrap's
1,630 distinct reflection blocks also occurs in ShaderCache, so it added no
names, only confirmation.

77,228 containers collapse to **44,667 distinct `RDEF` blocks** after
deduplicating on the chunk bytes, and those yield **3,125 distinct identifiers**:
2,350 members, 742 resource bindings, 24 constant buffers, 9 type names.

Permutations are unioned rather than selected. Windows ships no HLSL source, so
a maximal build cannot be constructed, and `#ifdef` blocks add whole constant
buffers, so no single permutation is a superset of the others. Walking all of
them is free once the container is being parsed anyway.

Struct type names live **only** in the shader-model-5 extended type descriptor -
five `u32` past the 16-byte descriptor, the last pointing at the type's own name.
An SM4-shaped parser drops them silently, and they are the highest-value field:
`LightRegionRenderData` survives nowhere else.

## The three tiers

| tier | probes | expected chance hits | hits |
|---|---|---|---|
| literal | 3,096 | 0.005 | 4 |
| undecorated | 381 | 0.001 | 0 |
| deunderscored | 2,303 | 0.004 | 19 |

Against 2,421 unresolved classes + 4,483 unresolved fields = 6,904 target states.
The sweep's *quiet* tier runs at 8-11, so noise is not the constraint here, which
is the point of an attested probe set.

**literal** is the identifier as reflected. The bin hash is FNV-1a over the
lowercased name, so shipped `modelHeight` and written `ModelHeight` are one hash
and the recasing is the house rule's, not a new claim. This is the `exe-strings`
standard.

**undecorated** strips the suffix the effect builder appends to a resource
binding: `BAKED_DIFFUSE_TEXTURE__TX` -> `BAKED_DIFFUSE_TEXTURE`. It found
nothing. Recorded because a tier that costs 381 probes and returns zero is worth
knowing about before someone builds it again.

**deunderscored** rejoins a `SCREAMING_SNAKE` identifier:
`SUN_PENUMBRA_SATURATION` -> `SunPenumbraSaturation`. **This tier is a
reconstruction.** An underscore is hashed like any other byte, so the shipped
identifier does not hash to the target; what hashes is the word sequence with
the separators removed. The shader attests the concept and the words in order
and the hash confirms the join, but that is not the standard the literal tier
meets, which is why it is a separate batch.

## What landed

### `shader-reflection` (2 fields)

| hash | name | evidence |
|---|---|---|
| `2119af58` | `ReflectionSkyTint` | member of the `LightRegionRenderData` struct in `LightRegionInfo_SharedDataBuffer`, 595 shaders |
| `e9552339` | `ModelHeight` | `$Globals` of 96 `skinnedmesh/*.vs`, shipped as `modelHeight` |

`ReflectionSkyTint` **completes `LightRegionRenderData`**: the class had 15
fields, 14 named by the `light-regions` batch and one left over. The reflected
struct declares 16 members - those 15 plus `mUnusedPadding0`, which has no CPU
side. This is the field that batch's note flagged as unnamed.

`ModelHeight` lands on an unnamed 4-field class whose one named field is
`RimOffset`, and `rimOffset` sits in the `$Globals` of 576 `skinnedmesh/*.ps`
alongside it. The class is the CPU side of that shader parameter block. Upstream
already spells the sibling `RimOffset` against a shipped `rimOffset`, so the
recasing here follows an existing precedent rather than setting one.

### `shader-reflection-macro` (19 fields)

**16 on one class.** An unresolved shader (`3dc2cb2da9509e57`) declares a
20-variable `$Globals` of water simulation tunables. An unnamed class based on
`IGameModeConfigClient` holds 27 fields, and `DAMPING` -> the already-named
`Damping` pins the alignment:

    WAVE_SPEED_SQ  WAVE_SPEED_MIN_MULTIPLIER  WAVE_SPEED_AMP_GAIN  DAMPING
    WAKE_FRONT_{LENGTH,WIDTH,WEIGHT}_MULT
    WAKE_TROUGH_LENGTH_{MIN,MAX}_SPEED_MULT  WAKE_TROUGH_WIDTH_MULT
    WAKE_TROUGH_BACKWARD_BIAS_{MIN,MAX}_SPEED_MULT  WAKE_TROUGH_WEIGHT_MULT
    WAKE_AMPLITUDE_{MIN,MAX}_SPEED_MULT

All 15 rejoin onto unresolved fields of that class, every one `F32` against the
shader's `float`. `NORMAL_STRENGTH`, from a sibling water shader
(`0d3b967ce4d8be7c`, bound to the same `WaterState` texture), makes 16.

The five `$Globals` entries that do *not* land - `IMPULSE_AMPLITUDE`,
`IMPULSE_COUNT`, `REPROJ_OFFSET`, `TERRAIN_BLEND_XFORM`, `TEXTURE_SCALE` - are
per-frame engine state rather than designer config, which is what a class based
on `IGameModeConfigClient` would be expected to omit. The absence corroborates
the alignment rather than weakening it.

**3 elsewhere**, each landing in the domain its constant buffer serves:

| hash | name | from | lands on |
|---|---|---|---|
| `671e8711` | `SunPenumbraSaturation` | `PerFramePixelCB`, 777 shaders | `MapSunProperties`, `MapLightingInfo` |
| `c6d048fc` | `HdrEnvDiffuseScale` | `PerFramePixelCB`, 777 shaders | + `MapLightingVolume` |
| `3bd77ed1` | `ApplyTeamColorCorrection` | `$Globals` of 34 `particlesystem/*.ps` | `VfxLegacyRenderComponent`, `VfxMaterialRenderComponent` |
| `adee7179` | `BloomIntensityScale` | `PostEffectPixelCB`, 11 shaders | `PostEffectOptions` |

The two lighting names fill classes whose remaining fields are `sunColor`,
`sunDirection`, `skyLightColor` and `lightMapColorScale` - the same block
`PerFramePixelCB` carries as `SUN_LIGHT_COLOR`, `SUN_LIGHT_DIRECTION` and
`LIGHT_MAP_COLOR_SCALE_AND_INTENSITY`. `BloomIntensityScale`'s owner is named by
the unproven `PostEffectOptions`, so the class name is not evidence; its named
fields (`Dof`, `Coc`, `DepthFog*`, `HeightFog*`) are.

## Held back

**`Bounce`** (`3363663d`, on `VfxPhysicsGroundCollisionModifier`) and **`Pivot`**
(`fbf05517`, on an `ISequenceAction` with `StartPosition`/`EndPosition`). Both
hit the literal tier and both fit their class semantically, which is exactly the
problem. `Bounce` occurs in **one** shader, `ui/ui_rgbshift.ps`, and `Pivot` in
two, `galaxyscreen_flipbook.vs` and `defaultenv_rotate.vs`. Nothing connects a
UI colour-shift parameter to a physics modifier. The corpus is ~3k identifiers of
ordinary graphics vocabulary, so a common word landing on a plausible field is
the coincidence it produces most, and a hit is attestation only when the block
the name lives in corresponds to the class that owns the field.

`data/shaders/shaders.bin` confirms it independently: `Bounce` is a parameter of
exactly one `CustomShaderDef`, `Shaders/UI/UI_RGBShift`, and `Pivot` of two,
`Shaders/StaticMesh/DefaultEnv_Rotate` and `Shaders/SkinnedMesh/GalaxyScreen_Flipbook`.
Both reproduce the DXBC finding from a separate file in a separate wad.

Unproven, not refuted, so they are **not** in `hashes/bad/`.

## Corroboration

The `light-regions` batch is reproduced whole and independently: `SunLightColor`,
`ProbeIndex`, `CharacterSunLightColor`, `CharacterProbeIndex`,
`CharacterSunLightDirection`, `DepthFogColor/Start/End/MaxIntensity`,
`HeightFogColor/Start/End/MaxIntensity` and the class `LightRegionRenderData` all
come back out of a sweep that was not looking for them.

73 further literal probes hit hashes already named. Most of that is worthless -
`Alpha`, `Color`, `Speed`, `Radius` are shared vocabulary, not the same entity -
and the same caveat that holds `Bounce` back applies to reading them as
confirmation. The multi-word ones (`SampleRadius`, `DitherStrength`,
`TextureResolution`, `DelayTime`, `CameraPos`) are worth more.

`SSAOKernel` and `SSAO_TEXTURE_SharedTexture` confirm the engine spells the
acronym in capitals, which is what the `ssao-casing` batch and the `MapSSAO`
entry in `names.py`'s exempt list rest on.

## Casing

Shader casing is its own namespace and is not evidence for a bin field's
spelling. `LightRegionRenderData` declares `Priority` where the tables carry
`priority`, and both are correct in their own domain. Names entered under the
repo rule: `modelHeight` -> `ModelHeight`, `HDR_*` -> `Hdr*`.

## The two unproven batches

`light-regions-unproven` (6 rows) and `map-graphics-unproven` (5 rows) are marked
DO NOT SUBMIT for want of a string. A full sweep was expected to promote or
refute them. It did **neither**: none of `DefaultRenderData`, `LightChannels`,
`PriorityOverride`, `TextureRenderDataList`, `LightRegionTextureData`,
`PostEffectOptions`, `MapSSAORenderer`, `RenderTargetSize`, `MapAntiAliasing` or
`MapChunkVisibility` occurs in the reflection corpus in any tier. Both batches
stay as they are.

The near misses are worth recording so nobody re-runs this expecting them:
`LIGHT_REGION_TEXTURE_SIZE` and `LIGHT_REGION_TEXTURE_UPDATE_SharedTexture`
exist, and neither attests `LightRegionTextureData` or `TextureRenderDataList`.

## Not attempted

`ISGN`/`OSGN` vertex semantics were parsed and ignored - they are `POSITION0`
and `TEXCOORD6`, useful for vertex-format work and not for meta names.

`data/shaders/shaders.bin` was run as a second probe set and is a **measured**
dead end, not an assumed one. Dump it with `league_structs/tools/bin-dump`; a
parser written against the modern contiguous type enum dies on it, because
container tags are the legacy `0x80`+ numbering. It holds 351 `CustomShaderDef`
entries and **zero unresolved field hashes**, so there is nothing in it to crack.
Its 3,912 strings - `ShaderPhysicalParameter`/`ShaderTexture` names, sampler
names, `featureDefines`, entry paths - are the artist-facing CPU-side names, so
they looked like a better probe set than the HLSL internals. They are not: 3,884
literal probes returned 50 already-known names and **no new hits**, and the
deunderscored tier returned none at all. The two fields it does hit are `Bounce`
and `Pivot`, the pair already held back above.

It is still worth keeping for casing and for `ModelHeight`, which it carries as
`modelHeight` on a `ShaderPhysicalParameter` beside `rimOffset` - a second
shipped source for that name, in a different file and a different wad.

Naming the water class itself needs its shader path, and both water shader paths
are among the 22 CDragon does not resolve.

The `.glsl` and `.metal` variants are text, so they carry full identifiers with
no permutation problem and no separator reconstruction. They are in the `.game`
path table but not in a Windows install; if a Mac or mobile build is reachable
that is a richer probe set than any tier above.

## Status

19 rows `pending`, `pr=-`. `HdrEnvDiffuseScale` and `NormalStrength` also fell
out of this sweep but were landed first by `map-entity-templates` and
`nova-item-getters`, so their rows live there.

`shader-reflection` is fit to submit: both names are identifiers the retail
client ships, which is the standard `light-regions` and `exe-strings` met.

`shader-reflection-macro` is solid but the PR has to say what it is - the shipped
identifier is `SUN_PENUMBRA_SATURATION` and the submitted name is
`SunPenumbraSaturation`, and a reader who assumes the string hashed would be
wrong. Submit only with that stated.

# VFX shapes, values, shading models and targeters, 16.21

Record of the batch `vfx-shading-targeters-16-21`. Its `batches.jsonl` note points here. The round read four
families in the 16.21 Windows client, build `8255794`: the spawn shapes (`IVfxShape`), the legacy animated
values (`ValueFloat` and its curves), the shading models (`IX3dShadingModel`) with the shared shader resources
(`X3DShared*`), and the spell targeting indicators (`TargeterDefinition`).

| family | live classes | unnamed classes before | after | unnamed fields before | after |
|---|---:|---:|---:|---:|---:|
| spawn shapes, animated values | 31 | 0 | 0 | 0 | 0 |
| shading models and shared resources | 23 | 2 | 0 | 3 | 0 |
| targeters and `SharedTargeterResources` | 24 | 1 | 0 | 8 | 0 |

No string attests any name; casing follows named siblings.

## What fixes the names

| hash | name | what fixes it |
|---|---|---|
| 4bcf438a | X3DSharedDataBufferDef | the `sharedDataBuffers` / `dataBuffers` element type, twin of `X3DSharedTextureDef`, `X3DSharedSamplerDef` and `X3DSharedConstantBufferDef`; the client binds it as a shader data buffer and logs `Missing databuffer "%s"` when one is not found |
| 52fc6665 | DataBufferElementType | U8 on `X3DSharedDataBufferDef`, twin of `TextureElementType` on `X3DSharedTextureDef`; the client derives the element stride from it (4, 8, 12, 16, 48 or 64 bytes, or the size of the `StructureLink` struct) |
| c89676aa, 2d0b672c | MinLod, MaxLod | U32 on `X3DSharedSamplerDef` next to `mipLodBias`; the client converts both to float and writes them to the sampler's minimum and maximum mip level, in that order |
| 9eb07cec | EnvironmentPbrDynamicShadingModel | environment shading model that binds the PBR probe without a baked light channel; the only hit among 134M `Environment<two words>ShadingModel` candidates (noise 0.03); registers between `EnvironmentBakedLightingShadingModel` and `EnvironmentPbrShadingModel` |
| a8fef3df | SharedTargeterResources | the class of the object `Shared/SharedTargeterResources` in `Global.wad`; the client loads the default indicator textures from it |
| 25f550a, 68337617 | AoeRangeIndicator, WallRangeIndicator | texture fields of `SharedTargeterResources` next to `CircularRangeIndicator` and `ConicRangeIndicator`; live values `aoe.tex` and `walltargetcursor.tex`, used by the area and wall indicators |
| 49a76d44 | AutoAttackDistanceIndicatorOutOfRange | twin of `AutoAttackDistanceIndicator`; live value `autoattackrangeindicator_outrange.tex` |
| 4549fe91 | AutoAttackLineIndicator | removed field of `SharedTargeterResources`, next to `AutoAttackDistanceIndicator` and `AutoAttackRangeIndicator` |
| 87753e0a, 2fec612a | LineRangeLocalBase, LineRangeLocalTarget | base and tip textures of the local line indicator (`locallinemissilebase.tex`, `locallinemissiletarget.tex`); both hashes reduce to one prefix hash, `LineRangeLocal` |
| ba54f86d, d3a0a511 | LineRangeGlobalIndicatorBase, LineRangeGlobalIndicatorTarget | base and tip textures of the global line indicator; both hashes reduce to one prefix hash, and `LineRangeGlobal` + one word gives `Indicator` as the only hit (noise 1e-6) |

`X3DSharedDataBufferDef`, `DataBufferElementType`, `MinLod`, `MaxLod`, `AutoAttackDistanceIndicatorOutOfRange` and the four line
fields are tier 3 (twin names or shared-prefix pairs, each read in the client). `EnvironmentPbrDynamicShadingModel` is tier 4 (single hash, reader and registration order agree);
no shipped bin uses it. `SharedTargeterResources`, `AoeRangeIndicator`, `WallRangeIndicator` and
`AutoAttackLineIndicator` are tier 4 (single hash; reader and shipped values agree).

## What is left

- `GenericSplineMovementSpec` field `0xff738f10` (U8, three values) is outside this round.

## Status

All rows `pending`.

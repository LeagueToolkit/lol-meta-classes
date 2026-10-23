# VFX components, 16.21

Record of the batch `vfx-components-16-21`. Its `batches.jsonl` note points here. It names classes and fields
of the component emitter: `VfxComponentBase` and its components, the physics modifiers, the spawn and
lifetime behaviours, the render and geometry support classes and the facing modes. The driver graph is
covered by `vfx-driver-graph` and `driver-family`.

The batch names 17 classes and 60 fields (one of them, `DynamicParticleParameter`, is on `VfxDriverContainer`):

| part | live classes | unnamed classes before | after | unnamed fields before | after |
|---|---:|---:|---:|---:|---:|
| components and their support classes | 108 | 26 | 9 | 97 | 38 |

No string attests any name; casing follows named siblings (`Rgb`, `Uv`, `Initial` / `Keyed`).

## What fixes the names

Every class was read in the 16.21 Windows client, build `8255794`. The component path runs in that build:
the readers traced spawn, step, kill and draw for each class below.

**Field sets (tier 3).** The modular physics modifiers use `Initial` (sampled at spawn) and `Keyed` (sampled
over the particle's life) with `Local` / `World` variants. One lattice over these parts gives complete sets,
each field matched to the slot that reads it:

| class | fields |
|---|---|
| `VfxPhysicsAccelerationModifier` | `InitialLocalAcceleration`, `InitialWorldAcceleration`, `KeyedLocalAcceleration`, `KeyedWorldAcceleration` |
| `VfxPhysicsVelocityModifier` | `InitialLocalVelocity`, `InitialWorldVelocity`, `KeyedLocalVelocity`, `KeyedWorldVelocity`, `InitialRadialVelocity`, `KeyedRadialVelocity`, `RadialOriginOffset` |
| `VfxPhysicsRotationModifier` | `InitialRotationalVelocity`, `KeyedRotationalVelocity`, `KeyedRotation`, `UseKeyedRotation`, `WorldSpaceRotation` |
| `VfxPhysicsOrbitModifier` | `InitialOrbitOffset`, `KeyedOrbitOffset`, `InitialOrbitRotationalVelocity`, `KeyedOrbitRotationalVelocity`, `InitialRotationOffset`, `WorldSpaceRotation`, `DragScaleFactor`, `OrbitOriginSpace` |
| `VfxPhysicsPositionModifier` | `InitialLocalOffset`, `InitialWorldOffset`, `LocalKeyedOffset`, `WorldKeyedOffset` |
| `VfxScaleByVelocityData` | `MinScaleFactor`, `MaxScaleFactor` (scale lerps between them by speed over `VelocityThreshold`) |
| `VfxModularPhysicsComponent` | `IsDirectionVelocityOriented`, `IsDirectionVelocityOrientedOnSpawn` (one flag byte, each step / first step only) |
| `VfxFacingModeCameraFacingPerAxis` | `AlignPitch`, `AlignYaw` |
| 0x54a1b7a6 | `VeryLow`, `VeryHigh` next to the named `Low`, `medium`, `High` |

**Class twins and sets (tier 3).**

| hash | name | what fixes it |
|---|---|---|
| 362b4996 | VfxShimmerPrimitiveNonRenderable | twin of `VfxPrimitiveNonRenderable`; no fields, no vertex builder; between `VfxShimmerPrimitiveMesh` and `VfxShimmerPrimitiveQuad` |
| a87ee7f5 | VfxShimmerPrimitiveQuadBase | base of `VfxShimmerPrimitiveQuad` in PBE 16.22; takes its `PivotPoint` and `FacingMode` |
| 8152c1ec, 1c3dfd0c | VfxAnimatedColorRgb, VfxAnimatedColorRgbVariableData | `Rgb` twins of `VfxAnimatedColor` / `VfxAnimatedColorVariableData`; both between those two |
| 32952395 | VfxUvAnimationContainer | the `UvAnimation` field type, next to `VfxColorContainer`, `VfxDriverContainer`, `VfxMaterialContainer`; flipbook start frame, frame count, curve |
| b56b2deb, 1addb679 | VfxFacingModeCameraFacingPerAxis, VfxFacingModeCameraFacingRotationOriented | camera-facing basis per axis / around the particle's local Z; between `VfxFacingModeCameraFacing` and `VfxFacingModeViewAligned` |
| 709be6c3, 7015f762, dbb4f634 | VfxLifetimeStateBehavior, SimpleOutro, LoopingIntroOutro | the `LifetimeBehavior` interface and two of its three behaviours; the behaviours carry no `Vfx` prefix and sort in two separate runs |

**Singles (tier 4: reader and registration order agree).**

| hash | name | what fixes it |
|---|---|---|
| 5467726b | VfxEmptyRenderComponent | render component with only `RenderOrder` and no draw; between `VfxEmitterVelocityDriver` and `VfxFacingModeCameraAligned` |
| 31beb841 | VfxDefaultSpawnBehavior | the spawn behaviour `VfxLifetimeComponent` creates when none is set; rate and burst |
| 3fd44d66 | VfxRegularSpawnRate | rate only, burst slot is a stub |
| 44ad896b | VfxSharedMaterial | material container that returns its `Material` link; between `VfxScaleVector3Driver` and `VfxShimmerEmitterDefinitionData` |
| ef7e7bb1 | VfxScaleByVelocityData | the `ScaleByVelocity` field type on `VfxPhysicsScaleModifier`; between `VfxRibbonDefinitionData` and `VfxScaleVector2Driver` |
| 9999dd64 | VfxDynamicParticleParameterDefinition | the `DynamicParticleParameter` field type on `VfxDriverContainer`; fills the dynamic particle buffer |
| 472a8dd8 | VfxFacingModeCameraFacingDirectionOriented | camera-facing basis around the movement direction; completes the set of three `CameraFacing` modes. It registers just before `VfxFacingModeCameraFacing`, not after it: the weakest class name of the batch |
| fields | SpawnBurstCount, EmitterLoops, DoesParticleLifeScale, ChildDefinitionData, EmitterOrientationWeight, Attractors, ScaleByVelocity, Bounce, GroundHeightOffset, InitialFrameOffset, NumFramesInAnimation, PivotPoint, CensoredColorModulate, DynamicParticleParameter, ForceMultiplier, MaxAcceleration | each read in the client |
| legacy fields | IsBurstEmitter, FlexEmitterRate, EmitterRateByVelocity, EmitterTimeActiveDuringPeriod, EmitterTimeOffset, TextureDivisions, ColorTextureScale, ColorTextureOffset, DisableBackfaceCulling | `VfxLegacyLifetimeComponent` / `VfxLegacyRenderComponent`; the code reads each like the matching `VfxEmitterDefinitionData` field |

## Held back

- `0x73fc541d` = `UvScrollAlpha` (exact, but no reader found), `0x1fd12444` = `BindToParticleSystem`,
  `0xb5d23158` = `PhysicsUpdateModel` (four-word search with about 7% noise), `0xcf04b143` = `HorizontalOnly`,
  `0xf6f54bbe` = `ForceScale` (class `0x4a2445d4` is not in the read build).

## What is left

- Classes: `0x287a50ff` (the third lifetime behaviour: intro and outro without loops; `SimpleIntroOutro` does
  not match), `0x402b2a8f` (attractor modifier), `0x8edc8e76` (noise impulse modifier), `0xc685a3b6` (a
  modifier with no runtime), `0x4a2445d4`, `0x54a1b7a6` (per effects-quality spawn percentages),
  `0x6d613e54`, `0x6ec6ed80`, `0xbd3d4dee`.
- 38 fields, most of them flag bits and legacy render fields.

## Status

All rows `pending`. Tier 3 rows are fit to submit upstream; tier 4 rows go in a separate, flagged part of
any upstream PR.

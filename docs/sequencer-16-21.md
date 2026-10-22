# Sequencer, 16.21

Record of the batch `sequencer-16-21`. Its `batches.jsonl` note points here. It closes the holdouts of
`sequencer-actions` (16.15) and names the instance and input classes added since.

The batch names 54 classes and 4 fields:

| tree | live | unnamed before | unnamed after |
|---|---:|---:|---:|
| `ISequenceAction` (eb31be9b) | 67 | 12 | 0 |
| `ISequenceActionInstance` (fb677f88) | 36 | 23 | 0 |
| `SeqInput*` inputs | 57 | 15 | 1 |
| `IMinimapIconModifier` (d5b0f96f) | 3 | 3 | 0 |

No string attests any name; casing follows named siblings (`Ui`, `Hdr`).

## What fixes the names

Every class was read in the 16.21 Windows client, build `8255794`.

**Instance suffix (tier 3).** Each action creates one runtime instance named `<Action>Instance`
(`SequenceActionHideInstance`, `SeqActionEquipGearInstance`). Hashing every named action plus `Instance`
names 13 instance classes from about 70 probes. Folding `Instance` off the unnamed instances lands exactly
on nine unnamed action hashes, which pairs them before either side has a name.

**The `GameScreenSequenceActionUi` prefix (tier 3).** Folding natural suffixes off the UI element actions
gives three shared states: `<P1>Color` / `<P1>Scale`, `<P2>MoveElement` / `<P2>PlaySpineAnimation`,
`<P3>Element` / `<P3>Scene`. `P2` is `GameScreenSequenceActionUi` exactly, `P1` is `P2 + ChangeElement`
and `P3` is `P2 + Enable`. That names six actions and their six instances:

| hash | name |
|---|---|
| 37fe5e09 / 2b2278d6 | GameScreenSequenceActionUiMoveElement / ...Instance |
| 3ddbfcb4 / 56ca5fcf | GameScreenSequenceActionUiChangeElementScale / ...Instance |
| d13da199 / 045397e6 | GameScreenSequenceActionUiChangeElementColor / ...Instance |
| 1499e53b / 05140c28 | GameScreenSequenceActionUiEnableElement / ...Instance |
| bfd585a7 / fd52ca74 | GameScreenSequenceActionUiEnableScene / ...Instance |
| 59bbc4db / ffabacc8 | GameScreenSequenceActionUiPlaySpineAnimation / ...Instance |

The prefix sorts where the 16.15 record predicted (`GameCalculationModified` < `GameScreen...` <
`IGameCalculation`).

**Singles (tier 4: reader and registration order agree).**

| hash | name | what fixes it |
|---|---|---|
| b213e4ff / c4fd2fbc | TftSequenceActionPlaySpineAnimation / ...Instance | plays a clip on a Spine UI element; between `TftSequenceActionPlaySound` and `TftSequenceActionShowText` |
| 9162a320 / 4110cd5b | SequenceActionAttachToMainCamera / ...Instance | parents the object to a point in front of the main camera |
| 5e518c82 / 7945f8bd | SequenceActionShakeMainCamera / ...Instance | input-driven camera shake; field `ShakesPerSec` (d67d18ce); weakest of the set: one of two in-bracket hits, the other meaningless |
| 546d62d1 | SequenceActionLerpColorInstance | created by `SeqActionLerpColor` and its subclasses |
| fea81f3e, dfc8c194 | SeqActionLightHdrScale, SeqInputLightHdrScale | write a light's HDR scale; between `SeqActionLightColor` and `SeqActionLightIntensityScale` |
| 6cae67ad, 4357c4df, b01df39a, 68d2500f | ChangeMinimapIcon, ClearIconOverride, SetIconOverride, SetIconEnabled | a minimap icon action and its three modifiers, in one ascending run |
| 98b663fd, 90e924a6 | SetLightChannels, CameraShakeFalloff | prefix-less actions and helpers in the same runs |
| inputs | SeqInputCombined{LightColor,LightHdrScale,LightIntensityScale,MaterialParamColor,MaterialParamFloat,PointLightRadius}, SeqInputConstUiVfx, SeqInputVarUiVfx, SeqInputGameEntityObject, SeqInputGameEntityTagObjectArray, SeqInputNearestGameEntityByTagObject, SeqInputMapPlaceableReferenceObject, SeqInputCreateZoomableCamera, TftGameStartSequenceUiElement | each input read; the `Combined` and `Const`/`Var` rows twin named siblings |
| fields | EndAudioEvent, PlayerSelector, SpawnDelay | `SeqInputCreateSound`, `SpawnInSequence` |

## Held back

- `0xd31bbf89` = `SpineAnimationDataItem` (noisy search), `0x5a439ea4` = `DefaultCameraSelector` and
  `0x259377f1` = `InputLockoutTime` (each the only meaningful result among several random matches of a
  three-word search).

## What is left

- `0x7b768f17`: a `SeqInput` that computes a look-at orientation in degrees from `SourcePosition` and
  `TargetPosition`.
- `0x34ecdf00` (TFT element interface), the interfaces `0x6900f3ca`, `0x29555a8c`, `0x23482a0a`,
  `0x7d08fe3e`, and the fields `0xcfe79554` (element hash), `0x457338e5`, `0x76df2a8b`, `0x8ef421e5`,
  `0x2b565a38`, `0xcae95eff`, `0xc199472a`.

## Status

All rows `pending`. Tier 3 rows are fit to submit upstream; tier 4 rows go in a separate, flagged part of
any upstream PR.

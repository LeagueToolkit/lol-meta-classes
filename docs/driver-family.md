# Driver family

Record of the batches `driver-family-readers` and `driver-family-brute`. Their `batches.jsonl`
notes point here.

The batches name 36 classes and 3 fields across five trees:

| tree | live | unnamed before | unnamed after |
|---|---:|---:|---:|
| `ILogicDriver` (995ca734) | 143 | 19 | 3 |
| `IVfxBaseDriver` (cbd100e7) | 98 | 14 | 1 |
| `IVfxMaterialDriver` (023f9fb3) | 6 | 2 | 0 |
| `ILogicDriverValueToString` (06ca3cfd) | 8 | 5 | 1 |
| `IMaterialLogicDriverSource` (fa04801e) | 2 | 1 | 0 |

Every name is a hash match. No string attests any of them. Casing is not attested: `Wh` /
`WH`, `Id` / `ID`, `MultiKill`, `Tft`, `Onto` / `OnTo` follow the spelling of named siblings.

## What fixes each name

### Two checks used throughout

- **Reader.** The class's evaluate function was read in the 16.21 Windows client, build
  `8255794`, and the name describes what it returns.
- **Registration order.** Class registrars are emitted roughly in alphabetical order of their
  source files. Over 2,817 adjacent named pairs the order holds for 92%, and a name that is a
  prefix of another sorts on either side of it (`AiHeroClient` before `AiHero`). A hit should
  sort between the named classes registered directly before and after it. This is a filter, not
  proof.

### `driver-family-readers` (tier 4: single hash, reader and order agree)

| hash | name | reader | registered between |
|---|---|---|---|
| ac0fb04b | ILogicVector2Driver | interface whose only holder is `VfxVector2LogicDriver.LogicDriver`; widens to Vec4 as (x, y, 0, 0) | `ILogicIntDriver` .. `ILogicVector3Driver` |
| 00206916 | BoundingBoxWhVector2Driver | width and height of the bounding box | `BoundingBoxSizeVector3Driver` .. `BoundingRadiusFloatDriver` |
| 06a97ad3 | InterestPointVector3Driver | the unit's look-at point: a tracked unit, a stored point, or the nearest enemy | `ISegmentGet` .. `IsAttackingBoolDriver` |
| 791c5a64 | IsAliveBoolDriver | not dead, and a second unit flag clear | same bracket |
| 83a9f4f8 | IsEmoteMutedBoolDriver | the unit's player is emote-muted, or is an enemy while `HideEnemySummonerEmotes` is on | `IsCastingBoolDriver` .. `IsLocalPlayerBoolDriver` |
| 4f92775c | LeaderFacingVector3Driver | facing axis of the object this unit is attached to | `IsMovingBoolDriver` .. `MoveSpeedFloatDriver` |
| 19da44b2 | TrackedMissilePositionVector3Driver | position of the unit's live missile of a `MissileSpellObjects` spell | after `MoveVelocityVector3Driver` |
| 608a3ee7 | GetGearIndexIntDriver | the gear index that `HasGearDynamicMaterialBoolDriver` compares | `DistanceToPlayerMaterialFloatDriver` .. `GetMapPropBoolDynamicMaterialBoolDriver` |
| 43d11d7f | HasMultiKillMaterialBoolDriver | the hero's multikill count == `MultiKillSize` (f7a07027) | `HasGearDynamicMaterialBoolDriver` .. `HasUnitTagBoolDriver` |
| c83eb239 | IsInShopRangeBoolDriver | dead, or inside a same-team shop radius | `IsInGrassDynamicMaterialBoolDriver` .. `IsInWaterBoolDriver` |
| 0cb6a541 | IsOnGameplayTextureBoolDriver | unit position inside a painted shape of `GameplayTexture` | `IsMapPropHoveredDynamicMaterialBoolDriver` .. `IsStealthedBoolDriver` |
| 9bc366ca | IsCharacterSkinIdBoolDriver | skin id == `SkinID`; `RespectDisguise` (3b0a7a6a) selects the disguise record | after `IsChampionNameBoolDriver` |
| c8d666b4 | TftIsEnemyMaterialBoolDriver | TFT team test relative to the board that the viewer is on | `SpellRankIntDriver` .. `TftTeamCannonMaterialDriver` |
| b7b43e1d | PercentageEnabledBoolMaterialDriver | rolls `Percentage` on each rising edge of `BoolDriver` and holds the result | `OneTrueMaterialDriver` .. `RandomFloatMaterialDriver` |
| b2a6e394 | HasValueChanged | true when the Vec4 of `LogicDriver` changes | before `ILogicDriverValueToString` |
| d37e7059 | ILogicDriverValueToStringList | joins item strings with a separator object | after `ILogicDriverValueToString` |
| 49832ca7 | ILogicDriverValueToStringSimple | constant text, no update | before `ILogicValueDriverUpdateViewElements` |
| 1aaa358b | LogicDriverLiteralString | copies `value` | `LogicDriverFloatToString` .. `LogicDriverValueToStringArray` |
| 9a7de981 | LogicDriverLocalizedString | string-table lookup of `value` | same bracket |
| 9b97cf6c (field) | PlayersToInclude | relation flags of the champion filter of `b5c28890` / `e67a27c8` | |

Weakest: `PercentageEnabledBoolMaterialDriver` (wide bracket, four words) and
`HasValueChanged` (no kind suffix, wide bracket).

### `driver-family-brute`

| hash | name | what fixes it | tier |
|---|---|---|---|
| 1d04cfa7, 3eb74cbe, 2d42ea41, 44852d75, 7cc5a312 | Vfx{Float,Vector2,Vector3,ColorRgb,Color}AnimatedWithRandomDriver | one middle `AnimatedWithRandom` reproduces five independent hashes | 3 |
| fbef6376, c5b349bf | VfxFloat / VfxColorAnimatedWithRandomMaterialDriver | the shared stems `0x6700ffbd` / `0x036d3b0c` with `1d04cfa7` / `7cc5a312` (vfx-driver-graph-math.md) | 3 |
| 2959e51d, 5ff9a600, 7a39d82b, fc39b68c | Vfx{Float,Vector2,Vector3,Color}AnimatedWithRandomByFactorDriver | one middle `AnimatedWithRandomByFactor` reproduces four hashes; each registers directly before its time-sampled sibling | 3 |
| 9a2d73f2, def9bfd5, 7c387678 | VfxSplatFloatOntoVector{2,3,4}Driver | the proven shared stem `0x94de5689`; one hit in a 151k-form lattice (chance 3.5e-5); sorts `VfxSpawnBehavior` .. `VfxSquareRootRangedRandomDriver` | 4 |
| 88406627 | VfxAttachedBoneScaleDriver | one hit in a 25k-form lattice over all targets (chance 2e-4); returns the scale of the owner's attachment joint; sorts `VfxAssetRemap` .. `VfxAudioComponentBase` | 4 |
| 134d9e65 | LocalPlayerMaterialLogicDriverSource | `hash-guesser force` depth 2 (run noise 1.6); its only virtual returns the local player's champion | 4 |
| 5f357aaf | IsContentCensoredFloatDriver | same run; sorts in the narrow bracket `IsCastingBoolDriver` .. `IsEmoteMutedBoolDriver`; Evaluate is a stub that returns 0, so the reader cannot confirm it | 4, weak |

The middle-run search: for one member, a meet-in-the-middle over `prefix + X + suffix` with X
up to four words of the split wordlist plus about 100 VFX words (1.2e14 effective probes for
one target). Only an X that also reproduces a sibling hash, with the sibling's own prefix and
suffix, counts. A proven shared-stem twin cannot serve as the check, because it agrees with
every X.

Positive control: the same search returns `VfxFloatOverLifeMaterialDriver` from
`VfxColorOverLifeMaterialDriver`.

## Negatives

Expected chance hits in brackets.

- All targets: `identity` and `delete` (0.001); identifier substrings of client binaries, 484k
  forms (0.004); a 25k-form semantic lattice (0.0002).
- `7e173e2f`, `34262325`, `b5c28890`, `e67a27c8`: `force` depth 2 with 7 prefixes and 15
  suffixes (1.6); `chain` depth 3 and 4 (0.7); semantic lattices of 25k-135k forms per target;
  bracketed meet-in-the-middle: `MapVisibility` + 3 words (3.7 per suffix, 4 suffixes),
  `Ge..`/`Gh..`/`Ha..` + 4 words (580 per suffix), `LogicDriver` + `A..`/`B..`/`Ch..` + 3 words
  (1.2 per suffix). All hits were noise.
- Fields: `force` depth 2 with `""`/`m`/`b` (0.07), `chain` depth 3 and 4 (0.05), `force` depth 3
  over the top 700 (0.72); per-field lattices of 600-47k forms.
- Held back: `0c011585` = `ValueSeparatorTextFormat` (`chain` depth 4, noise 0.05). The field
  holds a text source object, not a format string.

## What is left

| hash | what is known |
|---|---|
| 7e173e2f | float, `VisibilityController`; returns the fade weight of a map visibility controller; registers between `MapVisibilityLogicBoolDriver` and `MapVisibilityTransitionLengthDriver` |
| 34262325 | float, `AnimationName`; returns the playback speed scale of that clip; registers between `GearSelectionViewController` and `HasValueChanged` |
| b5c28890 | bool over a champion filter (relation `PlayersToInclude`, roles `6f521854`); any hero passes |
| e67a27c8 | text twin of `b5c28890`: names of the heroes that pass, joined |
| 6d613e54 | Vec3 driver graph node, one bool field `cc42a2fb`; not read |
| fields | `267c4080` (selects the other skin record), `6f521854` (role ids 0-5), `9c5f0056` (ignore disguise), `08f7bb6e` (record selector), `149271dd` (also scan a second buff list), `3ef62dce` (combat window, whole seconds), `8370ee35`, `4742b028`, `0c011585`, the four `Float4FromLogicDrivers` lane selectors (state `0x0B1D7668` before a final `x`/`y`/`z`/`w`) |

## Status

All rows `pending`. Tier 3 rows are fit to submit upstream. Tier 4 rows go in a separate,
flagged part of any upstream PR.

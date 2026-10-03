# logic-drivers - campaign record

Record for `logic-drivers`; its `batches.jsonl` note points here. Method: the
`crack-family` skill (`.claude/skills/crack-family/SKILL.md`). `ILogicDriver`
went from 66 live unnamed members to 23. 46 names landed: 43 classes and 3
fields.

The base fixes the return type and so the last word: `<X>Driver` /
`<X>MaterialDriver` (Vec4), `<X>FloatDriver`, `<X>IntDriver`, `<X>BoolDriver`,
`<X>Vector3Driver`, with Bool under Int under Float a widening chain. The 68
named siblings are all upstream, so the convention is externally attested.
`bin-grep --class ILogicDriver --subclasses` returns 154,958 shipped objects;
29 of the 66 unnamed had at least one instance, and reading the graphs they
sit in named a third of the batch.

## Runs

Suffix folding: 13 suffixes over 66 targets, 858 unfolds against ~13k states,
0.003 expected noise. Five classes unfold to one shared stem equal to the
known name `empty`; further stems equal `Sequence`, `ValueArray`, `MoveSpeed`
and `spellData`; two classes share a second stem under `MaterialBoolDriver` /
`MaterialFloatDriver`.

Shipped strings: the CommunityDragon corpora (2,820,530 forms, 723 states,
0.475 expected noise) returned **zero** - a clean negative, driver class names
do not appear in shipped asset paths or bin strings. The client binaries
(468,639 forms, 0.079) returned six, five already implied by structure:

| stem | shipped string | what it names |
|---|---|---|
| `EMPTY` | `EMPTY` | the five-slot `EmptyLogic*` lattice |
| `Sequence` | `Sequence` | the three `SequenceMaterial*` classes |
| `MoveSpeed` | `Character_MoveSpeed` | `MoveSpeedFloatDriver` |
| `IsClone` | `IsClone` | `IsCloneBoolDriver` |
| `math` | `... freeform math block` | `MathFloatDriver` and its four children |

Lattice completion: `Math` crossed with 40 operator words and 15 suffixes,
1,230 probes at 0.0007, returned all four arithmetic children at once. `Raw`
crossed with the five concept types, then `Eased` by hand, closed a 5x2 block
in 2,320 probes.

Guesser runs, quietest first; the prefixes came out of the passes above - a
stem proved on one slot anchored the next run (`--prefix Bounding` and
`--prefix Move` are why the depth-1 run at 0.008 noise reached
`BoundingRadiusFloatDriver` and `MoveVelocityVector3Driver`):

| run | probes | states | expected noise | candidates |
|---|---|---|---|---|
| `identity` + 16 suffixes | 90,084 | 800 | 0.017 | 3 |
| prefixed `force` depth 1 | 44,254 | 800 | 0.008 | 5 |
| `chain` depth 3 | 852,086 | 800 | 0.159 | 6 |
| prefixed `force` depth 2, top 400 | 2,245,600 | 800 | 0.418 | 11 |
| `Concept`-anchored `force` depth 2 | 19,990,164 | 100 | 0.465 | 5 |
| `MapVisibility`-anchored `force` depth 2 | 29,985,246 | 14 | 0.098 | 2 |
| `force` depth 2, full wordlist | 19,990,164 | 800 | 3.723 | 13 |
| `chain` depth 4 | 19,673,274 | 800 | 3.664 | 9 |

The remaining names came from reading the shipped graph - the pass that
produced `IsInWaterBoolDriver` and `IsStealthedBoolDriver`, neither of which
any search reached.

## The lattices

Each row is one stem; every cell is a name this batch added.

| stem | Vec4 | Float | Int | Bool | Vector3 |
|---|---|---|---|---|---|
| `EmptyLogic` | `EmptyLogicDriver` | `EmptyLogicFloatDriver` | `EmptyLogicIntDriver` | `EmptyLogicBoolDriver` | `EmptyLogicVector3Driver` |
| `Raw<T>Concept` | `RawVector4ConceptLogicDriver` | `RawFloatConceptLogicDriver` | `RawIntConceptLogicDriver` | `RawBoolConceptLogicDriver` | `RawVector3ConceptLogicDriver` |
| `Eased<T>Concept` | `EasedVector4ConceptLogicDriver` | `EasedFloatConceptLogicDriver` | `EasedIntConceptLogicDriver`* | `EasedBoolConceptLogicDriver`* | `EasedVector3ConceptLogicDriver` |
| `SequenceMaterial` | `SequenceMaterialVectorDriver` | `SequenceMaterialFloatDriver` | - | `SequenceMaterialBoolDriver` | - |

\* the two starred classes derive from `ILogicFloatDriver`, not from the base
their concept type would predict, and that is the argument for `Eased`: easing
a bool or an int produces a float, easing a float, Vec3 or Vec4 does not
change the type. All ten carry exactly one field, `.Concept: Link <T>Concept`,
all ten were introduced in 15.11, and `<T>Concept` carries `.EasingData`.

The `EmptyLogic` row is a whole patch cohort: 16.5 introduced exactly eight
drivers, five of them these, and all five are field-free.

The arithmetic block, whose interface is `MathFloatDriver` and whose children
each add one operand field naming the operation. An earlier attempt crossed
the four operator words with the family's two suffixes and missed - the stem
carries the interface's own word in front of the operator, which `--prefix
Math` supplied:

| hash | name | the field that names it |
|---|---|---|
| `f9e5b8b9` | `MathFloatDriver` | `.value`, the left operand, on the interface |
| `5b2fdd66` | `MathAddFloatDriver` | `.add` |
| `53dfc5b5` | `MathSubtractFloatDriver` | `.Subtract` |
| `ef339ef9` | `MathMultiplyFloatDriver` | `.multiplier` |
| `cc35f742` | `MathDivideFloatDriver` | `.Denominator` |

## Attested by the shipped graph

The strongest single piece of evidence in the batch is one expression out of
`ClientStates/Gameplay/UX/LoL/Skins/KaisaSkin71ViewController`:

```
    ValueDriver = 0x5b2fdd66 {                          -> MathAddFloatDriver
      value = 0xcc35f742 {                              -> MathDivideFloatDriver
        value       = 0xe622d482 { }                    -> PlayerGoldFloatDriver
        Denominator = 0xbafc3e15 {                      -> SpellDataNamedValueFloatDriver
          Spell     = 0xcce5bf1b
          ValueName = "GoldToNextForm"
          SpellLevel = 0x608a3ee7 { }                   -> still open
        }
      }
    }
```

A field-free float driver divided by a spell value Riot named
`GoldToNextForm`, feeding a progress meter on a skin that upgrades with gold.
That fixes `PlayerGoldFloatDriver` and `MathDivideFloatDriver` together, and
`SpellDataNamedValueFloatDriver` is the `.ValueName`-carrying child of the
`.Spell` + `.ScriptName` interface `SpellDataFloatDriver`.

The rest, one row per name:

| hash | name | what attests it |
|---|---|---|
| `f5821f8b` | `IsInWaterBoolDriver` | on Aatrox skins 33-39, LeeSin and `Strawberry_Boss_Aatrox`, it gates `Aatrox_Skin33_{Q,Q_Cast3,E,R}_Cast_InWater`, four of the five persistent Vfx it conditions |
| `77b42f3f` | `IsStealthedBoolDriver` | picks `Run2` over `Run_Base` on Teemo, Jade_Teemo and Evelynn, the camouflage and Demon Shade champions |
| `d4933ea0` | `IsCloneBoolDriver` | shipped string `IsClone`; instances on Shaco skins 71 and 72 |
| `635d04b7` | `IsChampionNameBoolDriver` | `.championName = "Characters/Neeko"`, `"Characters/Karma"` |
| `fe70e9c4` | `IsInCombatDynamicMaterialBoolDriver` | `.CombatGroup: U8 = 2`, wired beside the above under a `NotMaterialDriver` |
| `18278f59` | `HasUnitTagBoolDriver` | `.UnitTags: Embed ObjectTags` plus `.Has`, beside `HasGearDynamicMaterialBoolDriver` on Leona |
| `0820c053` | `HasSkinAugmentBoolDriver` | `.SkinAugment: Hash` and nothing else |
| `ed5506a6` | `TftTeamCannonMaterialDriver` | ships in exactly one asset, `Characters/TFT_TeamCannon/Skins/Skin0/Materials/TFT_Glow_Items_inst`; its `.success` / `.Failure` / `.Idle` / `.INACTIVE` are the cannon's states, and upstream already names `TftTeamCannonCooldownViewController` |
| `f9c21175` | `IgnoreDead` | Bool on `DistanceToPlayerMaterialFloatDriver`, default False |

## Everything else the pass named

| hash | name | what fixes it |
|---|---|---|
| `1792fdb5` | `ValueArrayFloatDriver` | unfolds to its own field name, `.ValueArray`, beside an index driver |
| `2fe549a9` | `MoveSpeedFloatDriver` | stem is the known field `MoveSpeed`, shipped as `Character_MoveSpeed`; 16.5 cohort |
| `8dda74d7` | `MoveVelocityVector3Driver` | same cohort, the Vec3 beside the speed scalar |
| `e90ab695` | `BoundingRadiusFloatDriver` | same cohort; 16.5's eight drivers are now all named |
| `ae4d057b` | `BoundingBoxSizeVector3Driver` | the Vec3 counterpart, 16.3 |
| `f3ddd31e` | `CharacterStatFloatDriver` | `.Stat: U8 = 10` selects the stat; newest class in the family, 16.15 |
| `73425974` | `FacingAndMovementAngleFloatDriver` | stem is the whole of upstream's `FacingAndMovementAngleParametricUpdater` |
| `1db6d987` | `MapVisibilityLogicBoolDriver` | `.VisibilityController: Link IMapVisibilityController` |
| `ca458424` | `MapVisibilityTransitionLengthDriver` | same field, Float return; `MapVisibilityFlagDefinition.TransitionTime` is the value it reads |
| `18291220` | `ComponentWeight` | `Vec3 = [1,1,1]` on a float driver reading three named strings |
| `44146c9d` | `MissileSpellObjects` | `List2 Hash` on a `JointOrientationRigPoseModifierData.OrientationSource` for Viktor's backarm, and on an `IFloatParametricUpdater` with the same shape |

**The weakest.** `MapVisibilityTransitionLengthDriver` spells "Length" where
the rest of that subsystem spells `TransitionTime` and `TransitionTimeDriver`.
Its run carried 0.098 expected noise and returned two hits, the other being
incoherent, so chance is a 0.45% explanation for the pair, but the name rests
on the hash alone and no shipped instance exists. Anyone submitting this
upstream should split it off or say so. `ComponentWeight` and
`MissileSpellObjects` are the same shape one tier better: quiet runs, coherent
structure, no string.

## What is left

23 live members, and 24 unresolved field hashes across them. Ruled out for all
23, so nobody re-treads: `identity`, `delete`, `chain` depth 3 and 4, `force`
depth 2 over the full wordlist, `force` depth 3 over the top 300, the whole
CommunityDragon string corpus (2.8M forms) and both client binaries (469k),
each split by return type so the suffix set stayed tight. Summed expected
noise about 12, zero coherent hits beyond what is listed above.

The three with the most context, and the most likely to fall next:

    fd51006c  I  interface, .IsExclusive .DelayOrder; the Ambessa input-buffer
              condition. Children fb16e4be (.OrderTypes: List2 U8 = {2,3,7},
              used as .InputCondition) and 0d91a223 (.0xe2e5b6dd: List2 U32,
              used as .DisplayCondition)
    b0be1066     Vec4 composer: .XDriver .YDriver .ZDriver .WDriver plus four
              Pointer 0x315aff8e modifiers {ModifierType, ModifierDriver} and
              four U8 modes. Ships in ViegoSkin43ViewController
    608a3ee7     field-free Int; feeds .SpellLevel on
              SpellDataNamedValueFloatDriver and indexes a tooltip ValueArray,
              so it returns a spell's current level. `SpellLevelIntDriver`,
              `SpellRankIntDriver` and 11 more arrangements all miss

Singletons, with what is known about each:

    c8d666b4     field-free Bool; the mBoolDriver of ColorChooserMaterialDriver
              in 87 TFTSet13 `RebelStatus*` materials and 39 TFTCommon
              `FrozenStatus` ones, switching a whole tint set on and off
    c83eb239     field-free Bool; ORs with a buff counter to show Sett skin 66's
              GritMeter scene
    83a9f4f8     field-free Bool; the whole Updater of a ConditionBoolClipData
              on the Ahri 86, Kaisa 71 and Leblanc 55 animation graphs
    9bc366ca     .SkinID: U32, .UseValidParentForChroma; ORed with
              IsChampionNameBoolDriver, values 54 and 70 on Karma
    b5c28890     four unresolved fields, two List2 U32; the EnableCondition of
              six Viego skin 43 views
    b7b43e1d     .Percentage: F32 = 0.5, .BoolDriver
    b2a6e394     .LogicDriver: Pointer ILogicDriver, returns Bool
    a8dd91e9     .mBoolDriver: Pointer ILogicBoolDriver, returns Float
    cb6a541      .GameplayTexture: Hash
    34262325     .AnimationName: String; a PersistentVfx field on Viego skin 43
    3eac408c     three Strings plus the ComponentWeight Vec3; 16.12. Its three
              String hashes b91d14d4, ee255a71, ef335b4 are the most heavily
              searched and most stubborn hashes in the family: `chain` depth 4
              and 5, `force` depth 3 over the top 600 under both the empty and
              the `m` prefix (675M probes, 0.47 summed expected noise), 13,451
              hand-written indexed bone schemes (`BoneA`/`StartBone`/`mJoint1`
              and 40 more prefixes crossed with 13 stems), 7,600 non-bone
              readings, and both string corpora. Zero hits on all of it, so the
              class's field vocabulary is reachable from nothing we know
    de125976     .minDistance 100.0, .maxDistance 1000.0, one Bool; Vayne skins
    7e173e2f     .VisibilityController, Float; the third of that trio
    19da44b2     .MissileSpellObjects: List2 Hash; the OrientationSource of a
              JointOrientationRigPoseModifierData on Viktor's C_Backarm3
    6a97ad3      field-free Vec3; the OrientationSource of a FaceTarget event
              on two TFTSetEvent5YR attack clips
    4f92775c     field-free Vec3, 14.10, no shipped instance
    fba9327c     field-free Float, 15.15, no shipped instance
    2ef7017      .0xaa13ab5a: U8, 15.15, no shipped instance

## Reader pass

A second pass, led by the code instead of the hash. Each driver's `Evaluate` was
read in the 16.18 Windows client, the behaviour gave the vocabulary, and a small
bounded search did the rest. Three batches: `logic-driver-readers`,
`spell-preview`, `logic-driver-view-updates`, plus one held-back name in
`logic-drivers-unproven`. 34 classes and 30 fields. None is attested by a string.

Drivers and their data classes (`logic-driver-readers`):

| hash | name | what the reader does |
|---|---|---|
| `a8dd91e9` | `EnabledTimerFloatDriver` | seconds its `mBoolDriver` has been true; one hit in about 19M candidates |
| `e78a175a` | `BoundingBoxLengthFloatDriver` | half the diagonal of the bounding box; registered directly before `BoundingRadiusFloatDriver`, and the registrars are in alphabetical order |
| `de125976` | `DistanceToEnemyMinionMaterialFloatDriver` | `DistanceToPlayerMaterialFloatDriver` with lane minions of the opposing team as targets |
| `fba9327c` | `LastDamageAmountFloatDriver` | a float written from the unit's death record |
| `02ef7017` | `LastDamageTypeMaterialBoolDriver` | compares `.DamageTypeToMatch` with a byte from the same record; the pair arrived together in 15.15 |
| `3eac408c` | `BoneDistanceFloatDriver` | distance between `.BoneAName` and `.BoneBName` in the frame of `.BoneBasisName`, weighted by `ComponentWeight` |
| `e6d1f14b` | `BoneTransformFloatDriver` | one translation or scale channel (`.BoneTransformChannel`) of `boneName` relative to `.BoneBasisName` |
| `55383fd3` | `AudioFloatDriver` | output of the `AudioRtpcObject` linked by `.RtpcObject` |
| `2f5be991` | `AudioRtpcObject` | reads one global RTPC by `rtpcName`; `.InputMinDb` / `.InputMaxDb` (defaults -48 and 0, one stem for both), `.AdaptationTime`, `.AdaptationZoom`, `.AttackSeconds`, `.ReleaseSeconds`, `.ResponseCurve` |
| `612b4ce2` | `ConceptEasingData` | the class behind every `<T>Concept.EasingData` |
| `3a302e74` | `ArrayIndexDriver` | the index operand of `ValueArrayFloatDriver` and of its string twin; one hit in about 420k |
| `4ee81483` | `UseRawValue` | on `CharacterStatFloatDriver`: raw stat instead of the 0..1 normalised one |
| `eaf5370d`, `9dba9f88`, `58074a16` | `PlaySpeedModifierLogicDriver`, `PlaySpeedModifierValueType`, `PlaySpeedModifierSource` | on `PersistentVfxData`, beside the known `PlaySpeedModifier` |

The three bone-name fields were the most searched hashes of the first pass. They
fell once the reader showed two endpoints and a basis joint: the stem is
`bone<X>Name`, with `A`, `B` and `Basis`.

`spell-preview`: the unnamed source class `239d0a76`, built inline in a per-unit
spell preview component, is `SpellPreviewSource`. The same stem fills six more
slots around the known `SpellPreviewData`: `SpellPreviewConditionData`
(`fd51006c`, the interface from "What is left"), its children
`SpellPreviewOrderConditionData` (`fb16e4be`) and
`SpellPreviewInputLockConditionData` (`0d91a223`), `SpellPreviewConfigData`
(`280745b1`), `SpellPreviewEffectData` (`55f6bf86`) and `SpellPreviewParamsData`
(`c7e628b9`). `CharacterRecord.dd661aab` is the field `SpellPreviewData`.

`logic-driver-view-updates`: the classes a `LogicDriverViewEntry` holds. Every
`I*` name lands on an interface with the predicted children:
`ILogicValueDriverUpdateViewElements` -> `LogicFloatDriverUpdateViewElements`,
`LogicDriverUpdateViewElements`; `IValueUpdateModifier` -> `IFloatUpdateModifier`
-> `DelayFloatUpdate`, `IInterpolateFloatUpdate` -> `InterpolateDeltaFloatUpdate`,
`InterpolateDurationFloatUpdate`; `Vector4UpdateModifier` with
`X/Y/Z/WUpdateModifier`; `ILogicDriverValueToString` -> `LogicDriverFloatToString`,
`LogicDriverValueToStringArray`; `IFloatUpdateElement` under the known
`IValueUpdateElement`; `LogicDriverFloatModifier`; `LogicDriverElementTooltip`.
Fields: `LogicDriverElementUpdates`, `LogicDriverElementMaterialSource`,
`ViewElementsToUpdate`, and `X/Y/Z/WValueModifier` on `b0be1066`.

**Held back.** `Float4FromLogicDrivers` for `b0be1066` (the Vec4 composer) came
from a wide search with about 1 in 5 odds of a chance hit. Do not submit it.

Bounded searches run with the reader vocabulary, for the record: classes, 1 to 3
free words over 708 words and 4 suffixes, 0.77 expected noise, one incoherent hit
rejected; fields, 2 words over 700 with 8 prefixes, 0.010, one hit
(`AdaptationZoom`); fields, 3 words over 260, 0.17, two hits sharing a stem
(`InputMinDb`, `InputMaxDb`).

Still open after this pass, with what the reader shows:

    34262325     returns the playback speed scale of the animation named by
              .AnimationName; the only value ever placed in
              PersistentVfxData.PlaySpeedModifierLogicDriver
    7e173e2f     returns the current fade weight (0..1) of a map visibility
              controller; about 170 MapVisibility<X>Driver forms miss
    608a3ee7     field-free Int, see above
    9af7b542     the key object of the SequenceMaterial* drivers, one PathHash
    8370ee35     Bool on both distance drivers: only count targets the source
              can see; about 1.5M candidates miss
    4742b028     Bool on BoneTransformFloatDriver: scale translation by 0.1 and
              mirror X
    0ed45b9e     F32 on ConceptEasingData: the ease duration in seconds
    050e3899, 65f22822   F32 on AudioRtpcObject: output multiplier, unknown
    8f60db30, 9060dcc3, 9160de56, 9e60f2cd   U8 component selectors on
              b0be1066; one stem (state 0b1d7668) plus x, y, z, w

## Status

All 46 rows of the first pass and all 64 of the reader pass are `status=pending`,
`pr=-`. Nothing submitted upstream.

Fit to submit as it stands: the four lattices, which are proved by filling
every slot of a row at once rather than by any single hash, and the nine rows
attested by a shipped string or a shipped driver graph. The remainder carry a
slot argument, which is the standard `map-entity-templates` and
`viewcontroller-family` met; `MapVisibilityTransitionLengthDriver` is flagged
above and is the one name in the batch that carries neither.

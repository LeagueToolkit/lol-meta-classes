# The logic driver pass

Reversing doc for `logic-drivers`. Its `batches.tsv` note points here.

`ILogicDriver` went from 66 live unnamed members to 23. 46 names landed: 43
classes and 3 fields.

## What the family is

A driver graph. `ILogicDriver` is the root of a small expression language the
engine evaluates every frame, and the base a class derives from *is* its return
type:

```
    ILogicDriver           I  Vec4 / colour        <X>Driver, <X>MaterialDriver
      ILogicFloatDriver    I  F32                  <X>FloatDriver
        ILogicIntDriver    I  I32                  <X>IntDriver
          ILogicBoolDriver I  bool                 <X>BoolDriver
      ILogicVector3Driver  I  Vec3                 <X>Vector3Driver
```

`ILogicIntDriver` under `ILogicFloatDriver` and `ILogicBoolDriver` under
`ILogicIntDriver` is a widening chain, so a bool driver is usable anywhere a
float is wanted. That is the single most useful fact in the family: **the base
class fixes the last word of the name**, which turns most of the search into a
one-word problem under an anchored suffix.

Shipped instances are everywhere: `bin-grep --class ILogicDriver --subclasses`
over the 455 installed WADs returns 154,958 objects, mostly
`StaticMaterialDef.dynamicMaterial.parameters[N].driver`,
`SkinCharacterDataProperties.PersistentEffectConditions[N].OwnerCondition`,
`AnimationGraphData.mClipDataMap{...}.Updater.driver` and
`LogicDriverViewController.Views[N]`. 29 of the 66 unnamed classes have at least
one instance, and reading the graph they sit in is what named a third of them.

## Method

Five passes, in the order they were run.

**1. Suffix folding.** FNV-1a's prime is invertible mod 2^32, so `unfold(h, s)`
is the state a name was in before suffix `s` was appended. Folding 13 driver
suffixes out of the 66 targets and comparing the states costs 858 unfolds
against ~13k known-name states, **0.003 expected chance hits**. Three things
fell out:

- five classes unfold to **one shared stem** under five different suffixes;
- that stem equals the known name `empty`, and three more stems equal
  `Sequence`, `ValueArray`, `MoveSpeed` and `spellData`;
- two classes share a second stem under `MaterialBoolDriver` /
  `MaterialFloatDriver`.

**2. Shipped strings.** Every identifier-shaped token in CommunityDragon's
`hashes.bin{hashes,entries}.txt` and `hashes.game.txt.*` (2,820,530 distinct
probe forms) against 723 target states: **0.475 expected chance hits, zero
actual**. A clean negative, and worth recording: driver class names do not
appear in shipped asset paths or bin strings.

The retail client binaries are a different story. 468,639 distinct forms at
**0.079 expected chance hits** returned six, five of which the structure already
implied:

| stem | shipped string | what it names |
|---|---|---|
| `EMPTY` | `EMPTY` | the five-slot `EmptyLogic*` lattice |
| `Sequence` | `Sequence` | the three `SequenceMaterial*` classes |
| `MoveSpeed` | `Character_MoveSpeed` | `MoveSpeedFloatDriver` |
| `IsClone` | `IsClone` | `IsCloneBoolDriver` |
| `math` | `... freeform math block` | `MathFloatDriver` and its four children |

**3. Lattice completion.** A stem that fills several slots at once is
`(p/2^32)^n`-competitive rather than `p/2^32`, so once a stem is known the rest
of its row is nearly free. `Math` from pass 2 crossed with 40 operator words and
15 suffixes is 1,230 probes, **0.0007 expected chance hits**, and returned all
four arithmetic children at once. `Raw` crossed with the five concept types the
same way, then `Eased` by hand, closed a 5x2 block in 2,320 probes.

**4. `hash-guesser`, anchored.** Recorded per run, quietest first:

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

The prefixes came out of passes 1-3: a stem proved on one slot is the anchor for
the next run. `--prefix Bounding` and `--prefix Move` are the reason the
depth-1 run at 0.008 noise reached `BoundingRadiusFloatDriver` and
`MoveVelocityVector3Driver` at all.

**5. Reading the shipped graph.** The remaining names came from what the driver
is wired to. This is the pass that produced `IsInWaterBoolDriver` and
`IsStealthedBoolDriver`, neither of which any search reached.

## The lattices

Each row is one stem; every cell is a name this batch added.

| stem | Vec4 | Float | Int | Bool | Vector3 |
|---|---|---|---|---|---|
| `EmptyLogic` | `EmptyLogicDriver` | `EmptyLogicFloatDriver` | `EmptyLogicIntDriver` | `EmptyLogicBoolDriver` | `EmptyLogicVector3Driver` |
| `Raw<T>Concept` | `RawVector4ConceptLogicDriver` | `RawFloatConceptLogicDriver` | `RawIntConceptLogicDriver` | `RawBoolConceptLogicDriver` | `RawVector3ConceptLogicDriver` |
| `Eased<T>Concept` | `EasedVector4ConceptLogicDriver` | `EasedFloatConceptLogicDriver` | `EasedIntConceptLogicDriver`* | `EasedBoolConceptLogicDriver`* | `EasedVector3ConceptLogicDriver` |
| `SequenceMaterial` | `SequenceMaterialVectorDriver` | `SequenceMaterialFloatDriver` | - | `SequenceMaterialBoolDriver` | - |

\* the two starred classes derive from `ILogicFloatDriver`, not from the base
their concept type would predict, and that is the argument for `Eased`: easing a
bool or an int produces a float, easing a float, Vec3 or Vec4 does not change the
type. All ten carry exactly one field, `.Concept: Link <T>Concept`, all ten were
introduced in 15.11, and `<T>Concept` carries `.EasingData`.

The `EmptyLogic` row is a whole patch cohort: 16.5 introduced exactly eight
drivers, five of them these, and all five are field-free.

The arithmetic block, whose interface is `MathFloatDriver` and whose children
each add one operand field naming the operation:

| hash | name | the field that names it |
|---|---|---|
| `f9e5b8b9` | `MathFloatDriver` | `.value`, the left operand, on the interface |
| `5b2fdd66` | `MathAddFloatDriver` | `.add` |
| `53dfc5b5` | `MathSubtractFloatDriver` | `.Subtract` |
| `ef339ef9` | `MathMultiplyFloatDriver` | `.multiplier` |
| `cc35f742` | `MathDivideFloatDriver` | `.Denominator` |

[docs/unnamed-families.md](unnamed-families.md) records an earlier attempt that
crossed the four operator words with the family's two suffixes and missed. The
stem carries the interface's own word in front of the operator, which is what
`--prefix Math` supplies and plain recombination at that depth does not.

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

A field-free float driver divided by a spell value Riot named `GoldToNextForm`,
feeding a progress meter on a skin that upgrades with gold. That fixes
`PlayerGoldFloatDriver` and `MathDivideFloatDriver` together, and
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

**The weakest.** `MapVisibilityTransitionLengthDriver` spells "Length" where the
rest of that subsystem spells `TransitionTime` and `TransitionTimeDriver`. Its
run carried 0.098 expected noise and returned two hits, the other being
incoherent, so chance is a 0.45% explanation for the pair, but the name rests on
the hash alone and no shipped instance exists. Anyone submitting this upstream
should split it off or say so. `ComponentWeight` and `MissileSpellObjects` are
the same shape one tier better: quiet runs, coherent structure, no string.

## What is left

23 live members, and 24 unresolved field hashes across them. Ruled out for all
23, so nobody re-treads: `identity`, `delete`, `chain` depth 3 and 4, `force`
depth 2 over the full wordlist, `force` depth 3 over the top 300, the whole
CommunityDragon string corpus (2.8M forms) and both client binaries (469k),
each split by return type so the suffix set stayed tight. Summed expected noise
about 12, zero coherent hits beyond what is listed above.

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

## Status

All 46 rows `status=pending`, `pr=-`. Nothing submitted upstream.

Fit to submit as it stands: the four lattices, which are proved by filling every
slot of a row at once rather than by any single hash, and the nine rows attested
by a shipped string or a shipped driver graph. The remainder carry a slot
argument, which is the standard `map-entity-templates` and `viewcontroller-family`
met; `MapVisibilityTransitionLengthDriver` is flagged above and is the one name
in the batch that carries neither.

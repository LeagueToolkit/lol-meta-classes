# The semantic pass

The reversing doc for three batches: `semantic-pass-structural`,
`semantic-pass-family` and `viewcontroller-family`. Their `batches.tsv` notes
point here.

## Why it is not a guesser run

The guesser recombines words taken from names we already know, so it cannot
reach a name containing a word the corpus has never used, at any depth. This
pass takes proposals from each unnamed class's structural neighbourhood in the
meta instead - referencing fields, member types, base and sibling stems, its own
field names - and checks those against the unresolved hashes.

The hash match is the filter, not the evidence. Nothing here is a name a
generator invented and a hash blessed.

## Noise budget

Expected chance hits are `probes x target states / 2^32`, and a structure-derived
proposal list is small enough that the number stays under one:

| pass | proposals | states | expected chance hits | landed |
|---|---|---|---|---|
| structural + family | 417k | ~7,300 | 0.71 | 254 |
| viewcontroller | 1.18M | ~7,100 | 1.96 | 11 |

"Landed" means hit an unresolved hash, which is not the same as shipped: 230 of
the 254 are in this repo's tables and 24 were not taken in review. See
"Proposed and not taken" below.

The generator was calibrated against ground truth first: run over classes that
are *already* named, it reproduced 2,080 of them at their exact spelling.

The ViewController pass had an in-family refutation available on top of that - a
real hit has to land on a class whose base actually is `ViewController`. Exactly
2 of the 11 fell outside, which is the predicted noise almost exactly. Both are
recorded in `hashes/bad/semantic-pass.txt` so they cannot be re-proposed:

    2b8568f9 BattlegroundsSearchCompanionViewControllerEvents
    9ab63a0b PlayerLoadableSpellViewControllerData

## What attests each batch

**`semantic-pass-structural`** (100: 24 classes, 76 fields). Proposed from the
declaring class's own neighbourhood, and every name resolves a field or class
the meta declares.

**`viewcontroller-family`** (9 classes). The base really is `ViewController`,
and the class's own members carry the feature word: `.TeamEditorData` for
`TftTeamEditorViewController`, `.Day`/`.Night` for
`MonarchDayNightViewController`, child `MinimapViewController` for
`MinimapViewControllerBase`.

**`semantic-pass-family`** (130: 28 classes, 102 fields). Family consistency
only: the class completes the naming pattern of its base/sibling family, or the
field is multi-word and coherent on a named owner. No second method confirms
these individually. They resolve correctly, but by this repo's own rule they are
not fit for a CDragon PR as they stand, and should be split off or renamed
`-unproven` first, matching `light-regions-unproven` and `map-graphics-unproven`.

## What is *not* attested per name

Worth stating plainly, because the batch structure invites over-reading. Of the
field names, counting a name as type-attested when its own declared type or its
declaring class's name repeats one of its words:

| batch | fields | attested by own type | by owner class name | neither |
|---|---|---|---|---|
| `semantic-pass-structural` | 76 | 20 | 1 | 55 |
| `semantic-pass-family` | 102 | 2 | 7 | 93 |

A primitive-typed field on an unnamed class has no type to echo it, so "neither"
is the expected outcome for most of them rather than a red flag - but it does
mean the proposal's own plausibility is doing the work for those names, and the
per-batch claim above is the only thing standing behind them.

The weakest are the single-word ones with no named sibling and a primitive type:
`Event` (String), `Killer` (U8), `Stat` (U8), `Quest` (Hash), `Provider` (Link),
`Has` (U8). Each is individually about 1.7e-6 to collide, so the *hashes* are
almost certainly real; that is not the same as the *names* being right.

## Proposed and not taken

24 of the 254 hits did not enter the tables: 16 classes and 8 fields. All 24
still resolve nothing, here or upstream.

    0x1e3c6622 EventPass                     0xc3a01413 AudioBank
    0x27bc6378 AugmentSet                    0xe78d94fa PortraitSet
    0x049233a8 GameEntityType                0xf301caaa RedGeComponentDef
    0x4ca99280 AudioContextEventType         0xbd6e6d8a GreenGeComponentDef
    0x5566d3a3 GameEntityGroup               0x712af79e IconGeComponentDef
    0x5784d4c7 LolGameEntityTemplate         0x8b54c9a7 SeqInputObject
    0x692bf354 ITimerControllerDefinition    0x94e82d1f SeqInputVector
    0x880c52da RegionBoundaryInstanceParams  0xb87473b5 LolInputUpdater

    0xfbe86875 Channels                      0xc04df9d4 FloatCurve
    0xcb1e8bfc CurveData                     0x2642955f ForwardAxis
    0x44ee0c82 EntityTag                     0xe92580fe OrientationMode
    0x20407665 FadeOut                       0x9b15d5eb SSAO

They are recorded here and **not** in `hashes/bad/semantic-pass.txt`, which is
deliberate. That file is for names a review actively refuted, and `--bad` blocks
them from ever being re-proposed; no per-name reason was recorded for these 24
at the time, so there is nothing to distinguish a refutation from a proposal
that merely was not confident enough. Blocking the second kind permanently on a
reason nobody wrote down would lose real candidates. Anyone revisiting these
should re-derive the call rather than trust the omission either way.

## The casing audit

FNV-1a lowercases, so casing is invisible to the hash: `LIST` and `List` are one
hash, and a generator's choice of spelling is not evidence of anything. Casing
has to be attested separately or it is invented.

Every word was re-counted against the corpus **with these batches excluded** -
the batches' own entries were the only support some spellings had - and against
the declaring class's own siblings. Fourteen names were recased on that check.

Word casing:

| from | to | batch | evidence |
|---|---|---|---|
| `CUSTOM` | `Custom` | structural | 79:0 |
| `FILE` | `File` | structural | 31:0 |
| `LIST` | `List` | structural | 190:0, own type `List Embed SpellLevelUpInfo` |
| `SFX` | `Sfx` | structural | 31:1, own type `SfxGeComponentDef` |
| `SPACING` | `Spacing` | structural | 4:0 |
| `STARTING` | `Starting` | structural | 16:0 |
| `TRANSITION` | `Transition` | structural | 104:0, embedded `TransitionAnimation` |
| `ButtonVFX` | `ButtonVfx` | family | 456:22 |
| `VFXIcon` | `VfxIcon` | family | 456:22 |
| `PIPIcon` | `PipIcon` | family | 42:2 |

Word boundary, where the corpus splits a compound the generator ran together:

| from | to | batch | evidence |
|---|---|---|---|
| `Taglist` | `TagList` | structural | `AudioTagListProperties`, `ChampionAugmentTagList`, `mTagList` |
| `Teamcolor` | `TeamColor` | structural | `mDeathEnemyTeamColor`, `mTeamColors` |
| `SequencerStatemachine` | `SequencerStateMachine` | family | `EntityStateMachine`, `StateMachineDef` |
| `TftpassID` | `TftPassId` | family | 12 `TftPass*` compounds |

The boundary fixes also pay the wordlist twice over: `Taglist` and `Tftpass`
were junk tokens, and `Tag`, `List`, `Tft`, `Pass` are all real words the
splitter can now reuse. `TeamColors` was in the same batch spelled correctly,
which is how the inconsistency surfaced.

`ID` versus `Id` was the one genuine disagreement. `GroupID` is kept, because
its own class carries upstream's `productID`. `TftPassID` had the same support
from a sibling `counterID` and was kept on that basis at first, but that class's
other upstream spellings are `name`, `leaderboard` and `tftpass`, so `counterID`
was a lone data point rather than a house style; corpus-wide the split is
`Id` 103 to `ID` 41. Recased to `TftPassId` on review.

## Status

All rows are `status=pending`, `pr=-`. Nothing has been submitted upstream.

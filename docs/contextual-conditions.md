# contextual-conditions

VO contextual conditions - the predicate classes a `ContextualRule` hangs off,
under two roots: `IContextualCondition` (14 live unnamed) and its character-scoped
sibling `ICharacterSubcondition` (4 live unnamed), 18 together.

**18 live unnamed -> 3.** Sixteen class names landed, one of them a dead class
outside the live 18, plus eight field names.

## What fixes each name

### Named off a shipped situation key or shipped instances (tier 1)

| hash | name | what fixes it |
|---|---|---|
| `b6da23cb` | ContextualConditionEnemyEncounterNumber | own fields `EncounterNumber`/`CompareOp`; 4068 of its 4079 shipped instances sit under `mSituations{EnemyEncounter}`, beside rules named per champion and `FirstEncounter3D<Champion>` VO events |
| `ac1764ca` | ContextualConditionDamageSelfMitigated | carries no fields at all; ships only under `mSituations{DamageSelfMitigated}`, as a bare `{}` gate |

### Own field name plus the situation it ships under (tier 3)

Seven landed in one 61-probe batch, each in the slot predicted for it
individually, which is what makes them a lattice rather than seven guesses.

| hash | name | what fixes it |
|---|---|---|
| `2363fb10` | ContextualConditionAnimationName | own field `AnimationName`; all 68 instances under `mSituations{AnimationEnded}` |
| `d55b5c23` | ContextualConditionDamageResultType | own field `DamageResultType`; ships under `DamageTakenAttempt` |
| `6a50b5d7` | ContextualConditionAbilityResourceSlot | own field `AbilityResourceSlot`; ships under `MaxResource` |
| `0dc0a353` | ContextualConditionAbilityResourceType | own field `AbilityResourceType`; ships under `MaxResource` |
| `01a4d9bd` | ContextualConditionCharacterHasSpellBuff | own field `SpellBuff`; Shyvana gates "(Dragon) Attack - Champion" on it and the plain "Attack - Champion" on the same instance wrapped in `ContextualConditionNegation`, which is what makes the verb `Has` |
| `b3420260` | ContextualConditionCharacterBuffStackCount | own fields `Count`/`Buff`/`CompareOp`/`BuffCaster`; Locke's instance reads `Buff=.../LockeQ, Count=3` under the rule "Q Hit Max Nails" |
| `4af7e9f2` | ContextualConditionKillComparison | two `0x7463e786` kill embeds plus `CompareOp`, the same shape as `70f5ed1c` one concept over; Ahri's two instances differ only in `CompareOp` and drive `Unique3DTransformAhead` / `Unique3DTransformBehind` |

### The gear rename boundary (tier 2)

`6ecc3b19` died at 15.17 and `061b427f` was born at 15.18 carrying **three of its
four field hashes**, one bool swapped. Paired spans meeting at one patch is the
strongest evidence this campaign produced, because chance cannot arrange it.

| hash | name | what fixes it |
|---|---|---|
| `6ecc3b19` | ContextualConditionGearId | dead, 14.21..15.17; fields `{0x20941997, 0x262bfa2e, 0x58ecfcaa, 0xdba9e788}` |
| `061b427f` | ContextualConditionGearIdEquip | born 15.18 with `{0x20941997, 0x262bfa2e, 0xaa8fd0c5, 0xdba9e788}`; 29 instances, all `mSituations{GearEquipped}`; Jinx's rules "Gear0/1/2" drive `Unique2DFormNewOne/Two/Three` |
| `5f945dcb` | ContextualConditionGearIdCheck | same 15.18 birth build as `061b427f`; sole field independently cracked as `GearIds` |

### The 14.15 cohort (tier 3)

`4ab36eb5` and `cdd217c1` were introduced in the same build, both hold a single
String, and both land under one stem. Neither hash is independent of the other -
`EndedVo` came out of `cdd217c1` and was then handed to the run that found
`4ab36eb5` - but the joint reading puts two hashes in one pattern in one cohort.

| hash | name | what fixes it |
|---|---|---|
| `4ab36eb5` | ContextualConditionEndedVoCacSituation | 0.075 expected noise on the run that found it; `Cac` is attested family vocabulary (`ContextualConditionCharacterHasCAC`, field `mCacs`); its 4 instances all sit under `OnVOEndedNearby` holding situation names (`TauntEndedNearby`, `JokeEndedNearby`) |
| `cdd217c1` | ContextualConditionEndedVoEventName | own field `EventName`; same `EndedVo` stem and same introduction build as `4ab36eb5`, parallel String field - **weakest of the sixteen**, since it ships nowhere and nothing outside the hash attests it |

### Single hash plus coherent structure (tier 4, corroborated)

| hash | name | what fixes it |
|---|---|---|
| `0754c268` | ContextualConditionCharacterPrimarySearchTag | carries its own cracked field `SearchTag`; `ContextualConditionCharacter` is the prefix every named `ICharacterSubcondition` uses; its shipped values are exactly the six role tags `tank/marksman/mage/support/fighter/assassin`; and upstream's own `hashes.binfields.txt` holds both `searchTags` and `searchTagsSecondary`, so the primary/secondary split is Riot's, not ours |
| `70f5ed1c` | ContextualConditionObjectiveCountComparison | two `0xe46a1cdb` embeds plus `CompareOp`; Shyvana's rules read "Ally team kill dragon while ahead on dragons" and "... while behind on dragons", differing only in `CompareOp` 2 vs 5 - the semantics are attested, the word `Comparison` is not |

### Fields

`CountOffset` completes a 2x2 against the kill embed, so it is proved by the
lattice rather than by its own hash:

| embed | threshold | offset | source |
|---|---|---|---|
| `0x7463e786` kills | `KillThreshold` | `KillOffset` | `SourceId` |
| `0xe46a1cdb` objectives | `CountThreshold` | **`CountOffset`** | `SourceId` |

| hash | name | what fixes it |
|---|---|---|
| `6b5e21ef` | CountOffset | the lattice row above |
| `cd2c8b8f` | SearchTag | its six shipped values are the champion role tags, lowercased exactly as `searchTags` carries them upstream |
| `affb8300` | GearIds | sole field of `5f945dcb`, `List2 U32` beside `061b427f`'s pair of gear ids |
| `20941997` | NewGearId | survives the 15.17/15.18 rename unchanged |
| `262bfa2e` | PrevGearId | survives the rename unchanged |
| `aa8fd0c5` | CheckNewGearId | the bool the rename swapped in |
| `dba9e788` | CheckPrevGearId | survives the rename unchanged |
| `615964d8` | TimeWindow | declared on both members of the 16.11 pair and nowhere else in the meta, `F32` defaulting to 8.0 |

The template the batch proved: **`ContextualCondition` + the class's own
distinguishing field name**, with the `mSituations` key supplying the name when a
class has no fields.

## Negatives

Ordered as run. `probes x target states / 2^32` is the expected chance-hit
figure; read it before the hits.

| run | probes | expected | outcome |
|---|---|---|---|
| encounter-number proposals | 47 | 0.00003 | 1 landed |
| family-wide structural proposals | 61 | 0.00004 | 7 landed |
| hand proposals, rounds 3-7 | ~640 | 0.0004 | 0 |
| `Transition` batch, class stem and field stem | 54 | 0.00003 | 0 |
| `chain` depth 3, 3 prefixes | 1,332,552 | 0.002 | 0 |
| `chain` depth 4, 3 prefixes | 31,478,937 | 0.051 | 0 |
| `force` depth 2, full wordlist, 3 prefixes | 30,213,306 | 0.049 | 1 landed, 1 refuted |
| binfields `force` depth 2, full wordlist | 10,071,102 | 0.023 | 2 landed |
| `force` depth 3, top 400, 2 prefixes | 128,320,800 | 0.179 | 1 refuted |
| `SearchTag` vocabulary batch | 12 | 0.00002 | 0 |
| `mutate`, full wordlist, unanchored | 1,784,981,846 | 2.494 | 3 refuted |
| `force` depth 2, full wordlist, 24 folded suffixes | 10,071,102 | 0.338 | 2 landed |
| binfields `force` depth 2, full wordlist, prefix `m` | 10,071,102 | 0.030 | 0 |
| binfields `force` depth 3, top 500, 2 prefixes | 250,501,000 | 0.233 | 0 |
| binfields `force` depth 2, full wordlist, 20 folded suffixes | 20,142,204 | 0.347 | 1 refuted |
| cohort-pair proposals | 87 | 0.0001 | 0 |
| `chain` depth 5, prefix `ContextualCondition` | 252,622,038 | 0.353 | 0 |
| `force` depth 4, 146-word CAC domain list | 457,505,454 | 0.639 | 2 refuted |
| `force` depth 2, 4 `EndedVo` prefixes, 8 folded suffixes | 40,284,408 | 0.075 | 1 landed |
| `force` depth 2, 3 `Cac` prefixes, 24 folded suffixes | 30,213,306 | 0.675 | 0 new |
| binfields `force` depth 4, domain list, 4 prefixes | 1,880,588,640 | 0.438 | 1 refuted |

About 4.9e9 probes, 5.6 summed expected chance hits, 9 refuted candidates - the
match is the point. Every chance hit was word salad against the fields of the
class it landed on; all nine are in `hashes/bad/semantic-pass.txt` so no later
run can re-propose them.

What the sweeps establish, and what nobody should re-tread:

- Every **one- and two-word** extension of `ContextualCondition`,
  `ContextualConditionGear` and `ContextualConditionCharacter` over the full
  3,173-word list is exhausted, as is every **three-word** one over the top 400.
- Every **five-word** name built from attested bigrams is exhausted.
- For the three still open, every **four-word** name over a 146-word CAC domain
  list is exhausted, and so is every **three-word** name over the full list ending
  in any of 24 domain suffixes.
- `ContextualConditionEndedVoSituationName`, the obvious reading of `4ab36eb5`
  once `cdd217c1` was in hand, misses. So does every `Cac` variant of `cdd217c1`.
- Unanchored `mutate` is worthless on this family: these names are not one word
  away from any existing name, and the mode spends 1.8e9 probes proving it.

Two passes returned nothing and should not be repeated as attestation:

**A binary string pass is worthless here.** Checked against mac string dumps of
16.17.8057408, both arches: `Contextual` does not occur as a substring anywhere
in either. The CAC system resolves these names by hash only, so no class in
either root has its name in the shipped executable.

**A 117-name identifier probe set** covering `ContextualAction*` and
`ContextualCondition*` symbols: 75 already named at matching casing, 25 casing
variants of already-named, 17 no match, **0 new**. It is missing exactly the ten
names this repo had to crack itself, which makes it a calibration check and
nothing more; its provenance is not one the repo can cite, so per
docs/string-attestation.md it backs nothing here.

## What is left

Three, and **none of them ships a single instance anywhere in the WADs**, so
bin-grep cannot reach them and only a field crack or new vocabulary will.

| hash | root | known | born |
|---|---|---|---|
| `9e9e1f6d` | IContextualCondition | `Count`=1, `CompareOp`, `CharacterType` Option U8, `TimeWindow` F32=8.0, `0xeb60b2e8` Hash | 16.11 |
| `cae2695f` | IContextualCondition | `0x24392a01` List2 Hash, `TimeWindow` F32=8.0, `CharacterType` Option U8, `0xb293a6a9` Bool=true | 16.11 |
| `bd465a5` | IContextualCondition | `0x65fad9fb` Bool | 15.4 |

`9e9e1f6d` and `cae2695f` are a two-class 16.11 cohort: same birth build, same
`TimeWindow` defaulting to 8.0, same optional `CharacterType`, one holding a
single Hash and the other a list of them. Naming either should name the other.
Nothing else introduced in that build is CAC-related, so the cohort argument
reaches no further than the pair itself.

`bd465a5` is the shape every single-Bool sibling in the family shares
(`ContextualConditionSpellIsReady`, `ContextualConditionItemPurchased`,
`ContextualConditionNeutralMinionCampIsAlive`): the bool is named after the class
stem. So `0x65fad9fb` and the class name are one crack, not two - and the field
has survived depth-4 over the domain list under prefixes `""`, `m`, `mIs` and
`Is`.

Open field hashes beside those: `0x4bd0e2b` (bool on `b3420260`), `0x55848081`
and `0x6caa1bcc` (the two kill embeds on `4af7e9f2`), `0xb95ee9e0` (second
objectives embed on `70f5ed1c`), `0x58ecfcaa` (the bool the gear rename dropped).
The two comparison classes name their embed pair differently from each other,
which is why `BaseSource` does not transfer from `70f5ed1c` to `4af7e9f2`.

## Status

All 24 rows `pending`, `pr=-`: 16 in `ledger.bintypes.tsv`, 8 in
`ledger.binfields.tsv`.

Fit to submit as one unit: everything at tier 1, 2 and 3. `cdd217c1` and
`70f5ed1c` are tier 4 and should be split off or called out explicitly in any
upstream PR - `cdd217c1` because nothing outside the hash and its cohort-mate
attests it, `70f5ed1c` because its semantics are attested but the word
`Comparison` is not.

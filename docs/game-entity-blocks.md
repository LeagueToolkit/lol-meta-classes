# The game entity block pass

Reversing doc for `game-entity-blocks`. Its `batches.tsv` note points here.

`GameEntityBlock` went from 28 live unnamed members to 4. 53 names landed: 48
classes and 5 fields.

## What the family is

`IScriptBlock` is the visual scripting statement set. `GameEntityBlock` is the
slice of it that acts on a `GameEntity`, and every member is one editor block:

```
    IScriptBlock              I  the statement            <Verb><Noun>Block
      GameEntityBlock         I  acts on an entity        <Verb>Entity<Noun>Block
      LevelScriptBlock        I  acts on the level        mixed, Block optional
      ILoopScriptBlock        I  loops
      IRunFunctionBlock       I  calls a function
```

A block's fields are its signature, and the pointer types say which way each one
points:

| field type | role |
|---|---|
| `Pointer I<T>Get` | an input, evaluated when the block runs |
| `Embed <T>TableSet` | an output, written to a script variable |
| `Embed <T>ArrayTableSet` | an output that writes several |

So `.Target: Pointer IEntityGet` + `.Position: Pointer IVectorGet` is *set* a
position, and the same `.Target` with `.Dest: Embed VectorTableSet` is *get*
one. That is the whole of what the first pass read.

`GameEntityBlock` had **no named member at all** before this batch, so the
naming pattern came from the 30 named `IScriptBlock` siblings, which fix
`<Verb><Noun>Block` and supply the verbs `Get`, `Set`, `Create`, `Destroy`,
`Insert`, `Remove`, `Copy`, `Shuffle`, `Sort`, `Concatenate`, `Preload`,
`ForEach`. The suffix is not universal: 7 of the 15 named `LevelScriptBlock`
children spell no `Block` at all (`AddLevelTimer`, `CreateNeutralCamp`,
`SpawnAiTurret`), which is why four names below do not carry it either.

## Method

Six passes, in the order they were run.

**1. Suffix folding.** FNV-1a's prime is invertible mod 2^32, so `unfold(h, s)`
is the state a name was in before suffix `s` was appended. 41 targets folded
through 11 suffixes is 451 unfolds against ~13k known-name states, **0.001
expected chance hits**. Three things fell out, and each one opened a lattice:

- `unfold(0x32790cbf, "EntityBlock")` = `Destroy` and
  `unfold(0x2bfff098, "EntityBlock")` = `Show`, both known words. The
  `<Verb>EntityBlock` shape is fixed by the hashes, not proposed.
- `0x5d85166e` and `0x7dcd3672` unfold to **one shared stem** under `Get` and
  `TableGet`, and that stem is `0x44ee0c82`, an unresolved *field* hash the
  family declares. The field names the type: `EntityTag`.
- `0x4dc8b7cf` and `0xacf818f1` share a stem the same way, and it equals
  `fnv("EntityGroup")`.

**2. Structure-led recombination.** Verb crossed with noun in three arrangements
over the shape each member's fields predict: 35,250 probes, 41 target states,
**0.0003 expected chance hits**, 18 hits. Widening to prepositions
(`<Verb><Noun>To/From<Noun>Block`) added three more at 0.004.

**3. Shipped strings.** Every identifier-shaped form in CommunityDragon's
`hashes.bin*` and `hashes.game.txt.*` (4,046,399 distinct forms) against 261
target states: **0.246 expected chance hits, zero actual**. Block class names do
not appear in shipped asset paths, the same clean negative `logic-drivers`
recorded.

The retail clients are where they live. 144,107 forms from the shipped Windows
client at **0.009 expected chance hits** and 190,735 from the macOS binaries at
0.012 returned the same two field names, and both are whole strings:

| hash | shipped string | what it names |
|---|---|---|
| `7190b62d` | `EntityTemplateVar` | the `IDataObjectGet` on `SpawnEntityBlock` and `CreateEntityBlock` |
| `c7a370a7` | `MapIndex` | the `IntTableSet` output on `GetEntityMapIndexBlock` |

**4. Field recombination.** `force` depth 2 over the full wordlist against the
family's 11 unresolved field hashes, 20,040,780 probes at **0.051 expected
chance hits**, returned `ShowDuration` and `HideDuration` and nothing else. They
are the two floats that separate `ActivateEntity*` from `DeactivateEntity*`, so
the fields confirm the verbs that pass 2 had already proposed for their owners.

**5. Lattice completion.** A stem that fills several slots at once is
`(p/2^32)^n`-competitive rather than `p/2^32`. `MapIndex` from pass 3 crossed
with the block templates is 33,696 probes at **0.0003 expected chance hits** and
returned `GetEntityMapIndexBlock` plus the `AddScriptToEntity` /
`RemoveScriptFromEntity` pair. Swapping `Group` for `Tag` across the same
templates, 386,130 probes at 0.004, returned the two pre-rename blocks.

**6. The type closure.** The blocks are wired with `<T>Get` / `<T>TableGet` /
`<T>TableSet`, and those rows are half named upstream already. 101,696 probes of
`<word><tail>` at **0.054 expected chance hits** over the whole unresolved class
set returned 10 names, every one landing on a class whose `I<T>Get` base is
already named for the same `T`. A second sweep of 82,498 probes at 0.043 added
`ExecuteFunctionBlock` and `WhileLoopBlock`.

## The 15.17 rename

The single strongest argument in the batch. In 15.16 the concept was
`EntityGroup`; from 15.17 it is `EntityTag`. Six hashes move together, each
one's lifespan ending or starting exactly at the boundary:

| 14.24-15.16 | hash | 15.17+ | hash |
|---|---|---|---|
| `IEntityGroupGet` | `13d71082` | `IEntityTagGet` | `b60096a7` |
| `EntityGroupGet` | `4dc8b7cf` | `EntityTagGet` | `5d85166e` |
| `EntityGroupTableGet` | `acf818f1` | `EntityTagTableGet` | `7dcd3672` |
| `EntityGroupTableSet` | `2f87c01d` | `EntityTagTableSet` | `72e9216e` |
| `ActivateEntityGroup` | `6408e602` | `ActivateEntityTagBlock` | `17411ce` |
| `DeactivateEntityGroup` | `c03dda3f` | `DeactivateEntityTagBlock` | `b0afb2dd` |

`EntityGroupTableSet` exists in 15.16 and no other build; `EntityTagTableSet`
starts in 15.17 and is still live. The field the blocks declare renames on the
same boundary, `.Group` to `.tag`. Twelve hashes, one rename, and no way for
chance to line up the spans.

## The lattices

Every cell is a name this batch added. The two verb pairs each fill a 2x2, and
the tag column is the same block aimed at a tag instead of one entity:

| verb | fields | `<V>EntityBlock` | `<V>EntityTagBlock` |
|---|---|---|---|
| Show | `.duration` | `ShowEntityBlock` | `ShowEntityTagBlock` |
| Hide | `.duration` | `HideEntityBlock` | `HideEntityTagBlock` |
| Activate | `.ShowDuration` + a bool | `ActivateEntityBlock` | `ActivateEntityTagBlock` |
| Deactivate | `.HideDuration` + a bool | `DeactivateEntityBlock` | `DeactivateEntityTagBlock` |

`ShowDuration` sits on the two Activate blocks and `HideDuration` on the two
Deactivate blocks, which is the row label recovered from the fields alone.

The transform accessors, where the Set half takes the value as an input and the
Get half writes it to a variable:

| concept | Set | Get |
|---|---|---|
| Position | `SetEntityPositionBlock` | `GetEntityPositionBlock` |
| Rotation | `SetEntityRotationBlock` | `GetEntityRotationBlock` |
| Direction | - | `GetEntityDirectionBlock` |

The script value type system the blocks are wired with. A filled cell is a name
this batch added, an *italic* cell was already named:

| T | `I<T>Get` | `<T>Get` | `<T>TableGet` | `<T>TableSet` |
|---|---|---|---|---|
| Entity | *IEntityGet* | - | `EntityTableGet` | *EntityTableSet* |
| EntityTag | `IEntityTagGet` | `EntityTagGet` | `EntityTagTableGet` | `EntityTagTableSet` |
| EntityGroup | `IEntityGroupGet` | `EntityGroupGet` | `EntityGroupTableGet` | `EntityGroupTableSet` |
| DataObject | `IDataObjectGet` | *DataObjectGet* | - | *DataObjectTableSet* |
| Unit | `IUnitGet` | - | `UnitTableGet` | *UnitTableSet* |
| Color | *IColorGet* | *ColorGet* | `ColorTableGet` | *ColorTableSet* |
| Segment | *ISegmentGet* | *SegmentGet* | `SegmentTableGet` | *SegmentTableSet* |
| Path | *IPathGet* | - | `PathTableGet` | - |
| EntityArray | - | - | - | `EntityArrayTableSet` |

`IUnitGet` and `UnitTableGet` are the clearest single check in the batch: they
came out of the same run, and `UnitTableGet`'s two bases are `ScriptTableGet`
and `IUnitGet`, the other hash the run named.

## Attested by shipped data

`bin-grep` over all 455 WADs, `--class GameEntityBlock --subclasses`: 96
objects, in `LolSpellScript` and `BuffScript` on Global, Map11 and Map453. Only
three members ship at all, and all three carry their name:

| class | instances | what attests it |
|---|---|---|
| `GetEntityPositionBlock` | 39 | its `.Dest.Var` is the script variable it writes, and all 31 distinct values are positions: `Pos`, `Pos1`-`Pos10`, `StartPos`, `EndPos`, `TargetPos`, `NextPos`, `PrevPoint`, `MiddlePos`, `OriginalPos`, `AttackPos`, `MisSpawnPos`, `MisEndPos`, `PortalPos`, `FaelightPos`, `ArenaEntry`, `BotRAttackPoint`, `TopLAttackPoint`, `Point1Pos`-`Point4Pos` |
| `ExecuteFunctionEntityBlock` | 55 | every instance sets `.callback.FunctionName` and `.callback.Script`, and the block's own `.FunctionDefinition` carries `FunctionInputs` / `FunctionOutputs` |
| `ActivateEntityBlock` | 2 | on `BuffScript 0x994368ce`, twice, both times as the tail of a nested sequence |

`ExecuteFunctionEntityBlock` gets a second, independent check: pass 6 named
`0x71cab931` `ExecuteFunctionBlock` from the `IRunFunctionBlock` row, so the
`ExecuteFunction` stem is attested on a class this campaign did not aim at.

## Everything else the pass named

| hash | name | what fixes it |
|---|---|---|
| `32790cbf` | `DestroyEntityBlock` | unfolds to `Destroy`; `.Entity` and nothing else |
| `91ccdfbc` | `SpawnEntityBlock` | `.OutEntity`, `.EntityTemplateVar`, `.Position`; arrives 16.8 |
| `d1dec547` | `CreateEntityBlock` | the same block before it, last seen 16.7, one patch earlier |
| `4957da2f` | `SpawnPrefabBlock` | `.Prefab: Pointer IDataObjectGet` plus an `EntityArrayTableSet` out, so it spawns several |
| `a55c9d58` | `PlaceEntityBlock` | `.Entity` in, `.OutEntity` out, 16.9 |
| `0babe88f` | `AddTagToEntityBlock` | `.tag` + `.Entity`, 15.18 |
| `14ef8081` | `RemoveTagFromEntityBlock` | the other half of that pair, same patch |
| `2d67b83f` | `GetTemplateFromEntityBlock` | `.Entity` in, `.template: Embed DataObjectTableSet` out |
| `2994bb4f` | `GetEntityMapIndexBlock` | its own output field is the shipped string `MapIndex`; same 16.5 cohort as the row above |
| `92a4d6f9` | `AddScriptToEntity` | `.ScriptDef: Embed GameEntityScriptDefinition` |
| `a004a785` | `RemoveScriptFromEntity` | `.Script` + `.RemoveAll`, same 15.7 patch as the row above |
| `44ee0c82` | `EntityTag` | the shared stem of `EntityTagGet` and `EntityTagTableGet` |
| `eacbd3e9` | `IAsyncBlock` | `IScriptBlock` child the meta flags as an interface, one field, `.Sequence: Embed ScriptSequence` |
| `9c33070d` | `WhileLoopBlock` | `ILoopScriptBlock` child whose only field is `.Condition: Pointer IScriptCondition` |
| `71cab931` | `ExecuteFunctionBlock` | `IRunFunctionBlock` child with `.Script: Link FunctionModule`, `.Function`, and a call spec carrying `FunctionName` and `StaticFunction` |

`EntityTag` was proposed by `semantic-pass` and not taken (docs/semantic-pass.md
lists it). It is re-derived here from two unfolds rather than from judgement,
which is what that doc asks for.

**The weakest.** `ISequencerBlock` (`046a7786`) and `IRenderingBlock`
(`72f61cad`) sit under `0xece68ca6`, a 12-member family of field-free
interfaces under `IScriptBlock` that declares nothing anywhere. The base and the
interface flag fit `I<Domain>Block`, and both fell out of one 0.054-noise run,
but there is no second slot to check them against and no shipped instance.
Anyone submitting this upstream should split those two off or say so.
`PlaceEntityBlock` is the same shape one tier better: it has a predicted slot,
but nothing beyond it.

**Casing.** `Deactivate` over `DeActivate`: the hash cannot tell them apart, and
upstream ships one of each (`DeactivateEarlySeconds`, `DeActivate`). The one-word
spelling is standard English and keeps `De` out of the wordlist, where it would
be a junk token. `MapIndex` and `EntityTemplateVar` are the strings as shipped,
and our tables already carry `EventMapIndex` and `ShopMapIndex`.

## What is left

4 live members, and 6 unresolved field hashes across them.

    1b7c5512  .Enabled: Pointer IBoolGet, .Target: Pointer IEntityGet
    f8ae8458  the same two fields, a distinct class since 14.24
    80c4552b  the same two plus .Icon: Link GameEntityIconData, so the third of
              the trio toggles an entity's icon
    d6d3da87  two List2 Pointer IEntityTagGet and nothing else, fields 98840e54
              and e2226a17; 15.19

    94018232  Bool on both Activate blocks, beside ShowDuration
    9e8fcd0c  Bool on both Deactivate blocks, beside HideDuration
    7addff1c  the EntityArrayTableSet out on SpawnPrefabBlock
    890d1ad6  Bool on CreateEntityBlock
    98840e54  and e2226a17, the two tag lists on d6d3da87

None of the four has a shipped instance, so there is no graph to read. Ruled out
for them, so nobody re-treads: `identity` and `delete` under 8 prefixes and 6
suffixes, `chain` depth 4, `force` depth 2 over the full wordlist under 6
suffixes, `force` depth 2 under 8 prefixes crossed with 7 suffixes, `force`
depth 3 over the top 700, 42 verbs crossed with the full wordlist and 8
prepositions over 7 tails, and both string corpora against 17 folded suffixes
each. Summed expected noise about 3, and the only hits were the four incoherent
ones the two noisiest runs predicted.

The pair constraint was tried too and is worth recording as a negative: whatever
`94018232` is on the Activate blocks, `9e8fcd0c` is its Deactivate counterpart,
so a name of the form `Show<W>` / `Hide<W>` would have to satisfy both hashes at
once. No `W` in the wordlist does at depth 1 or 2, while the control
`ShowDuration` / `HideDuration` is found immediately. The two bools do not share
a tail under any verb pair the corpus knows.

## Status

All 53 rows `status=pending`, `pr=-`. Nothing submitted upstream.

Fit to submit as it stands: the 15.17 rename block, which is proved by six
lifespans meeting at one patch boundary rather than by any single hash; the four
lattice rows, proved by filling every cell at once; the two fields carrying a
retail client string; and `GetEntityPositionBlock`, which 39 shipped instances
name outright. The rest carry a slot argument, which is the standard
`map-entity-templates` and `viewcontroller-family` met. `ISequencerBlock` and
`IRenderingBlock` are flagged above and are the two names in the batch that
carry neither.

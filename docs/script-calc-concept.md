# Script blocks, concept blocks and spell calculation parts

Record of the batches `concept-blocks`, `game-calculation-parts` and `script-tree-readers`. Their
`batches.jsonl` notes point here.

The batches name 79 classes and 30 fields:

| tree | live | unnamed before | unnamed after |
|---|---:|---:|---:|
| `IScriptBlock` (3c2ab5b0) | 137 | 55 | 11 |
| `IScriptValueGet` (f6e711b0) | 52 | 20 | 5 |
| `IScriptCondition` (c621aa68) | 14 | 6 | 0 |
| `ConceptBlockBase` (91b37a79), inside `IScriptBlock` | 16 | 15 | 0 |
| `IGameCalculationPart` (b60012ce) | 38 | 11 | 0 |
| `IGameCalculation` (17373233) | 5 | 1 | 0 |

## What fixes the names

Three checks, used together:

- **Reader.** Every class's evaluating function was read in the 16.21 Windows client, build `8255794`:
  `Execute` for blocks, `GetValue` for getters, the condition test, `Evaluate` for calculation parts.
  Every name in this record matches what its function does.
- **Registration order.** Registrars and, for script blocks, the `Execute` bodies are emitted in
  alphabetical order of class name. Every class name sorts between the named classes registered before
  and after it (see `driver-family.md` for the measured reliability of this rule).
- **Lattices.** Most names belong to a set that one pattern fills, so several independent hashes
  confirm the pattern together.

### `concept-blocks` (tier 3)

| hashes | names | what fixes it |
|---|---|---|
| 1bbd2264, 47c8ad9b, 7a3483c4 | Add{Int,Float}ConceptBlock, AddConceptBlockBase | one `Add` row of the lattice; `AddValue` (a85a9d2d) is the added operand |
| 4e02c1f7, 6f301bf2, c1def894, cecd632c, fa6a7623, 7b440079 | Read{Vector4,Vector3,Bool,Float,Int}ConceptBlock, ReadConceptBlockBase | `Read` row; each writes the concept value into its `Output` variables |
| 2a721ef1, 34639b56, 612b3741, 927b13d0, a5ad6466, ca17e84f | Set{Vector4,Float,Int,Vector3,Bool}ConceptBlock, SetConceptBlockBase | `Set` row; `NewValueXyz`, `NewValueW`, `NewValueColor` (512a5551, b0e1f0cb, 84ba3181) are the parts of the Vector4 value |

The three rows fill 15 class slots from one 151k-form lattice (chance hits 1e-3 over all targets).
The `Read` row registers between `IsBitSetCondition` and `RoleBoundQuestConfig`, the `Set` row between
`ScriptDataObjectList` and `ShadowOverrideSettings`.

### `game-calculation-parts` (tier 3 unless marked)

| hash | name | what fixes it |
|---|---|---|
| 2b25a73a | AbilityResourceByNamedDataValueCalculationPart | twin of `AbilityResourceByCoefficientCalculationPart`; field `StatFormula` (d13afc27) twins `mStatFormula` |
| 382277da, 8a96ea3c | Min / MaxOfSubPartsCalculationPart | pair; `SubParts` casing follows `SumOfSubPartsCalculationPart` |
| 9e9e2e5c, ba007871 | ExternalNamedDataValueCalculationPart, ExternalCalculationCalculationPart | pair; `SourceObject` links another spell or item |
| 1d452085 | StatByExternalNamedDataValueCalculationPart | the stat form of the external data value; field `DataValueSourceObject` (a519b194) |
| f3cbe7b2 | SpellCalculationCalculationPart | evaluates another calculation of the same spell |
| e945834a | ByItemModifierCalculationPart | returns `Coefficient` when the caster has `ItemModifier` (tier 4) |
| 4ce08984, b22609db, ee18a47b | ByCharLevelNamedDataValue{Breakpoints,s,sInterpolation}CalculationPart | data-value forms of the three `ByCharLevel*` parts; fields `Level1DataValue`, `BonusPerLevelDataValue`, `InitialBonusPerLevelDataValue`, `DataValueBreakpoints`, `ScalePastDefaultMaxLevel`, `ScaleByStatProgressionMultiplier` twin the named fields |
| 0333530c | NamedDataValueBreakpoint | twin of `Breakpoint`; fields `AdditionalBonusAtThisLevelDataValue`, `BonusPerLevelAtAndAfterDataValue` |
| e9a3c91d | GameCalculationWithRangedSplit | subclass of `GameCalculation` with `mRangedMultiplier`, applied when the caster is ranged; one hit in a 1.2M-form lattice (chance 3e-4) (tier 4) |
| a8cb9c14, 353099b0 | UseTargetStats, UseTargetForRequirements | bools that switch the stat or requirement source to the target (tier 4) |

### `script-tree-readers`

| hashes | names | what fixes it | tier |
|---|---|---|---|
| 41fddcf4, 807f462a, adf77958, ae79839e | GetCurrentScriptHashBlock, AttachDynamicScript, DetachDynamicScript, field ScriptHash | error strings in the client's `Execute` bodies | 1 |
| a407be92, 6a66088d, 23b32b94, 40dd375d, 31d87ea6, 914839f7, b30f6438, e51539c7, 71bb3057 | DataObject{Bool,Int,Float,String}Get, DataObject{Bool,Int,Float,String}TableGet, DataObjectValueTableGet | lattice around `DataObjectGet`; each reads a property path of a data object | 3 |
| 1a6cd1f8, 1ef0f01e, 51d069e1 | NilGet, NilUnitGet, NilEntityGet | lattice; each returns no value | 3 |
| d554425c, db992149, 72651449 | UiElementITableGet, ResourceKeyValueTableGet, EntityReferenceGet | twins of `UiElementIGet`, the resource key getter, the entity getters | 4 |
| 662c1ec3 | ILogicDriverSourceGet | base of `IEntityGet` and `IUnitGet`; returns a logic driver source | 4 |
| 467070a9, 5727ed42, cb3604f0, 0b905f14, 5480000a, d7153407 | IsNilCondition, CustomTableHasKeyCondition, EmoteIsCondition, IsEntityTagActive, EntityHasTag, EntityExistsCondition | each test read | 4 |
| 03f4b8c5, 7a0b0e42, 803f9938 | IGameScreen{Audio,Debug,Operation}Block | interfaces beside `IGameScreen{Camera,Entity,Vfx}Block` | 3 |
| 10757364, 1b5a55fa | UiElementSetEnable, UiElementSetColor | pair | 3 |
| 1b7c5512, f8ae8458, 80c4552b | ClearGameEntityIconOverride, SetGameEntityIconEnabled, SetGameEntityIconOverride | trio on the icon component | 3 |
| 15efa930, 1b08a666 | AttachEntityBlock, DetachEntityBlock | pair on `Source` / `Attachment` | 3 |
| ff4e30bb, 1d1499b7, 5fa82956 | GameEntityLoopBlock, ForEachEntityWith{Tag,Script}Block | interface and two loops; fields `FilterByMapIndex`, `MapIndexOverride` (1757fdc8, 7c891337) | 3 |
| 394dede2, 9efea40a, a23138c5, 3178c9c1 | GetRandomFromCustomTableBlock, GetAverageValueInCustomTableBlock, ForEachInSortedCustomTableBlock, RemoveEachFromCustomTableWhereBlock | custom table family; fields `IterKey`, `IterValue` (031923de, 4b091a3c) | 4 |
| 6587b424, fa131962, b1a8a3c0, 15ebaa9c, 1f8480d8 | IsVarInTableBlock, BehaviorScriptCommentBlock, ExecuteFunctionClientBlock, FreeformMathBlock, FindInString | each `Execute` read; `ExecuteFunctionClientBlock` twins `ExecuteFunction{Level,Entity}Block` | 4 |
| 28706210, 07496f1e, a0d8d1a6, ed124985 | GetDragonSoulTypes, GetCampSpawnTypes, GetBuildingLaneBlock, SetWaveBehaviorOverrideBlock | level blocks; fields `SoulTypes`, `SpawnTypes` (02e104d7, 7511a599) | 4 |
| fields | ActivateThenShow, HideThenDeactivate, TagsToActivate, TagsToDeactivate, ValuesToExclude, EventParameterOverrides, OutEntities | each use read; `OutEntities` from the 16.15 client, where its class still existed | 4 |

`CampIdentifier` (5a4ef4e7) was found too; the upstream list serves it already.

## Negatives and held back

- Guesser runs over the remaining script targets (41 classes, 31 fields): `force` depth 2 under 15
  suffixes and `""`/`I` (noise 3.2, all hits but the readers' confirmed ones were noise), `chain` depth 3
  (0.14), fields `force` depth 2 under `""`/`m` (0.16).
- Held back: `IBtLoopScriptBlock` for c07dd66c (breaks its registration bracket), `VfxResourceKeyGet` for
  7908fe0e (the shipped keys are not VFX resources), `ValueForTeam` for d7899f68 (on a stub block),
  `ActivateImmediately` for 890d1ad6 (removed field, no client of its build read).

## What is left

| hash | what is known |
|---|---|
| 025bfa52 | level block, `value` + `BarracksProperty`; stub in the client |
| 3103e774, fc88c9d9, c5e16ef4, c07dd66c | interfaces with no subclass |
| 6b73a615 | runs the calling `IRunFunctionBlock`'s sequence from inside a function body |
| 7c537128 | fires a script event to every listener with parameter overrides |
| 81a1e232 | entity loop within `radius` of `Center`; F32 90f1641f never read |
| 92b6b7fa | table of the ints `MinValue`..`MaxValue` in random order, minus `ValuesToExclude` |
| d6d3da87 | activates and deactivates entity tags (`TagsToActivate`, `TagsToDeactivate`) |
| ed070692 | returns the selected terrain as the index of the lowest set bit |
| 0efe2309, bf65c0ce, ed030d64, 257477d2, 7908fe0e | getters: map placeable position, path points, string hash, Int64 table, literal resource key |
| fields | 1f50a19e, 4283375a, 90f1641f, 9511ac73, 9c60d5ba / a1162603 / bec77248 (tag filters: any / all / none), 9f97e7c, e2d12280, d7899f68, 29bec30a, 96cbc77d, 7633e7cf, 7d27af7f, 9ab1e29d, acc210ca, bafc65c0 |

## Status

All rows `pending`. Tier 1 and 3 rows are fit to submit upstream; tier 4 rows go in a separate, flagged
part of any upstream PR.

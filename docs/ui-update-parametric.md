# UI update elements, UI variables and parametric updaters

Record of the batch `ui-update-parametric`. Its `batches.jsonl` note points here.

The batch names 19 classes and 3 fields:

| tree | live | unnamed before | unnamed after |
|---|---:|---:|---:|
| `IUiVariable` (b041c60a) | 9 | 8 | 0 |
| `IValueUpdateElement` (53de4f74) with `IFloatUpdateElement` (bc413e21) | 11 | 9 | 1 |
| `IBaseParametricUpdater` (68b3a178) | 39 | 5 | 2 |

Every name is a hash match. No string attests any of them; casing follows named siblings (`Ui`, not
`UI`, per the repo rule).

## What fixes each name

Each class's code was read in the 16.21 Windows client, build `8255794`, and each name fits its
registration bracket.

| hashes | names | what fixes it | tier |
|---|---|---|---|
| 02ae7663, 4dd20b88, 7ac03bd3, 24de77f2, a8c7473f, d3d7f73c, 14503484, 3b09052f | UiVariable{Float,Int,Bool,String,TraKey,Image,Object,List} | one `UiVariable<Kind>` row; each `value` type matches its kind; all register between `UISceneViewPane` and `ViewControllerFilterI` | 3 |
| 9e1e8775, 394f5aaf, b5215699, d4ac6ed4 | FloatUpdate{Text,Meter,Radial,Arc}Element | one row; each updates the UI element its field names (`TextElement`, `MeterElement`, `RadialElement`, `ArcElement`) | 3 |
| b011f563, abe2709a, b37ab600 | FloatButtonElementSet{Active,Enabled,Selected} | one row; each writes the button state that its name says | 3 |
| fb66c6d7 | ValueUpdateIconElementMaterialParameters | writes the value into the `ParameterNames` of a material the code names `UpdateIconElementMaterial_%u` | 4 |
| d59163d9 | HasMovedParametricUpdater | child of `IsMovingParametricUpdater`; latches true once the unit moves | 4 |
| e7b61183 | LookAtMissileAngleParametricUpdater | angle from facing to the unit's missile; registers between `LookAtInterestDistanceParametricUpdater` and `LookAtSpellTargetAngleParametricUpdater` | 4 |
| 994ae080 | FacingAndCursorAngleParametricUpdater | angle from facing to the cursor ground point; registers directly before `FacingAndMovementAngleParametricUpdater` | 4 |
| 75d77845, 7a721423, 8d9a165f (fields) | ObjectSchema, ValueUpdateModifiers, SchemaFieldId | the schema hash of object and list variables; the modifier list of the view element drivers; the variable's map key (`SchemaFieldId` weakest: single field, `Id`/`ID` not attested) | 4 |

The class lattices came from one 227k-form run over all 27 targets (1.4e-3 chance hits); the button
row from a meet-in-the-middle over the reader's vocabulary.

## Negatives

- Guesser over the eight classes left after the lattices: `force` depth 2 under 6 prefixes and 6 suffixes
  (noise 0.75), `chain` depth 3 (0.01); fields `force` depth 2 under `""`/`m` (0.02). No hits.
- Targeted lattices for `994ae080`, `267a1194`, `056b6e09` and the two missile fields (109k forms, 1.3e-4).

## What is left

| hash | what is known |
|---|---|
| 267a1194 | bool updater for a map prop: tests the per-arena visibility flag byte of its TFT board (`Operation` 0 any bit, 1 equal); registers between `TftArenaOwnerStreakParametricUpdater` and `TftBoardPosition` |
| 056b6e09 | interface of the missile-following updaters, holds `MissileSpellObjects` |
| ab5db4f2 | interface of the colored float elements (`ColorEasing`, `colors`); base of the Meter, Radial and Arc elements |
| 8f149e18, e1795243 | on `LookAtMissileAngleParametricUpdater`: a yaw offset in degrees, and a bool that measures relative to the skeleton root |

## Status

All rows `pending`. Tier 3 rows are fit to submit upstream; tier 4 rows go in a separate, flagged part of
any upstream PR.

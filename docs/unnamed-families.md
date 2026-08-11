# Unnamed class families in the live meta

A survey of what is still unnamed in the build the game currently ships, and
which parts of it are worth spending a campaign on. Companion to
[docs/plans/vocabulary-expansion.md](plans/vocabulary-expansion.md), which asks
what words a search needs; this asks where to point it.

Scope is patch 16.15: 5,340 classes have existed, 4,533 are in the latest dump,
and **1,824 of those still resolve to nothing**. Grouping them by base class
splits that three ways: 696 sit in a family of 8 or more unnamed members, 719
have no base at all, and 409 are in families too small to be worth aiming a run
at. So the families below cover 38% of the live unnamed surface, and the single
largest remaining block is the baseless 719, which no family argument reaches.

Counts are as of the `map-entity-templates` campaign, which took
`MapPlaceableBase` out of this table - see
[docs/map-entity-templates.md](map-entity-templates.md). Two more have been
worked since, and their rows below are left at the pre-campaign count so the
census stays one snapshot: `ILogicDriver` is down from 66 unnamed to 23
([docs/logic-drivers.md](logic-drivers.md)), and `IScriptBlock` from 85 to 56
via its `GameEntityBlock` sub-root
([docs/game-entity-blocks.md](game-entity-blocks.md)).

## Reading the census

Rows come from `guesser_families.py rank`; the write-ups below are read off
`emit --subtree --live` packs.

| column | what it measures |
|---|---|
| `unnamed` | live unnamed classes in the subtree, not just direct children |
| `named` | live named ones beside them, which is what fixes the naming pattern |
| `handle` | members carrying at least one **resolved** field |
| `res%` | resolved share of the family's own field hashes |
| `defs` | fields whose default is not the type's zero |
| `+clo` | further unnamed classes reached by following the family's field types |

`unnamed` alone ranks the families; the rest decide whether one is workable,
and the two are close to independent (74 unnamed with 1 handle is nothing to
aim at; 28 unnamed with 27 handles is self-describing).

## Census

Overlapping roots collapsed to the outermost family; sub-roots worth naming
separately are listed in the write-ups below.

| unnamed | named | handle | res% | defs | +clo | span | family |
|---|---|---|---|---|---|---|---|
| 85 | 44 | 65 | 78 | 49 | 11 | 13.20..16.15 | `IScriptBlock` (3c2ab5b0) |
| 74 | 20 | 1 | 50 | 0 | 0 | 16.7..16.11 | `BaseParams` (c03c9e4e) |
| 66 | 68 | 40 | 67 | 9 | 3 | 13.15..16.15 | `ILogicDriver` (995ca734) |
| 65 | 0 | 55 | 63 | 42 | 31 | 13.15..16.13 | `ISequenceAction` (eb31be9b) |
| 52 | 189 | 43 | 46 | 98 | 237 | 13.15..16.14 | `ViewController` (ed511190) |
| 47 | 19 | 24 | 21 | 125 | 49 | 13.18..16.15 | `IGameModeConfigBase` (3e5051fa) |
| 37 | 9 | 4 | 18 | 7 | 1 | 15.4..16.11 | `IKeyBind` (5ba190cc) |
| 34 | 0 | 0 | 0 | 0 | 0 | 13.15..16.13 | `ISequenceActionInstance` (fb677f88) |
| 28 | 23 | 15 | 100 | 1 | 2 | 14.9..16.6 | `IScriptValueGet` (f6e711b0) |
| 27 | 0 | 1 | 14 | 1 | 0 | 15.20..16.11 | `0x2a9f4223` |
| 22 | 56 | 22 | 100 | 8 | 2 | 15.22..16.8 | `IVfxBaseDriver` (cbd100e7) |
| 15 | 0 | 5 | 12 | 15 | 2 | 16.7..16.14 | `0x4ca99280` |
| 15 | 0 | 10 | 100 | 0 | 3 | 14.14..16.1 | `0x8930818a` |
| 14 | 28 | 6 | 75 | 1 | 0 | 15.14..16.9 | `Cheat` (946adb4c) |
| 11 | 26 | 8 | 56 | 1 | 8 | 13.15..16.9 | `IGameCalculationPart` (b60012ce) |

Prior campaigns, from `hashes/overrides/ledger.bintypes.tsv`, so nobody
re-treads: `IVfxBaseDriver` is 56/56 ours (`vfx-driver-graph`), `BaseParams`
20/20 and `ViewController` 31/189 (`season16-modes-ui`, `viewcontroller-family`),
`IGameModeConfigBase` 5/19 (`gamemode-configs`). `MapPlaceableBase` is finished
and off the table: `map-entity-templates` took it from 28 unnamed to 5,
`logic-drivers` took `ILogicDriver` from 66 to 23, and `game-entity-blocks` took
`GameEntityBlock` from 28 to 4, which is most of `IScriptBlock`'s drop from 85
to 56. **`ISequenceAction` has never been touched by a campaign here** - every
name beside it came from upstream.

## The nine worth following up

Ordered by expected yield, not by size. `MapPlaceableBase` was number 5 on this
list and has been worked - see
[docs/map-entity-templates.md](map-entity-templates.md). The method for all of
these is the `crack-family` skill (`.claude/skills/crack-family/SKILL.md`).

### 1. `IScriptBlock` (3c2ab5b0) - 85 unnamed, 44 named

The visual scripting block set. The single best target in the meta: biggest
family, 65 of 85 members carry a resolved field, and the field *types* encode
the block's signature directly.

- `Pointer I<T>Get` is an input, `Embed <T>TableSet` is an output. A block with
  `.Target: Pointer IEntityGet` and `.Position: Pointer IVectorGet` sets a
  position; the same pair with `.Dest: Embed VectorTableSet` reads one.
- 44 named siblings fix the pattern to `<Verb><Noun>Block`, with a large
  attested verb vocabulary already: `Get`, `Set`, `Create`, `Destroy`, `Insert`,
  `Remove`, `Copy`, `Shuffle`, `Sort`, `Concatenate`, `Preload`, `ForEach`.
- Sub-roots worth their own `--only` run: `IUiBlock`, `LevelScriptBlock`,
  `ILoopScriptBlock`, `IBehaviorScriptBlock`, `IRunFunctionBlock`. The sixth,
  `GameEntityBlock`, has been worked - 28 unnamed to 4, and the family is down
  to 56 because of it.
- Its own base `0x38a7f9b3` is unnamed and has exactly two children,
  `IScriptBlock` and `SwitchCase`.

The `GameEntityBlock` record is
[docs/game-entity-blocks.md](game-entity-blocks.md); its read - a block's
inputs and outputs name it - carries to the remaining sub-roots.

### 2. `ISequenceAction` (eb31be9b) - 65 unnamed, 0 named

The cinematic sequencer's action set, and the largest family with **no named
member at all** to pattern-match against. That sounds worse than it is: 55 of
the 65 carry resolved fields, and the defaults are unusually loud.

- `0x1cf9835` and `0x70aa7cbc` embed `{Magnitude: 20.0, ShakesPerSecond: 6.0,
  FalloffRate: 2.0}` beside `{FalloffRadius, FalloffEasingType}`. That is a
  camera shake, spelled out in numbers.
- `0x755cf26f.SoundEvent`, `0xa2913bfb.ParticleSystem: Link
  VfxSystemDefinitionData`, `0xd13da199.{StartColor, EndColor, AlphaOnly,
  EasingType}`, `0x8a46e486.{AnimationName, PauseOnEnd}`: each names its own
  action.
- Interior interfaces group the tweens: `0xf71f4123` (`StartFloat`/`EndFloat`),
  `0x8e134d2`, `0x99320e3c` (`EventName`, `ForceStopWhenDone`), `0xa2913bfb`
  (particles). Their children differ only by the type they interpolate.
- 31 further unnamed classes hang off the field types, including the heavily
  shared `0x94e82d1f` (a location source, referenced by 31 fields) and
  `0x8b54c9a7` (an object source). Both were proposed and not taken in the
  semantic pass as `SeqInputVector` and `SeqInputObject`.

### 3. `ILogicDriver` (995ca734) - 66 unnamed, 68 named

**Worked: `logic-drivers` landed 46 names and left 23. See
[docs/logic-drivers.md](logic-drivers.md); the arithmetic quartet below is
cracked, and so is the Concept cluster.** The rest of this section is the
pre-campaign reading, kept because it is what the campaign was aimed with.

Material and gameplay logic drivers. Untouched by any campaign here, and the 68
named siblings are all upstream, so the convention is externally attested:
`<Predicate><Type>Driver` with `Bool`, `Float`, `Int`, `Vector3` and `Material`
variants (`IsAttackingBoolDriver`, `FloorFloatMaterialDriver`, `MinMaterialDriver`).

The arithmetic quartet is the cleanest single lead in the whole survey:

```
    0xf9e5b8b9  INTERFACE  .value: Pointer ILogicFloatDriver
      0x5b2fdd66           .add
      0x53dfc5b5           .Subtract
      0xef339ef9           .multiplier
      0xcc35f742           .Denominator
```

Four children of one interface, each adding exactly one operand field that
names the operation. `Add`/`Subtract`/`Multiply`/`Divide` crossed with the
family's two suffixes did not hit, so the stem differs from the operand word;
that is a bounded search over one interface's four children, not a fishing trip.

It was the interface's own word that was missing: the base is `MathFloatDriver`
and the children are `Math<Op>FloatDriver`. The word came out of the retail
client binary, not out of recombination.

A second cluster: 10 classes introduced together in 15.11, each declaring a
single `.Concept: Link <T>Concept` field and nothing else. They factor exactly
5 concept types by 2, and the pair is the *output* type, which the base already
names:

```
    FloatConcept    0x89e28df1, 0xcfb52801   both ILogicFloatDriver
    Vector3Concept  0x5b6af6b7, 0x9ddc7667   both ILogicVector3Driver
    Vector4Concept  0x246724da, 0x764bdc8a   both ILogicDriver
    IntConcept      0x7ffa190e ILogicIntDriver   0x3acde15e ILogicFloatDriver
    BoolConcept     0xd686c7f9 ILogicBoolDriver  0x5169d349 ILogicFloatDriver
```

Input type in the link, output type in the base, and the two cross-typed rows
say the pattern is "read concept X as Y" rather than one class per concept.

They are `Raw<T>ConceptLogicDriver` and `Eased<T>ConceptLogicDriver`. The two
cross-typed rows are the tell: easing a bool or an int yields a float, which is
why exactly those two `Eased` classes derive from `ILogicFloatDriver`.

### 4. `ViewController` (ed511190) - 52 unnamed, 189 named

Richest per-class context anywhere: 4.7 resolved fields per unnamed member, 98
non-zero defaults, and 237 further unnamed classes reachable through the field
types, which is more unnamed surface in the closure than any other family holds
outright.

The fields are in-engine Ui element handles and read like a screenshot:
`.EmoteWheelNeButton_Icon`, `.RewardGrid`, `.CountdownText`, `.SearchbarText`,
`.StarShardCurrencyButton`. 189 named siblings fix `<Feature>ViewController`.

Partly worked already (31 of the 189 are ours, across `season16-modes-ui` and
`viewcontroller-family`), and `viewcontroller-family` records the in-family
refutation that makes this one cheap to check: a real hit has to land on a class
whose base actually is `ViewController`. The 237-class closure is the
unexplored half, not the 52.

### 5. `IGameModeConfigBase` (3e5051fa) - 47 unnamed, 19 named

The `gamemode-configs` campaign named 7 and stopped; 47 remain, with 125
non-zero defaults and 49 more unnamed classes in the closure - the most default
evidence per class of any family. [docs/gamemode-configs.md](gamemode-configs.md)
already records the method (per-target search over each class's recursive
structure vocabulary, 0.001-0.007 expected chance hits per target) and its own
dead ends, so this is resumable rather than new work.

### 6. `IScriptValueGet` (f6e711b0) - 28 unnamed, 23 named

Small but the highest-confidence family in the survey: **100% of its members'
field hashes are resolved**, and the base tuple is a near-unique fingerprint,
because these classes use multiple inheritance to declare what they return.

```
    0x40dd375d  bases IStringGet, IScriptValueGet   .PathHash  .PropPath
    0xa407be92  bases IBoolGet,   IScriptValueGet   .PathHash  .PropPath
    0x6a66088d  bases IFloatGet, IIntGet, IScriptValueGet
```

Three classes differing only in return type, over an identical `PropPath` +
`PathHash` pair. The 23 named siblings (`BoolGet`, `IntGet`, `StringGet`,
`VectorTableGet`, `FloatOffsetTableGet`) fix `<Type>[Table]Get` exactly, so the
name is a function of the base tuple plus the field pair.

### 7. `BaseParams` (c03c9e4e) - 74 unnamed, 20 named

The second largest family and the one that suits a **guesser sweep rather than a
semantic pass**, which is why it is listed this low despite the size. 72 of the
74 have no fields at all, so there is nothing to derive a proposal from, but the
constraints are otherwise ideal:

- fixed affix, attested 20 times over: `ParamsDamage`, `ParamsChampionKill`,
  `ParamsGoldEarned`, `ParamsHeal`, `ParamsAssist`, `ParamsMinionKill`.
- the family is closed and recent (16.7, four more by 16.11), and the subject is
  a closed set: gameplay telemetry events.
- 74 targets instead of 2,400 is a 32x noise discount on every probe.

`--prefix Params --only` over an event wordlist is the shape. `Params<Event>`
and `<Event>Params` both occur among the named 20, so run both.

### 8. `IKeyBind` (5ba190cc) - 37 unnamed, 9 named

Structurally the thinnest large family, at 18% resolved and 4 members with any
handle at all, so it is listed for a different reason: **the target list is
enumerable from outside the meta.** Every one of the 37 is a keybind, 35 of them
`InputEventBoolKeybind` children, and the retail client enumerates its bindable
actions in the settings Ui and in `PersistedSettings.json`.

This is the `string-attestation` method, not the guesser: probe shipped strings
against these 37 hashes. Noise is arithmetic zero at that target count, and a
hit is a name Riot wrote. The named siblings (`LolPingKeybind`,
`LolEmoteKeybind`, `LolSpellSlotLevelUpKeybind`) and the few resolved fields
(`.PingCategory`, `.EmoteDirection`, `.HoldType`, `.ToggleType`) show the
vocabulary the strings should be filtered for.

### 9. `0x4ca99280` - 15 unnamed, 0 named

Small, but wholly unexplored, entirely 16.7 or newer, and the only family in the
survey whose **root and every member are unnamed**. 15 non-zero defaults across
28 unresolved field hashes.

The shape is a priority-ranked selector: a repeated embed
`{disabled: False, priority: 0.5, ...}` appears on most members, members carry
their own `.priority` at 0.9, 0.5, 0.1, and `0x4036941a.PrimaryEvent` points at
an 11-field String-heavy class. `0x36255113` and `0x9583cf01` share five field
hashes. The semantic pass proposed `AudioContextEventType` for the root and did
not take it; nothing has revisited it since.

Being new and unnamed at the root means no vocabulary in the corpus reaches it,
so this one needs the shipped-data pass first and the search second.

## Big families that are not what they look like

### `ISequenceActionInstance` (fb677f88) - 34 unnamed, and free

Zero fields on all 34 members, zero named siblings: by the census it is the most
hopeless family in the meta. It is actually **derivative of `ISequenceAction`**,
and lands for nothing once that campaign runs.

The two families are paired 1:1 by introduction build. Of the 19 builds that
introduce an instance class, 17 introduce exactly as many actions in the same
build, and no build ever introduces more instances than actions:

| patch | actions | instances |
|---|---|---|
| 14.16 | 5 | 5 |
| 16.4 | 8 | 8 |
| 15.13 | 3 | 3 |
| 13.15, 13.24, 14.15, 14.18, 14.21, 14.3, 15.18, 15.22, 15.3, 15.6, 16.10, 16.13 | 1 each | 1 each |
| 14.1, 16.6 | 2 | 2 |
| 15.10 | 2 | 1 |
| 15.12 | 3 | 1 |

50 actions to 34 instances overall, with 3 of the actions being interfaces. So
the instance class is the runtime half of an action, emitted alongside it, and
`ISequenceActionInstance` itself fixes the suffix. Do not run a search at this
family; name the actions and pair by build.

### `0x2a9f4223` - 27 unnamed, and genuinely void

The keybind-set container tree, sitting under `LolKeybindSet`. Root, both
intermediate interfaces (`0x7f631ac9`, `0xb5754dad`, `0xe057ab66`) and all 27
members are unnamed, and the entire subtree declares **two** distinct field
hashes: a `List2 U32` and a self-referential `List2 Pointer`. No named sibling,
no default, nothing in the closure. Same for `0xece68ca6` (11 members, all
interfaces, no fields anywhere).

Nothing structural can be derived here and recombination has no anchor. These
need an attested string or a registration-order argument, and until one exists
they are correctly left alone.

## Status

Three of the ten have been worked. `map-entity-templates` landed 109 names out
of `MapPlaceableBase` ([docs/map-entity-templates.md](map-entity-templates.md));
`logic-drivers` landed 46 out of `ILogicDriver`
([docs/logic-drivers.md](logic-drivers.md)); `game-entity-blocks` landed 53 out
of `IScriptBlock`'s `GameEntityBlock` sub-root
([docs/game-entity-blocks.md](game-entity-blocks.md)). Nothing else here has
entered a table.

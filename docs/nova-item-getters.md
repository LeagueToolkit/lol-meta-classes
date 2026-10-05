# nova-item-getters - campaign record

Record for `nova-item-getters`. Method: the `crack-family` skill
(`.claude/skills/crack-family/SKILL.md`). 56 names are landed and `pending`. A second
batch, `nova-item-selection-filters` (section 6), proves the filter tree and one image
getter, which leaves 13 hashes open. The negatives below say where not to spend.

Patch 16.17 (build `8104348`) introduced 85 classes. 62 of them form one system across
nine roots, and **all 62 were live unnamed**. The first pass proved 32 and the second
17, which leaves 13.

| root | what it is | members | proved | left |
| --- | --- | ---: | ---: | ---: |
| `0x822cf77c` | Bool getter tree | 7 | 7 | 0 |
| `0xa79ac316` | Float getter tree | 6 | 6 | 0 |
| `0x53bee89d` | Int getter tree | 9 | 8 | 1 |
| `0x8d941811` | Image getter tree | 7 | 6 | 1 |
| `0x3f8dac45` | String getter tree | 6 | 6 | 0 |
| `0x70f6f74b` | filter / predicate tree | 16 | 16 | 0 |
| `0x40452a8d` | variable-binding tree | 6 | 0 | 6 |
| `0x19ccc111` | input reference tree | 3 | 0 | 3 |
| `0x664d2c6d` | empty interface pair | 2 | 0 | 2 |

The five getter trees are **parallel expressions of one concept set**, in the sense of
the skill's section 2: the same node roles repeat once per value type. That is what
made them cheap, and it is why the filter and binding trees are still open. They do
not repeat the type word the same way.

---

## 1. What fixes the names

The whole batch rests on one proved stem. `recover_stem` was run **once per node
role** rather than once over the family, so each run is a 4-way or 5-way agreement
worth 96 to 128 bits. Five roles independently returned the same prefix state
`0xd6fbee84`:

| role | suffix folded out | members agreeing |
| --- | --- | ---: |
| interface | `''` | 5 |
| literal | `value` | 4 |
| named property | `property` | 4 |
| fallback chain | `switch` | 5 |
| conditional | `if` | 5 |

Cross-role agreement on one state is the evidence. A single role's hit would not be.
The state resolves to the prefix `novaitemget`, and `Nova` is externally attested:
`NovaScoreboardViewController` (`0xb67bce14`, 16.9) is an upstream name in
`hashes/hashes.bintypes.txt`, not one of ours.

Two independent corroborations, neither used to derive anything:

**The base check passes for all 32.** `guesser_families.py check` reports the base
class per hit. Every proposal landed on the class whose base actually is its family
root. This is the skill's own refutation test and it did not refute one member.

**The four non-existent nodes are absent.** `NovaItemGetImageValue`,
`NovaItemGetImageProperty`, `NovaItemGetImageConcept` and `NovaItemGetStringConcept`
all miss. They are exactly the four cells the lattice does not contain. The absent
string concept matches the shipped Concept system, which has `BoolConcept`,
`IntConcept` and `FloatConcept` and no string one.

**Predecessor.** `MonarchPropertyGet` (`0x3e149034`, 14.23) is the same construction
one generation earlier: one subclass per value type, the same `value` property hash
`0x425ed3ca`, the same `<Codename><Noun>Get<Type>` template. Nova's version adds the
`Property` / `Concept` / `Switch` / `If` roles and becomes a tree.

### 1.1 Evidence table - classes

**Tier 3** (lattice, several hashes corroborating one pattern) for the 26 grid
members. The grid is 5 types by 5 roles with 4 cells absent, and every filled cell
landed in its predicted inheritance slot.

| hash | name | what fixes it |
| --- | --- | --- |
| `0x822cf77c` | `NovaItemGetBool` | interface row of the proved lattice |
| `0x74e03875` | `NovaItemGetBoolValue` | lattice + `value` = `0x425ed3ca` |
| `0x49b83ded` | `NovaItemGetBoolProperty` | lattice + `PropertyName` field |
| `0x902a32e0` | `NovaItemGetBoolConcept` | lattice + `Concept` field |
| `0x786e4e48` | `NovaItemGetBoolSwitch` | lattice + `DataOptions` field |
| `0x385be74b` | `NovaItemGetBoolIf` | lattice + `Condition` embed |
| `0xa79ac316` | `NovaItemGetFloat` | interface row |
| `0x7dcbc017` | `NovaItemGetFloatValue` | lattice |
| `0xb3cd3cc3` | `NovaItemGetFloatProperty` | lattice |
| `0x2b65f34e` | `NovaItemGetFloatConcept` | lattice |
| `0x8e0b143e` | `NovaItemGetFloatSwitch` | lattice |
| `0x7ea21ad1` | `NovaItemGetFloatIf` | lattice |
| `0x53bee89d` | `NovaItemGetInt` | interface row |
| `0x6fb58e22` | `NovaItemGetIntValue` | lattice |
| `0x3d196b00` | `NovaItemGetIntProperty` | lattice |
| `0x60e9ac3b` | `NovaItemGetIntConcept` | lattice |
| `0x2c9d2ca9` | `NovaItemGetIntSwitch` | lattice |
| `0xbc3a680e` | `NovaItemGetIntIf` | lattice |
| `0x8d941811` | `NovaItemGetImage` | interface row |
| `0x55e4081d` | `NovaItemGetImageSwitch` | lattice |
| `0x4c56ab8a` | `NovaItemGetImageIf` | lattice |
| `0x3f8dac45` | `NovaItemGetString` | interface row |
| `0x23025f4a` | `NovaItemGetStringValue` | lattice |
| `0x56276148` | `NovaItemGetStringProperty` | lattice |
| `0xb554f9f1` | `NovaItemGetStringSwitch` | lattice |
| `0x53806086` | `NovaItemGetStringIf` | lattice |

**Tier 4** (proved stem, but the role word rests on a single hash each) for the six
leaf getters. **Flag these in any PR, or split them off.** The stem `NovaItemGet` is
tier 3, the last word is not.

| hash | name | what fixes it | strength |
| --- | --- | --- | --- |
| `0x5d820e16` | `NovaItemGetItemId` | proved stem, and the class has **no properties**, so it returns a fixed attribute | good: the search never saw the property count |
| `0x62c1f14e` | `NovaItemGetDisplaySlot` | same, the second property-less Int node | good, same argument |
| `0x89ea80a5` | `NovaItemGetIcon` | same, the property-less Image node | good, same argument |
| `0x5a7f871f` | `NovaItemGetTooltip` | proved stem, class holds `TooltipKey: Hash` | good: field name matches |
| `0x5711e197` | `NovaItemGetImageLookup` | proved stem, class holds `Key: Hash` | **weakest of the six**, `Lookup` is not attested by any field |
| `0xd5e5500b` | `NovaItemGetBoolCompare` | proved stem, class holds only a `Condition` embed | weak, `Compare` is not attested by any field |

### 1.2 Evidence table - fields

24 field hashes, all previously unnamed in this repo and in the CommunityDragon
corpora. Tier 3: the typed sets corroborate each other, and the `<Type>Variable` /
`<Type>Property` pairs form a closed 5-row lattice on the binding tree.

| hash | name | what fixes it |
| --- | --- | --- |
| `0xa4c6cd6b` | `PropertyName` | one hash across all four `…Property` nodes |
| `0xf3e7ac05` | `DataOptions` | one hash across all four `…Switch` nodes |
| `0x15d49cde` | `Operand` | one hash across the four literal-comparison filters |
| `0xb7218bcb` | `IntProperty` | binding lattice, paired with `IntVariable` |
| `0x8a59b96a` | `IntVariable` | binding lattice |
| `0xc3f30ba0` | `BoolProperty` | binding lattice, paired with `BoolVariable` |
| `0x622ef3dd` | `BoolVariable` | binding lattice |
| `0x3f03cd2f` | `ImageProperty` | binding lattice, paired with `ImageVariable` |
| `0x09055cde` | `ImageVariable` | binding lattice |
| `0x56b336c5` | `FloatVariable` | binding lattice, pairs the named `FloatProperty` |
| `0xaf8d054a` | `TextVariable` | binding lattice, pairs the named `TextProperty` |
| `0x9837f87b` | `VariableValues` | type is `Map Hash Pointer IUiVariable`, which names it |
| `0x9a1a1c7e` | `CategoryValue` | category group, with `CategoryName` on the same class |
| `0xb5bd9b2a` | `CategoryName` | category group |
| `0x78115fc4` | `CategoryId` | category group, beside `DisplayNameTra` |
| `0x5eb0c06d` | `Categories` | type is `List2 Link 0x5c00206c`, the category entry |
| `0xb209de14` | `HeaderButton` | beside `Group` and `TitleText` |
| `0x496a0260` | `CompletionText` | beside `StatusText` and `IsExpanded` |
| `0xc4c472c8` | `IsExpanded` | same class |
| `0x18932962` | `KillerType` | on an `IContextualCondition` subclass |
| `0xb74298af` | `Mult` | beside `add`, and the default is exactly `1/48` to `add`'s `48` |
| `0x74752bb3` | `NormalStrength` | beside `TextureSize` on the water-sim config - `0xdea3b4a8` at 16.17, `0x595773f6` before it |
| `0x8ce41f71` | `WaterEnabled` | paired with `SandEnabled` on the same class |
| `0xd75074d8` | `SandEnabled` | paired with `WaterEnabled` |

**`0x01339cb3` = `OverlayDisabled` was re-derived by the same run and is already
named upstream.** It was left in the target set by accident and came back correct.
Treat that as the pass's blind control, not as a new name.

The `IntVariable` / `IntProperty` pair is what named the Image tree. Binding class
`0xd46044c0` holds `ImageVariable: Hash` beside `ImageProperty: Pointer 0x8d941811`,
which fixes `0x8d941811` as the image tree with no guessing.

---

## 2. Negatives - do not re-run these

Summed expected noise across the whole campaign is about **1,345**, and essentially
all of it is the one character sweep. Read that row as a warning, not a result.

| run | probes | targets | expected noise | outcome |
| --- | ---: | ---: | ---: | --- |
| 2-token MITM, bintypes+binfields vocab (3,022 words) | 9.14e6 | 55 | 0.12 | **0 hits** |
| 2-token MITM, full corpus vocab (19,445 words) | 3.78e8 | 55 | 4.84 | **0 hits** |
| 2-token MITM on the prefix state `0xd6fbee84` | 3.78e8 | 1 | 0.09 | **0 hits** |
| character MITM, every a-z string up to 9 chars | 5.65e12 | 1 | 1315 | 1,397 candidates, all junk |
| `novaitem*` prefixes crossed with 1-2 tokens | 2.65e9 | 39 | 24.0 | noise only |
| `recover_stem` on the comparison sets, type word last | - | 4 each | - | no consistent stem |
| `recover_stem` on the binding tree, words `bool/float/int/image/text` | - | 5 | - | no consistent stem |
| exe running-hash scan, every string in the 16.17 client | - | 1 | - | no string begins with the prefix |
| `exe_typenames.py --match`, 2,033 type descriptors | - | all | - | named 2 classes, neither in this family |

Four readings worth keeping.

**Recombination cannot reach this family.** The full-corpus two-token sweep named 0 of
55 at 4.84 expected noise. The missing token is in no name the game has shipped. That
is the skill's stopping condition, and it is why the stem route was needed.

**The character sweep is a measured dead end.** It returned 1,397 candidates against
1,315 expected, which is pure chance to two figures. A single 32-bit state carries no
cross-check, so blind enumeration cannot work on it at any depth. The answer was 11
characters and the sweep reached 9.

**The client binary is a clean zero and proves nothing.** This matches
`contextual-conditions`: the game resolves meta names by hash and never materializes
the string. Do not read the miss as evidence against any name here.

**What actually found the prefix** was a curated 136-word seed list at three tokens,
3.6e8 probes against one state at 0.08 expected noise, returning `novaitemget` alone.
Human or model vocabulary, not a bigger machine.

---

## 3. What is left - 13 open hashes

### 3.1 The filter tree, `0x70f6f74b` (16)

Solved. See section 6.

### 3.2 The variable-binding tree, `0x40452a8d` (6)

```
0x40452a8d   INTERFACE, no properties
  0x49c42124   IntVariable   + IntProperty   -> NovaItemGetInt
  0xc8714ff3   BoolVariable  + BoolProperty  -> NovaItemGetBool
  0xdbab6aeb   FloatVariable + FloatProperty -> NovaItemGetFloat
  0xd46044c0   ImageVariable + ImageProperty -> NovaItemGetImage
  0x8aa21ee4   TextVariable  + TextProperty  -> NovaItemGetString, plus 0x78812955: Bool
```

Typed-parallel, so `recover_stem` applies directly. It failed with
`bool/float/int/image/text`. Retry with `string` for `text`, and with the role word as
`Variable`, `Var`, `Set`, `Bind` or `Write`. `NovaItemSet<Type>` is the obvious
candidate and it does **not** match. Note the extra `Bool` on the text node only.

### 3.3 The input reference tree, `0x19ccc111` (3)

```
0x19ccc111   INTERFACE, no properties
  0x2142e1f7   SpellSlot: Embed SpellBookIndex
  0xafd2f805   InputEvent: Hash, 0xb915a5ee: String
```

`SpellBookIndex` is the spell-slot keybind struct Monarch introduced at 14.14 and
mainline adopted at 15.3. Try `Keybind` as the last word, which both
`MonarchSpellSlotKeybind` and `LoLSpellSlotKeybind` use.

### 3.4 Two getter nodes

`0x556b035c` is `NovaItemGetImageBySlot` (section 6).

```
0xc752c9d7  base NovaItemGetImage   Property: Pointer NovaItemGetString
0x9b8a2421  base NovaItemGetInt     0x127a3f97: Pointer NovaItemGetString
```

Each takes a computed argument. Likely last words: `Slot`, `Path`, `Parse`, `Length`,
`Count`. `Lookup` is already spent on `0x5711e197`.

### 3.5 The empty pair, `0x664d2c6d` (2)

Two nested interfaces, no members, and nothing in the 16.17 set references either.
Nothing constrains them. Leave them.

### 3.6 Stem states with slots waiting

| state | what it is | slots waiting |
| --- | --- | --- |
| `0xd6fbee84` | `novaitemget`, proved | none, spent |
| `0x478f319d` | `novaitemselectionfilter_`, proved | none, spent |
| - | the binding-tree stem, unknown | 6 |
| - | the input-tree stem, unknown | 3 |

### 3.7 Open field hashes

| hash | on | type | note |
| --- | --- | --- | --- |
| `0x5e16be82` | the five `…If` nodes | `Pointer <self>` | **five classes at once, so a 5-way `crack_pair` target.** Best single field lead |
| `0x127a3f97` | `0x9b8a2421` | `Pointer NovaItemGetString` | int from a string |
| `0x78812955` | `0x8aa21ee4` | `Bool` | only on the text binding |

---

## 4. Context routes that are not cracking

Every pass above searches the same closed vocabulary. These bring in new evidence
instead, which is what the skill means by "the unlock is new attested vocabulary".
Ordered by expected value for **these 30 hashes**.

### 4.1 Registration order from the client binary (highest value, needs one input)

`League of Legends.exe` emits its `MetaClass_register` calls in roughly alphabetical
order by class name. Rebuilding that order gives each unnamed class a **lexical
bracket** between its named neighbours, which constrains a name without any
vocabulary at all. It is also **case-sensitive**, which makes it the casing oracle
this batch otherwise lacks.

The tool is `league_structs/tools/exe_regorder.py`. It needs a 16.17 exe and the
right `MetaClass_register` RVA, which must be re-found per build. Known values:
`0x1411A65F0` at 16.13, `0x1411B15C0` at 16.15, `0x119C0D0` at 16.17.8057408. Our
build is **8104348**, so the RVA is not yet known and must be re-derived.

Why it pays here specifically: the filter tree has 16 members. A bracket constrains
each one independently, and brackets are about 90% stable across builds. Note the
tool had a 1.3% dropout fixed on 2026-08-26. Any index recorded before that is stale.

### 4.2 The consumer side - what evaluates the trees

Nothing in this campaign knows what *runs* these getters. Finding the evaluator names
the roles directly, because the vtable slot order and the switch on node type both
mirror the tree. This is ordinary reversing rather than hashing, and it is the only
route that can distinguish, say, "and" from "or" between `0x756c1ccc` and
`0xc7ad307e`. Those two are indistinguishable by structure: identical signatures,
identical types. **No amount of hashing separates them.** A reader must.

### 4.3 The debug build's enum tables

`Operator: U8` on eight filter classes is an enum. The 2024 all-logs debug build
keeps 434 `EnumRegistrar` tables plus a class-to-field name block. If the operator
enum is among them, its members (`Equal`, `Less`, `GreaterOrEqual`, and so on) name
the comparison semantics outright, and the enum's own type name probably carries the
family prefix. See `league_structs/docs/reversing/DebugBuild_ReflectionNames.md`. The
block is about 18 months stale, so a 16.17 class will not be in it, but the
**vocabulary** it yields is what the searches above are short of.

### 4.4 The macOS build

Per `docs/string-attestation.md` the repo cannot extract strings itself and must be
given a dump. Ask for the **macOS** build: its `__TEXT,__cstring` extracts far
cleaner than the Windows one, and game-string coverage is complete. Caveat that cuts
the other way: macOS RTTI class names go to zero, so it is cleaner but not a superset.
The dump's provenance rule applies - a non-retail source is never named in docs,
batch notes, commits or PRs.

### 4.5 Wait for shipped data

`bin-grep --subclasses` returns **0 matches across all 392 WADs** for every root in
this system. The control (`LogicDriverViewController`, `0xf0e5d4f6`) returns 22 in
`UI.wad.client`, so the scanner works and the zero is real.

The moment Riot ships one authored instance, tier 1 evidence appears: object paths,
the variables these bind to, and the strings beside them. `Nova` is a live codename
with clientconfig queues, so this is a question of when. **Re-run the scan on every
new dump.** This is the cheapest route on the list and it costs nothing but patience.

### 4.6 The LCU and rcp side

Nova's UI half may exist as `rcp-be-lol-game-data` assets or LCU plugin code, which
are plain files rather than hashed meta. `hashes.lcu.txt` is the index. The current
`nova` hits there are all unrelated (`Dreadnova`, `frostnova`), but that is a
statement about 16.17, not about the next build.

### 4.7 Sibling codenames as vocabulary

`JADE` is further along than Nova: it ships authored map data on `Map12` and
`Map453`, and a full HUD replacement including `jadeitemshop` and `scoreboard_jade`.
`JadeItemRecommendations` (`0x90fcea80`, 16.13) has five unnamed fields and is
directly about items. Naming Jade's item classes would supply exactly the vocabulary
Nova's searches are missing, and the two are siblings by the
`*_RANKED_SOLO_5x5` queue pair. Consider working Jade first.

---

## 5. Status

All 56 rows `pending`, `pr=-`: 32 in `ledger.bintypes.jsonl`, 24 in
`ledger.binfields.jsonl`, one batch `nova-item-getters`. No PR is open.

| group | count | tier | fit to submit |
| --- | ---: | --- | --- |
| getter lattice, 26 classes | 26 | 3 | yes |
| field names, 24 | 24 | 3 | yes |
| leaf getters, 6 classes | 6 | 4 | **flag or split off** |
| everything in section 3 | 13 | - | no, open |
| `nova-item-selection-filters`, 23 classes and 7 fields | 30 | 3 | yes, see section 6 |

Casing follows the repo rule: PascalCase, acronyms title-cased. Word boundaries are
attested rather than invented - `Nova` by the upstream
`NovaScoreboardViewController`, `Item` and `Get` by `MonarchPropertyGet` and by
corpus counts. `ItemId` follows the repo's title-cased-acronym rule rather than
`ItemID`, which is the one call in this batch that a shipped string could still
overturn. Twelve fields were drafted in the engine's own camelCase and landed
recased - `categoryID` as `CategoryId`, `mult` as `Mult`, and so on. The bin-hash is
FNV-1a over the lowercased name, so none of them moved.

Full working notes, including the shipped-data scan and the two unrelated 16.17
findings, are in `league_structs/docs/reversing/NovaItemGet_TypedGetters.md`. The
cross-tree stem driver is `league_structs/tools/fnv_stemcross.py`.

---

## 6. The filter tree, solved - batch `nova-item-selection-filters`

### 6.1 What fixes the names

Suffix folding with **one** word per member, not a typed role word. Folding `and`,
`not` and `or` out of the three logic nodes gives one state, `0x478f319d`. Eleven more
members then land on that state:

| pass | probes x targets | expected noise | hits |
| --- | --- | ---: | --- |
| fold one vocabulary word out of all 16 | 5.0e4 states | - | `and` / `not` / `or` agree on `0x478f319d` |
| state + 2 words | 1.0e7 x 13 | 0.03 | `HasTag`, `IsItem` |
| state + type word + 2 words | 4.1e7 x 8 | 0.07 | all eight `<Type>DataCompare` / `<Type>ValueCompare` |
| state + one of 9 lead words + 2 words | 9.2e7 x 1 | 0.02 | `IsValidItem` |
| root: five `I…` leads + 2 words, 27,631-word list | 3.8e9 x 1 | 0.89 | `INovaItemSelectionFilter` |
| 9 stems + up to 5 characters of `[a-z0-9_]` | 6.4e8 x 1 | 0.15 | the root stem + exactly `_` is the state `0x478f319d` |
| 11 stems + 1 word, 378,779-word list | 4.2e6 x 3 | 0.003 | `NovaItemSelectionFilterList` = `0x21820ed8` |
| word + one of 4 stems + word | 3.1e9 x 1 | 0.71 | `LinkedNovaItemSelectionFilterList` = `0xd87aa3f7` |

The fold also returns a second agreeing triple, `lod` / `ult` / `vs` on `0x7d2b76fb`.
It is an FNV artifact of the first and not a second stem: no other member lands on it.
A three-way agreement on one state is therefore worth less than 96 bits by itself, and
the eleven further members are what carry this one.

Independent of the hashes:

- **Registration order.** In the 16.18 client the sixteen registrars sit directly
  after `NovaItemGetTooltip` in exactly the sorted order of the names: `…FilterList`,
  then `…Filter_And`, the eight compares, `_HasTag`, `_IntDataCompare`,
  `_IntValueCompare`, `_IsItem`, `_IsValidItem`, `_Not`, `_Or`, the two string
  compares. `L` sorts before `_`, which is why the list comes first. The root sits in
  the interface run after `INeutralCampSpawnBehavior`, and the linked list in the `L`
  run after `LevelScriptOnUpdate`.
- **The reader.** Each class's evaluate function in the 16.18 client does what its
  name says (section 6.4). This is what separates `And` from `Or`, which structure
  cannot.
- **Convention.** `ViewControllerFilter_And` and `OptionItemFilter_And` are upstream
  names of the same shape.

### 6.2 Evidence table - landed

| hash | table | name | what fixes it |
| --- | --- | --- | --- |
| `0x70f6f74b` | class | `INovaItemSelectionFilter` | root; its stem plus `_` is the proved state |
| `0x21820ed8` | class | `NovaItemSelectionFilterList` | root stem + `List`; the `Condition` embed of every `…If` getter |
| `0xd87aa3f7` | class | `LinkedNovaItemSelectionFilterList` | `Linked` + the list's name; the reader resolves its `Filters` link to a list and evaluates it |
| `0x1cd34280` | class | `NovaItemData` | registered directly before the `NovaItemGet` run; link target of the `NovaItem` field below |
| `0x556b035c` | class | `NovaItemGetImageBySlot` | proved `NovaItemGet` stem, field `slot`, registered between `…GetIcon` and `…GetImage` |
| `0x254a25a7` | class | `NovaItemGetStatValue` | proved stem, field `Stat`, registered between `…GetItemId` and `…GetStringIf` |
| `0x83efb6f9` | class, field | `WeightedTags` | one hash as class and as field; the class is one `Map String F32`; the `HasTag` reader needs a weight above zero |
| `0x21d40f1b` | field | `TagList` | `Pointer ITagList`, beside `WeightedTags` |
| `0xaa8b4ed6` | class | `IItemGameModeData` | interface base of `NovaItemData`; pairs with the field below |
| `0x5f309a68` | field | `GameModeData` | `ItemData`: `Map Hash Pointer IItemGameModeData` |
| `0xc10d4fdc` | class | `ISubphaseAdditionalData` | interface, registered after the filter root; pairs with the field below |
| `0xa42117a5` | field | `SubphaseAdditionalData` | `LolModesSubphaseData`: `List2 Link ISubphaseAdditionalData` |
| `0x09a05a12` | field | `DataGetter` | the getter all eight compares evaluate first |
| `0x0aeac70d` | field | `OperandGetter` | the second getter of the four `…DataCompare` nodes, where `…ValueCompare` holds `Operand` |
| `0xf82a9a98` | field | `NovaItem` | on `…_IsItem` beside `Item`; the reader resolves it as a link to `NovaItemData` |

`NovaItemGetImageBySlot`, `NovaItemGetStatValue`, `WeightedTags`, `IItemGameModeData`
and `ISubphaseAdditionalData` came out of one broad two-word sweep that was far above
one expected chance hit per target. What lands them is the slot, the pairing or the
reader in the right-hand column, not the hash. Treat them as tier 4 where only a field
name backs them (`NovaItemGetImageBySlot`, `NovaItemGetStatValue`).

### 6.3 Evidence table - the fourteen subclasses

Landed in the same batch. They carry a `_` separator, which the naming rule refused
until this campaign; the rule now keeps a separator as written (`scripts/names.py`).
Each is fixed by the shared state of section 6.1, its place in the registration order
and its reader.

| hash | name |
| --- | --- |
| `0xc7ad307e` | `NovaItemSelectionFilter_And` |
| `0x5cf296d4` | `NovaItemSelectionFilter_BoolDataCompare` |
| `0xe7791d51` | `NovaItemSelectionFilter_BoolValueCompare` |
| `0x17975a2c` | `NovaItemSelectionFilter_FloatDataCompare` |
| `0x2cb48189` | `NovaItemSelectionFilter_FloatValueCompare` |
| `0x63d91c69` | `NovaItemSelectionFilter_HasTag` |
| `0x484bd29d` | `NovaItemSelectionFilter_IntDataCompare` |
| `0xb8efcc36` | `NovaItemSelectionFilter_IntValueCompare` |
| `0xf9420b5e` | `NovaItemSelectionFilter_IsItem` |
| `0xebbc64d2` | `NovaItemSelectionFilter_IsValidItem` |
| `0xf4198792` | `NovaItemSelectionFilter_Not` |
| `0x756c1ccc` | `NovaItemSelectionFilter_Or` |
| `0x2c2d1a17` | `NovaItemSelectionFilter_StringDataCompare` |
| `0x1f1eadd0` | `NovaItemSelectionFilter_StringValueCompare` |

### 6.4 What the reader does (16.18 client)

- A filter is evaluated against one item. Every filter that reads the item is false
  when there is none.
- `NovaItemSelectionFilterList`: every filter must pass. An empty list passes.
  `NovaItemGetBoolIf` returns no value when its `Condition` list fails; the other
  `…If` getters were not read.
- `_And`: same as the list. `_Or`: any filter passes, and an empty one fails.
  `_Not`: the negation of `Filter`.
- `LinkedNovaItemSelectionFilterList`: resolves `Filters` to a list object and
  evaluates it. An unresolved link fails.
- `_IsValidItem`: an item is present.
- `_IsItem`: the item is the one `Item` links. When `Item` does not resolve, the one
  `NovaItem` links.
- `_HasTag`: `tag` matches a name in the item's `WeightedTags` with a weight above
  zero, or one of its `tags`.
- The compares read `DataGetter`, then `OperandGetter` or `Operand`, and fail when
  either getter returns no value. `Operator`: 0 equal, 1 not equal, 2 greater,
  3 less, 4 greater or equal, 5 less or equal. The string compares accept 0 to 3
  only.

### 6.5 Negatives and leftovers

- The binding tree does not fold: `bool` / `float` / `image` / `int` / `text`, with
  `boolean`, `icon`, `integer` and `string` as alternates, crossed with one word from
  the 27,631-word list before or after the type word and with a `_` in three
  positions. No state shared by all five.
- `0x51486c30` (Bool) and `ObjectName` joined `NovaItemSelectionFilterList` in 16.19.
  The two-word sweep returns only `DebugCategory` for the Bool, which nothing
  supports. Not landed. No reader: the 16.19 client is not readable.
- `0x5e16be82`, `0x127a3f97`, `0x78812955`, `0xc752c9d7`, `0x9b8a2421`, the binding
  tree, the input tree and the empty pair stay open.
- `0xdc7275e9` holds `TagList` and `WeightedTags` and stays unnamed.
- A filter-tree stem of plain words does not exist: `Nova`, `NovaItem`, `NovaShop`
  and about 30 further leads crossed with one word of the 378,779-word list and two of the
  27,631-word list returned noise only. The separator is the reason.

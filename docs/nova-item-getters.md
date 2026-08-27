# nova-item-getters - campaign record

Record for `nova-item-getters`. Method: the `crack-family` skill
(`.claude/skills/crack-family/SKILL.md`). 56 names are landed and `pending`, 30
hashes are open, and the negatives below say where not to spend.

Patch 16.17 (build `8104348`) introduced 85 classes. 62 of them form one system across
nine roots, and **all 62 were live unnamed**. The first pass proved and landed 32,
which left 30. A second pass decomposed 11 of those 30 down to a single unknown
prefix (section 3.1) but landed none of them, so 30 are still open.

| root | what it is | members | proved | left |
| --- | --- | ---: | ---: | ---: |
| `0x822cf77c` | Bool getter tree | 7 | 7 | 0 |
| `0xa79ac316` | Float getter tree | 6 | 6 | 0 |
| `0x53bee89d` | Int getter tree | 9 | 8 | 1 |
| `0x8d941811` | Image getter tree | 7 | 5 | 2 |
| `0x3f8dac45` | String getter tree | 6 | 6 | 0 |
| `0x70f6f74b` | filter / predicate tree | 16 | 0 | 16 (11 decomposed, see 3.1) |
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

A second pass added these. The first block is the one that paid.

| run | probes | targets | expected noise | outcome |
| --- | ---: | ---: | ---: | --- |
| `recover_stem`, 2-token tails, both comparison sets | 1.83e9 | 4 each | 2.3e-20 | **the stem, twice** |
| `0x478f319d` x 1 corpus token, filter tree | 3.08e4 | 17 | 1.2e-4 | **`And`, `Or`, `Not`** |
| `recover_stem`, 2-token tails, binding tree | 1.46e10 | 5 | 4.3e-29 | no consistent stem |
| 1-token sweep, every open field hash, 73,816-word vocab incl. 2.3M game paths | 7.38e4 | 21 | 3.6e-4 | 0 hits |
| `crack_pair`, 18 oppositions x 4 field pairs | 2.73e11 | 4 pairs | 1.5e-8 | 0 hits |
| suffix folding, field hashes vs 55 anchored states | 6.47e5 | 21 | 0.008 | 0 hits |
| shared-stem across the open field hashes | 3.08e4 | 210 pairs | 0.002 | 0 hits |
| input tree anchored on `keybind` / `binding` / `bind` | 4.74e9 | 3 | 3.3 | 4 hits, all noise |
| `P` from nova-anchors + 2 corpus tokens | 1.04e10 | 1 | 2.4 | 2 hits, both noise |
| `P` from a 159-word curated list, depth 2-4 | 6.39e8 | 1 | 0.15 | 0 hits |
| `P` anchored both ends, 14 endings x 7 anchors | 9.29e10 | 1 | 21.6 | 18 hits, all noise |
| `P` by `crack_pair` against the root and the embed, 28 shapes | 2.65e10 | 1 | 1.4e-9 | 0 hits |
| both proved states x 594,671 tails | 1.19e6 | 25 | 0.007 | 0 hits |

Two readings. **The type word was never last**, which is why the first pass's
`recover_stem` on these sets returned nothing: the tail is two tokens
(`DataCompare`, `ValueCompare`), and a one-token suffix list cannot reach it.
**`P` resists everything a closed vocabulary can do** - 1.4e-9 noise on the 64-bit
`crack_pair` is as clean a zero as this repo can produce, so the missing token is
not in any name the game has shipped. That is the same stopping condition section 4
already describes, and it points at the same routes.

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

## 3. What is left - 30 open hashes

### 3.1 The filter tree, `0x70f6f74b` (16, highest value)

A predicate tree over the getter trees. `0x21820ed8` is the `Condition` embed every
`…If` node holds, and it is not a subclass.

```
0x70f6f74b   INTERFACE, no properties
0x21820ed8   Filters: List2 Pointer 0x70f6f74b     (embed, no base)

  provider vs provider   { 0x9a05a12: Pointer T, Operator: U8, 0xaeac70d: Pointer T }
    0x17975a2c  T = NovaItemGetFloat      0x2c2d1a17  T = NovaItemGetString
    0x484bd29d  T = NovaItemGetInt        0x5cf296d4  T = NovaItemGetBool

  provider vs literal    { Operand: <T>, 0x9a05a12: Pointer T, Operator: U8 }
    0x2cb48189  Operand F32               0x1f1eadd0  Operand String
    0xb8efcc36  Operand I32               0xe7791d51  Operand Bool

  0x756c1ccc   Filters: List2 Pointer 0x70f6f74b    and / or
  0xc7ad307e   Filters: List2 Pointer 0x70f6f74b    and / or
  0xf4198792   Filter:  Pointer 0x70f6f74b          not
  0xd87aa3f7   Filters: Hash
  0x63d91c69   tag: Hash
  0xf9420b5e   Item: Hash, 0xf82a9a98: Hash
  0xebbc64d2   no properties                        constant
```

**Lead 1 is spent, and it worked.** `recover_stem` with the type word off the tail
returns a consistent stem for *both* comparison sets, and both land on the same
prefix state `0x478f319d`:

| set | tail folded out | members | check | expected noise |
| --- | --- | ---: | --- | ---: |
| provider vs provider | `DataCompare` | 4 | 96-bit | 2.3e-20 |
| provider vs literal | `ValueCompare` | 4 | 96-bit | 2.3e-20 |

Crossing that state with one corpus token then returns `And`, `Or` and `Not` at
1.2e-4 expected noise, which settles the pair section 4.2 said no hash search could
separate. Eleven of the sixteen now decompose, all on one unknown prefix `P` with
`fnv(P) = 0x478f319d`:

| hash | name | what fixes it |
| --- | --- | --- |
| `0xc7ad307e` | `<P>And` | stem + `And`; holds `Filters` (`List2`) |
| `0x756c1ccc` | `<P>Or` | stem + `Or`; holds `Filters` (`List2`) |
| `0xf4198792` | `<P>Not` | stem + `Not`; holds `Filter`, **singular, one pointer** |
| `0x5cf296d4` | `<P>BoolDataCompare` | cross-set stem |
| `0x17975a2c` | `<P>FloatDataCompare` | cross-set stem |
| `0x484bd29d` | `<P>IntDataCompare` | cross-set stem |
| `0x2c2d1a17` | `<P>StringDataCompare` | cross-set stem |
| `0xe7791d51` | `<P>BoolValueCompare` | cross-set stem |
| `0x2cb48189` | `<P>FloatValueCompare` | cross-set stem |
| `0xb8efcc36` | `<P>IntValueCompare` | cross-set stem |
| `0x1f1eadd0` | `<P>StringValueCompare` | cross-set stem |

Four independent agreements on `0x478f319d`: the two 96-bit folds, the three role
words, and the base check - all eleven have `0x70f6f74b` as their base. The
`Filters` / `Filter` split is corroboration the search never saw: `And` and `Or`
take a list, `Not` takes one. `DataCompare` compares two providers, `ValueCompare`
compares a provider against the literal in `Operand`, which is what those two
signatures do.

**Nothing here can be landed until `P` is recovered** - a name needs the whole
string. Section 2 records what has already failed against it. The underscore form of
lead 2 is refuted: the role words concatenate, so it is `<P>And`, not `<P>_And`.

Remaining leads:

1. **Recover `P`.** One 32-bit state with no cross-check, the same wall the
   `novaitemget` half hit. It is worth more than the rest of this doc combined: it
   lands eleven classes at once.
2. **`crack_pair` on `0x9a05a12` and `0xaeac70d`.** Still open. They are the two
   comparison operands, each on four classes, so solving them together is a 64-bit
   constraint.
3. **`Operator: U8` is an enum.** Its member names would name the tree by
   association. See section 4.3.

### 3.2 The variable-binding tree, `0x40452a8d` (6)

```
0x40452a8d   INTERFACE, no properties
  0x49c42124   IntVariable   + IntProperty   -> NovaItemGetInt
  0xc8714ff3   BoolVariable  + BoolProperty  -> NovaItemGetBool
  0xdbab6aeb   FloatVariable + FloatProperty -> NovaItemGetFloat
  0xd46044c0   ImageVariable + ImageProperty -> NovaItemGetImage
  0x8aa21ee4   TextVariable  + TextProperty  -> NovaItemGetString, plus 0x78812955: Bool
```

Typed-parallel, so `recover_stem` applies directly, and it is now spent: the retry
ran with widened type words (`string`/`text`/`str`/`loc`/`label`/`name`/`tra` for the
text node, eight words for image) against 594,671 one- and two-token tails, a 128-bit
check at 4.3e-29 noise, and returned **no consistent stem**. The same run recovered
the two filter stems, so the machinery was working.

That is a real result, not a null: whatever these five share, it is not
`<stem><typeword><tail>`. Either the type word is absent from the name, or it is not
at a token boundary the corpus can supply. Note the extra `Bool` on the text node
only.

### 3.3 The input reference tree, `0x19ccc111` (3)

```
0x19ccc111   INTERFACE, no properties
  0x2142e1f7   SpellSlot: Embed SpellBookIndex
  0xafd2f805   InputEvent: Hash, 0xb915a5ee: String
```

`SpellBookIndex` is the spell-slot keybind struct Monarch introduced at 14.14 and
mainline adopted at 15.3. `Keybind` as the last word, which both
`MonarchSpellSlotKeybind` and `LoLSpellSlotKeybind` use, has been tried and missed:
anchored on `keybind`/`keybinds`/`keybinding`/`binding`/`bind` with a two-token stem
it returned 4 hits against 3.3 expected noise, which is nothing. Anchoring the front
on `nova*` as well drops it to 0.001 noise and 0 hits.

### 3.4 Three getter nodes

```
0x556b035c  base NovaItemGetImage   slot: Pointer NovaItemGetInt
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
| `0xd6fbee84` | `novaitemget`, proved and resolved | none, spent |
| `0x478f319d` | the filter-tree stem, **proved as a state, prefix unrecovered** | 11 |
| - | the filter-tree leftovers, unknown | 5 |
| - | the binding-tree stem, unknown | 6 |
| - | the input-tree stem, unknown | 3 |

`0x478f319d` is the single highest-value target in this campaign. Neither proved
state reaches any other open hash: both were crossed with 594,671 one- and two-token
tails against all 25 remaining hashes at 0.007 expected noise, for 0 hits.

### 3.7 Open field hashes

| hash | on | type | note |
| --- | --- | --- | --- |
| `0x5e16be82` | the five `…If` nodes | `Pointer <self>` | **five classes at once, so a 5-way `crack_pair` target.** Best single field lead |
| `0x9a05a12` | 8 filter nodes | `Pointer T` | left comparison operand |
| `0xaeac70d` | 4 filter nodes | `Pointer T` | right comparison operand |
| `0xf82a9a98` | `0xf9420b5e` | `Hash` | beside `Item` |
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
mirror the tree.

This section used to say that no hash search could separate `0x756c1ccc` from
`0xc7ad307e`, because their signatures are identical. That was wrong, and section 3.1
records how: once the tree's stem state was proved, one corpus token off it returned
`Or` and `And` respectively at 1.2e-4 expected noise. Identical *signatures* still
leave the names distinguishable when the stem is known. The route below keeps its
value for the roles no word landed on, and for what the tree is actually wired to.

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

All 56 rows `pending`, `pr=-`: 32 in `ledger.bintypes.tsv`, 24 in
`ledger.binfields.tsv`, one batch `nova-item-getters`. No PR is open.

| group | count | tier | fit to submit |
| --- | ---: | --- | --- |
| getter lattice, 26 classes | 26 | 3 | yes |
| field names, 24 | 24 | 3 | yes |
| leaf getters, 6 classes | 6 | 4 | **flag or split off** |
| everything in section 3 | 30 | - | no, open |

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

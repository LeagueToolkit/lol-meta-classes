# Plan: vocabulary expansion

Recombination cannot reach a name containing a word the corpus has never used.
This is the plan for supplying those words without drowning the noise budget.

## What the gap actually is

Measured over the 14,949 names the repo knows, split with `scripts/names.py`,
after track 0:

| | |
|---|---|
| distinct words | 3,202 |
| words used in exactly one name | 954 (29% of the vocabulary) |
| names holding such a word | 914 (6%) - unreachable from the rest of the corpus |

Widening the threshold: names whose rarest word appears in <=2 names, 12%; <=3,
16%; <=5, 25%. So the reachable ceiling for pure recombination is somewhere
around 75-94% of names, and the top of that range is what this work buys.

The single-use words split three ways, and they want three different fixes:

| kind | count | example | supplied by |
|---|---|---|---|
| novel | 643 (66%) | `Roughness`, `Skybox`, `Skinning`, `Reverb`, `Silhouette`, `Stride` | track 2 |
| inflection | 247 (26%) | `Clones`, `Curves`, `Conversions`, `Coloring` | track 1 |
| numbered | 77 (8%) | `Color2`, `Custom1`-`Custom10`, `Crown1`-`Crown4` | track 1 |

A fourth kind hid inside "novel": `Statemachine`, `Gamemode`, `Itemfilter` are
not words, they are word-boundary failures in names spelled without the
boundary. Each is a junk token *and* two real words withheld. That was track 0,
now done.

Current unresolved targets: 2,421 classes + 4,483 fields = 6,904.

## The economics, which decide the shape of everything below

Noise is `probes x targets / 2^32`, and adding `V` words multiplies probes.
`mutate` over the top 500 words already costs 151 expected false positives in
the broad tier, so a globally-widened wordlist is unaffordable by construction.

Two things make expansion affordable, and every run below uses at least one:

- **`--only`**: a family of 70 hashes instead of all 6,904 is a 99x noise
  discount. Vocabulary expansion pays off *inside families*, not globally.
- **At most one new word per generated name.** A name is normally corpus words
  with one novel term in it. Capping extras at 1 makes cost `|V| x (corpus
  arrangements)` instead of `|V|^depth`, which is the difference between
  affordable and not.

Track 0 was the exception: it *shrinks* the wordlist rather than growing it, so
it cost nothing and went first.

## Track 0 - boundary repair (done)

17 names we author were re-cased to put the boundary back. Case-only, so no
hash moved; the wordlist went 3,162 -> 3,153 words and no new token entered,
because every recovered half was already in it. Nine tokens died outright:
`Botspell`, `Edgepan`, `Filtername`, `Gamemode`, `Healthbars`, `Itemfilter`,
`Middleframe`, `Statemachine`, `Uvoffset`. The other eight survive only in
upstream's mirror, which we do not rewrite.

A glued token was fixed only where the split spelling is attested. Evidence
ranks: a separator in a shipped path (a real byte, not a casing guess) over a
cased CDragon table over this repo's own corpus. Counts are occurrences across
`CommunityDragon/Data`'s tables; the lowercased ones (`hashes.game.txt`,
`hashes.lcu.txt`) can only ever show a separator, so they are read for that
alone.

| name | now | attestation |
|---|---|---|
| `Statemachine` | `StateMachine` | `state-machine` x5 paths, `StateMachine` x3 |
| `GamemodeDefinition`, `GamemodeEntity` | `GameMode*` | `game-mode` x28, `GameMode`/`gameMode` x88 |
| `BotspellTag` | `BotSpellTag` | `icon-bot-spell-placeholder.png`, `BotSpell1` |
| `SearchbarGroup` | `SearchBarGroup` | `ItemShop_SearchBar` x95 |
| `Healthbars`, `HealthbarBackground` | `HealthBar*` | `health_bar` x68, `healthBarData` x75 |
| `AutoattackMode` | `AutoAttackMode` | `Auto_Attack` x26, `AutoAttack` x54, glued 0 |
| `EquipTrakey`, `EquippedTrakey` | `*TraKey` | `ActiveTooltipTraKey` and 84 more |
| `InitialFramerate` | `InitialFrameRate` | `frameRate`, `frameRateMult` |
| `InitialUvoffset` | `InitialUvOffset` | `birthUVOffset`, `flexBirthUVOffset` |
| `ITftItemfilter` | `ITftItemFilter` | `IOptionItemFilter`, `OptionItemFilter_*` |
| `FilternameTraKey` | `FilterNameTraKey` | every other `*TraKey` splits its stem |
| `InventorySlotlinks` | `InventorySlotLinks` | siblings `LevelUpLinks`, `AbilityLinks`, `SummonerSpellLinks` |
| `Middleframe` | `MiddleFrame` | none either way; not a word in any corpus |
| `OffsetEdgepanSpeed` | `OffsetEdgePanSpeed` | none either way; not a word in any corpus |

Left glued, because the evidence says glued or says nothing and the token is a
word: `Backlight`, `Backpack`, `ClippingGuard`, `Datacenter`, `Deadzone*`
(1 v 1), `DamageOvertime` (glued x13, split 0), `CombatOverviewViewController`,
`SubteamBadge` (glued x151), and `Playstyle*`, whose spelling a shipped string
fixes (`docs/string-attestation.md`).

**Step 3 as first planned - a casing-only override per upstream capital run -
was dropped, measured rather than argued.** The 394 names are 71 distinct runs,
and `split_words.py` case-folds its dedup key while the guesser dedups words by
lowercase, so `TFT`/`UI`/`VFX`/`ASSETS`/`SAMPLER` already merge into
`Tft`/`Ui`/`Vfx`/`Assets`/`Sampler`. The wordlist holds zero case-variant
duplicates today. Only `PAR` (a genuine acronym) and `PIPVFX` survive as
caps-only entries, and only `PIPVFX` hides a boundary - one junk token for 394
permanent overrides that would each have to disagree with upstream forever.

## Track 1 - morphology (derived, no external sourcing)

Generate from the corpus itself, so nothing unattested enters:

- plural/singular pairs, `-ing`/`-ed`/`-er` forms of existing words
- `<Word><digit>` for words already seen numbered, digits 1-10
- the observed misspellings, kept as-is (`Defalt`, `Controle`, `Respoition`,
  `Customised`) - Riot's typos are attested vocabulary

Expected size 300-600 words. These are cheap because they are *shaped* like
corpus words and can be frequency-ranked by their base word's frequency, so
`--top` keeps working.

## Track 2 - domain vocabulary (external, ranked by attestation)

Source in this order, best first. The ranking matters more than the total: a
word Riot uses somewhere is worth many words from a glossary.

1. **CommunityDragon's other hashtables** - `hashes.game.txt`, `hashes.lcu.txt`,
   `hashes.binentries.txt`, `hashes.binhashes.txt`. Hundreds of thousands of
   real Riot paths and entry names, already public, already attested. Split them
   with `scripts/split_words.py` and take words absent from our vocabulary. This
   is the single highest-value source and should be exhausted before any of the
   rest.
2. **Shipped shader and asset names** already cracked here (`LightRegions.ps`
   and the `light-regions` / `gameplay-texture` batches).
3. **Public rendering and engine glossaries** - GL/Vulkan/D3D terms, common
   engine subsystem nouns. Last, and capped: these inflate `p` with words Riot
   may never use.

Rank extra words by frequency in their *source* corpus so `--top` still means
something, and keep the tiers in separate files so a run can take 1 without 3.

## Guesser change this needs

`crates/hash-guesser` has no way to say "these words are speculative". Add:

    --extra-words <file>     a second wordlist, tried alongside --words
    --max-extra <n>          how many may appear in one candidate (default 1)

`identity` and `delete` ignore it. `mutate` and `force` respect the cap. `chain`
must either exclude extras or let an extra sit anywhere in the chain, because a
new word has no attested adjacency - state which, and print the probe count
either way, as `chain` already does.

The run banner must report probes and expected false positives with extras
included, or the budget stops being checkable.

## Calibration gate, before any noise is spent

The 927 unreachable names are ground truth and cost nothing to test against.

1. Hide them from the wordlist.
2. Run each track's vocabulary against those 927 hashes with `--only`.
3. Report recall per track and per source tier.

This is the same shape as the semantic pass reproducing 2,080 already-named
classes. **Do not run against unresolved hashes until a track clears this.**
A track that cannot re-find known names will not find unknown ones, and its
words should be dropped rather than carried at a cost to every later run.

Suggested bar: track 1 >=40% of the inflection/numbered names, track 2 tier 1
>=15% of the novel ones. Below that, the tier is not paying for its `p`.

## Run plan

Once a track clears the gate, per family, not globally:

    python3 scripts/split_words.py hashes/hashes.*.txt hashes/overrides/*.txt > words.txt

    cargo run --release -p hash-guesser -- bintypes \
        --words words.txt --extra-words vocab.tier1.txt --max-extra 1 \
        --only family.txt --prefix <family stem> \
        --mode mutate --suffix Data --suffix Controller

Families come out of `scripts/guesser_families.py emit`, which already writes
one pack per class family. Sweep with `scripts/guesser_sweep.py` and read
`MANIFEST.tsv` for what each run cost before reading its hits.

## Landing rules

Unchanged, and worth restating because a bigger wordlist produces more
plausible-looking noise, not less: a hit is a candidate. It needs attestation in
shipped data before it enters a table, the batch note records the method, and
the per-name evidence goes in a doc under `docs/`. Names follow the casing rule
in `scripts/names.py` - `hashtool add` enforces it.

Record each vocabulary tier's size and its calibration recall in the batch note,
so a later reader can tell which tier a name came from.

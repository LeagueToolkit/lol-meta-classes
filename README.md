<div align="center">
  <a href="https://github.com/LeagueToolkit">
    <img src="https://avatars.githubusercontent.com/u/28510182?s=200&v=4" alt="LeagueToolkit logo" width="96" height="96">
  </a>
  <h1>lol-meta-classes</h1>
</div>

A Git-tracked database of League of Legends meta (`.bin`) classes and properties, dumped from the
game client on every patch. It records **which builds each class and property existed in, and every
definition it ever had** - type changes, base changes, removals - starting at patch 13.15. The
dumps, the generated database, and the tooling that produces both live here; a scheduled pipeline
keeps them current without a human in the loop.

<div align="center">

**[Layout](#layout)** · **[Database files](#database-files)** · **[Hashtables](#hashtables)** · **[Regenerating locally](#regenerating-locally)** · **[Automation](#automation)**

</div>

## Layout

```text
dumps/          raw per-build dumps from the game client, e.g. 16.13.7915903.json
db/             generated database - meta.db.json (versioned) + database.py (latest-build snapshot)
hashes/         type/field hash -> name tables, plus local overrides
crates/         the Rust toolchain that produces dumps/
scripts/        the Python that turns dumps/ into db/
docs/           format specs for both database files, plus per-campaign reversing write-ups
```

`dumps/` is the source of truth. Everything under `db/` is generated from it and is rebuilt from
scratch on every run, so never hand-edit those files.

One field there is easy to misread. A property's `value_type` of `Hash` does **not** name a hash
function or a width - both live on a helper object hanging off the property record, and the reader
asks the helper how many bytes to consume before reading any. From format version 3 the dump
describes those helpers, each with its storage width and its algorithm, the latter measured by
calling the helper rather than by reading its code. Anything that *writes* a `Hash` value has to
look at it: two properties can both be `Hash` and disagree on both counts.

An image has a handful of helpers standing behind thousands of `Hash` properties, so they are
interned in a top-level `hashers` table keyed by vtable and a property names its own with `hasher`:

```json
"hashers": {
  "0x25d0ff0": {
    "storage_width": 4,
    "hash_function": { "algorithm": "Fnv1a32", "lowercased": true }
  }
},
"classes": { "0x…": { "properties": { "0x…": { "value_type": "Hash", "hasher": "0x25d0ff0" } } } }
```

## Database files

### `db/meta.db.json`

The machine-readable database, and what you want if you care about history. For every class and
property it stores the build intervals it was present in and an ordered list of its distinct
definitions, so you can ask when a property appeared, when its type changed, and whether it still
exists in the current patch.

Full format spec: [docs/meta-db-format.md](docs/meta-db-format.md).

### `db/database.py`

A snapshot of the **latest build only**, written as Python-like text. It is not valid Python - the
format exists because a class-per-block, one-property-per-line layout produces readable Git diffs,
so a patch's schema changes are visible in the commit itself.

```python
class AbilityResourceByCoefficientCalculationPart(IGameCalculationPart):
    mCoefficient: (F32, 0x0, 0x0, 0x0) = 0.0
    mAbilityResource: (U8, 0x0, 0x0, 0x0) = 0
    mStatFormula: (U8, 0x0, 0x0, 0x0) = 0
    pass
```

Each property line is `Name: (field type, key type, value type, referenced class)` with the default
value appended when there is one.

Full format spec: [docs/database.md](docs/database.md).

### Browsing it

The database is published as a searchable wiki at
[meta-wiki.leaguetoolkit.dev](https://meta-wiki.leaguetoolkit.dev/), built from `db/meta.db.json` by
[LeagueToolkit/lol-meta-wiki](https://github.com/LeagueToolkit/lol-meta-wiki).

## Hashtables

Classes and properties appear in the game binary as FNV-1a-32 hashes, so names have to be resolved
from lookup tables. Anything that resolves to nothing stays a raw `0x...` hash in the output, in
both database files.

There are two layers, deliberately kept separate:

- `hashes/hashes.bintypes.txt`, `hashes/hashes.binfields.txt` - exact mirrors of the upstream
  [CommunityDragon](https://raw.communitydragon.org/data/hashes/lol/) tables. Sources are declared
  in `hashes/sources.toml`; only `scripts/update_hashes.py` writes these files.
- `hashes/overrides/` - local cracks upstream does not have yet. These are **not** baked into the
  mirror; `read_resolved_hashes` layers them on top at build time.

Keeping them apart means `git diff hashes/hashes.*.txt` is pure upstream drift and
`git diff hashes/overrides/` is your own work. Overrides should also be submitted upstream to CDTB;
`update_hashes.py` flags entries that upstream has since picked up so they can be deleted.

Adding a crack:

```bash
# Hash a name without touching anything
python3 scripts/hashtool.py fnv GameEntityPrefab

# Add names to an override table (the file is re-sorted for you)
python3 scripts/hashtool.py add bintypes GameEntityPrefab
python3 scripts/hashtool.py add binfields EnvMesh

# Resolve either direction, across the mirror and the overrides
python3 scripts/hashtool.py lookup 2b949af2
python3 scripts/hashtool.py lookup GameEntityPrefab

# Diff an external name list against what the repo already resolves
python3 scripts/hashtool.py check names.txt

# Check the naming rule across both override tables and both ledgers
python3 scripts/hashtool.py lint
```

### Names are PascalCase

Every name this repo adds - to an override table or to a ledger row - is
PascalCase: an uppercase first letter, then letters and digits. `hashtool add`
rejects anything else outright, and `hashtool lint --fix` recases a table that
has drifted.

It is a rule rather than a preference because it feeds the cracking pipeline
below. Guessing a hash means recombining words taken from names already known,
so a name is worth precisely the words recoverable from it, and camelCase hides
the first boundary: `abilityHaste` yields "Haste" and loses "Ability". Every
future name built on the lost word is lost with it.

There is one exception, for a *single* lowercase letter in front of an otherwise
PascalCase name - `mCoefficient`, `bHaveHitBone`. It exists because the argument
above runs backwards there. A one-letter word is worthless in a wordlist, so the
splitter drops it and the camelCase spelling costs nothing; capitalizing it does
not recover a word but adds one, and `MCoefficient` would put "M" in the
vocabulary permanently. Upstream spells 2001 of its 9439 field names this way, so
the recased form is also the unattested one. Two or more leading lowercase
letters are still rejected, because there the capital does recover a real word -
the "Uv" of `uvMode`, the "Is" of `isClickable`.

Acronyms fall to the same argument and are title-cased: `Ui` not `UI`, `Tft` not
`TFT`, `Lol` not `LoL`, `Wasd` not `WASD`. The splitter reads a capital run as
one word, so `UIElement` spends a token nothing else can reuse where
`UiElement` yields two that recur everywhere. A few names are exempt because
attestation, not convention, put the capitals there - `MapSSAO` from the
shader's `OMIT_SSAO`, `GroupID` from upstream's `productID` on its own class.

Interface names need no exception: `IFoo` is already PascalCase, and the splitter
returns "I" + "Foo" rather than "IF" + "oo", which is what keeps `I` a word in
its own right. It is one of the most productive words in the wordlist.

Recasing is always safe - the bin-hash is FNV-1a over the *lowercased* name, so
`abilityHaste` and `AbilityHaste` are one hash. Separators are not, since they
are hashed like any other byte, so a name containing one cannot be normalized
and is rejected instead of rewritten.

The rule binds what we author. `hashes/hashes.*.txt` is upstream's file byte for
byte and stays as served, so the two layers can disagree about spelling - and an
override that differs from upstream only in casing is kept, not pruned: it is
the only thing making the database say `MapSSAO` where upstream says `MapSsao`.

### Cracking unresolved hashes

A hash with no name is a raw `0x...` in the database. Names in this game are
arrangements of a small vocabulary rather than arbitrary strings, so the attack
is to split the names we know into words and recombine them against the hashes
we are missing.

```bash
# the vocabulary, frequency-ordered, from every name the repo knows
python3 scripts/split_words.py hashes/hashes.*.txt hashes/overrides/*.txt > words.txt

cargo run --release -p hash-guesser -- binfields --words words.txt -o hits.txt
```

`hash-guesser` takes its target set from `dumps/` - every class and property
hash the game has used across every committed build, minus everything already
named - so there is no `all.*.txt` to maintain. Output is `{hash} {name}`,
PascalCase by construction and checked before printing, so it feeds straight
into `hashtool add`. (`.cargo/config.toml` pins the workspace to
`x86_64-unknown-linux-gnu` for the dumper's C++ stubs, so on Windows add
`--target x86_64-pc-windows-msvc`; the guesser is pure Rust and builds
anywhere.)

**Output is candidates, not cracks.** A 32-bit hash collides with plausible
names by chance, so nothing from here belongs in an override table until it has
been confirmed against shipped data. Record what confirmed it in the batch note
(`add -b <batch> -e "..."`).

#### The noise budget

`p` probes against `t` target states produce about `p·t/2³²` hits by luck alone.
Every run prints that number and warns when it approaches the hit count; read it
before reading the hits. It, not speed, is the binding constraint - the guesser
does ~10⁹ probes/second, so any run slow enough to notice is already deep in the
noise, and a smaller wordlist beats a bigger machine.

Both factors are levers. `--top N` cuts the wordlist to its first N entries,
which is where `p` is decided. `--only <file>` restricts the hunt to listed
hashes, which is where `t` is: one family of 70 buys a 36x deeper search than
the full ~2500 at equal noise.

#### Modes

| mode | builds | cost |
| --- | --- | --- |
| `identity` | every known name, verbatim, against the other table | one probe per name |
| `delete` | every known name minus one word | ~10⁵ probes total, no wordlist |
| `mutate` | each wordlist word substituted and inserted at each position of each known name | wordlist x names x positions |
| `force` | every arrangement of up to `--depth` words | `len(wordlist)^depth` |
| `chain` | `force`, but each word must follow a word it has followed in a known name | see below |

`identity` runs first and costs nothing: class and field vocabularies overlap
heavily, so a name in one table is a live guess for the other.

`delete` is the third edit beside `mutate`'s insert and substitute. It needs no
wordlist, which is why it is priced apart - the whole pass fits in ~10⁵ probes
at ~0.07 expected noise. The shape is common in the corpus itself: 5521 of
13799 known names are another known name with one interior word removed.
Deleting the *last* word is left to `identity`, which already probes every
prefix.

`chain` constrains generation to word pairs the corpus attests. Those pairs are
0.16% of all possible pairs over 3113 words, so the collapse is large: depth 3
falls from ~10⁸ probes to under 10⁶, and depth 4 costs ~18M probes at ~11
expected noise - less than uniform depth 3 over the top 400 words, which cost
128M and 73. The probe count is exact, computed by DP over the word graph and
printed before the run spends it.

Word *permutations* of known names were measured for the same slot and do not
earn one.

#### Anchoring the tail

`--suffix` searches one word deeper for almost nothing. Class names are named by
their tail far more often than their head - `Data` ends 702 known class names,
`Instance` 691, `Controller` 211, `Driver` 144, then `Def`, `Block`,
`Definition`, `Get`. Reaching those with an extra search word multiplies probes
by the wordlist size; anchoring the tail multiplies `t` by the number of
suffixes instead, which is a handful.

FNV-1a folds left to right and its prime is invertible mod 2³², so each suffix
is folded *backwards* out of every target once, before the search starts. The
search then hunts the state a name would be in before that suffix was appended,
and the hot loop never sees the suffix at all.

```bash
cargo run --release -p hash-guesser -- bintypes --words words.txt \
    --only family.txt --prefix Params \
    --mode force --depth 2 --suffix Data --suffix Controller
```

`--prefix` is the same trick at the other end, hashed once instead of folded.
Families come out of the meta: classes sharing a base overwhelmingly share a
first word, so a base class supplies both the `--only` list and the `--prefix`.

A single lowercase letter is a legal prefix, and on the field table it is the
only way to reach a whole naming convention. The engine spells member fields
`mCoefficient`, 2001 of the 9439 upstream serves, but "M" occurs zero times in
the wordlist - the splitter drops a one-letter prefix rather than emit it - so no
recombination can assemble that name at any depth. `--prefix m` supplies the
letter the wordlist cannot:

```bash
cargo run --release -p hash-guesser -- binfields --mode identity --prefix m
```

Paired with `identity` or `delete` this costs ~5x10^4 probes, under 0.07 expected
false positives. `force` and `chain` under the same prefix are dominated by noise
well before they add anything: at depth 2 over the full wordlist, 15 candidates
against 10.5 expected chance hits.

#### What can actually refute a candidate

One check in the tool is evidence rather than arithmetic. Guessing bintypes, a
candidate *named* `I[A-Z]` is held to a class the dump flags as an interface -
the flag comes from the dump, not the hash. Among names already cracked,
`I[A-Z]` implies that flag in 125 of 127 cases, and the filter refuses about
half of all I-named candidates. It is not free: `IOptionTemplate` and
`ISequenceActionInstance` are real names it would reject, so
`--no-interface-filter` exists. The converse is deliberately not applied - 191
of the 316 named interfaces are not I-named, so the flag says nothing about a
candidate that does not start with `I`.

The check is on the finished name, not on the search prefix, and that matters:
`I` is also a *word* in the wordlist, at rank 26, because it falls out of
splitting every `IFoo` name. A prefix-scoped check lets `force` and `mutate`
assemble `I`+`Model`+`Joint`+`Content` under the empty prefix and walk past it.

Filters that sound good and are not, each measured against names already
cracked:

| idea | signal | on candidates | verdict |
| --- | --- | --- | --- |
| a name shares a word with its base class | 91.1% | 81.9% | 1.1x, useless - the wordlist already encodes it |
| `is.value` implies a name suffix | `Data` tops it at 20% | - | no |
| interfaces are never nested | 323 real interface-to-interface edges | - | false |
| an I-name implies zero properties | 83.5% vs 17.3% | - | true, but the interface flag already says it |
| register order is ascending | 66%, median run 2 | - | no (that 89% is Windows PE order, not the macOS dump) |

The pattern, and the reason `chain` and `delete` are not on that list: a prior
built out of the *names* is already baked into the wordlist, so as a **filter**
it cannot discriminate. As a constraint on **generation** it can, because the
wordlist is unigram - word order and co-occurrence are not in it, and a prior
that stops sequences being probed at all lowers `p·t/2³²` directly instead of
trying to sort hits afterwards. Discriminating between hits needs facts from
outside the hash: the meta flags, and ultimately attestation in shipped data.

#### Running a sweep

One run answers very little. `scripts/guesser_sweep.py` runs a spread of them
and records what each cost in noise:

```bash
python3 scripts/guesser_sweep.py --merge-ref sequencer-actions
python3 scripts/guesser_report.py
```

Output lands in `hash-guesser-out/` (gitignored - a candidate must never be
mistaken for a crack):

| file | what it is |
| --- | --- |
| `MANIFEST.tsv` | per run: probes, target states, expected false positives, hits |
| `raw/<run>.<table>.txt` | one file per run, unmerged |
| `highconf.<table>.txt` | union of the quiet runs - the tier worth reading line by line |
| `candidates.<table>.txt` | union of everything that ran |
| `candidates.json` | the above joined to `db/meta.db.json` |
| `candidates.html` | a self-contained review page built from that JSON |

Runs are split into two tiers by expected noise, not by mode: `hi` stays near
single-digit expected false positives, `br` is broad and part chance by
construction. Edit the `RUNS` table at the top of the script to change the
spread; a raw file whose run leaves that table is deleted rather than folded
into the next union.

`--merge-ref <ref>` folds another branch's override tables into a scratch copy
of `hashes/` so its names count as known rather than being re-guessed, and its
vocabulary feeds the wordlist. The working tree wins on a hash both carry, which
matters when the ref predates `hashtool lint --fix`: `uvAnimation` costs the
wordlist a word that `UvAnimation` keeps, which is the whole reason for
[the naming rule](#names-are-pascalcase).

`scripts/guesser_report.py` joins the sweep to the meta so a candidate can be
judged on something other than the hash that proposed it - base classes, the
interface and value flags, field names, owning classes, declared type, and which
runs found it. Class hashes and class names link to
[meta-wiki.leaguetoolkit.dev](https://meta-wiki.leaguetoolkit.dev/), which
serves unnamed classes at their hash slug, so a candidate's own page is one
click away. The page is `scripts/templates/candidates.html`, an HTML fragment
with a `__PAYLOAD__` placeholder; restyle it there, since nothing in the Python
generates markup. It makes no external requests, so it works opened from disk or
published as an artifact.

Both are rewrites of the tools in
[LeagueToolkit/LeagueHashes](https://github.com/LeagueToolkit/LeagueHashes)
(`split_words.py` and `xguesser.cpp`); `crates/hash-guesser/src/main.rs` lists
what changed.

#### The semantic pass

Recombination cannot reach a name containing a word the corpus has never used,
however deep it searches. Judgement can: `UiMetricKills` beside an unnamed
sibling invites `UiMetricDeaths`, but nothing in the hash encodes that Kills and
Deaths belong together. `scripts/guesser_families.py` supplies the context for
that call and then checks the answer.

```bash
# which families are worth the effort, biggest first
python3 scripts/guesser_families.py rank

# context packs: base class, named siblings, fields, types, lifespan, references
python3 scripts/guesser_families.py emit

# then propose names from a pack and verify what each one hits
python3 scripts/guesser_families.py check proposals.txt --matches-out only.txt
```

`rank` is the census of unnamed families in the build the game currently ships,
scored by how much context each one carries as well as by size - the two are
close to independent, and a family of 74 with no fields on any member is a
worse target than one of 28 that describes itself. The survey built on it,
including the ten families worth taking next, is
[docs/unnamed-families.md](docs/unnamed-families.md).

`emit` writes one pack per class family under `hash-guesser-out/families/`, plus
one per class holding unresolved fields - everything a name can be judged
against except the hash itself, including the inner classes the family's field
types reach and any default the constructor set. `--subtree` groups by the whole
inheritance subtree the way `rank` counts it rather than by direct children, and
`--live` drops classes the latest build no longer has. `check` reports whether a
proposal hits an unresolved class, an unresolved field, a name the tables
already carry, or nothing, and `--matches-out` writes the hits in a form
`--only` accepts.

A proposed list is a few hundred probes, so its noise is arithmetic zero. That
does not lower the bar: a name this pass proposes is exactly the kind of
plausible that collides, and it needs attestation like any other candidate.

## Regenerating locally

Rebuilding `db/` needs nothing but Python 3 and a checkout - it reads only committed files, no
network and no dumper build:

```bash
# Folds every dump in dumps/ into db/meta.db.json and db/database.py
python3 scripts/db_build.py

# Review what moved
git diff -- db/ | cat
```

Useful flags: `--dumps`/`--hashes` to point at other input directories, `--out`/`--py` to redirect
the outputs, and `--skip-py` to leave the `database.py` snapshot alone.

To read a single dump as C++-like structs:

```bash
# The whole dump
python3 scripts/dump_meta.py dumps/16.13.7915903.json > meta.hpp

# Only the given classes and everything they reference (names or 0x hashes)
python3 scripts/dump_meta.py dumps/16.13.7915903.json GameEntityPrefab > entity.hpp
```

Producing a new dump is the expensive path and needs the Rust toolchain plus, on Linux, `libc++1`,
`build-essential` and `cmake`:

```bash
cargo build --release --bin dumper
cargo run --release --bin meta-sync
```

`meta-sync` skips versions already present in `dumps/` and everything at or below the legacy cutoff
(`13.14.5227601`), whose meta format the dumper cannot read. When the dumper binary is not at the
default path - cross-compilation, or CI's `--target` subdirectory - point `DUMPER_PATH` at it.

## Automation

Five workflows keep the repo current:

- **Sync LoL Meta Classes** (`sync-on-manifest-update.yml`) - runs on a `manifest-updated`
  repository dispatch from [Morilli/riot-manifests](https://github.com/Morilli/riot-manifests), plus
  a weekly fallback and manual dispatch. It
  discovers new versions, dumps them, regenerates `db/`, and commits the result.
- **Manual LoL Meta Sync** (`manual-sync.yml`) - the same pipeline on demand, narrowed to one
  version or region.
- **Update Hashtables** (`update-hashes.yml`) - refreshes the upstream mirror every Monday and
  **opens a PR rather than committing**. A rename cascades through class and property names across
  the whole database history and into wiki URLs, so a human reads that diff.
- **Verify Generated DB** (`verify-generated.yml`) - on any PR touching `dumps/`, `hashes/` or the
  build scripts, rebuilds `db/` and fails if the committed output does not match. Make it a
  required check in branch protection for the guarantee to hold.
- **Notify Wiki** (`notify-wiki.yml`) - tells the wiki to redeploy when `db/meta.db.json` changes,
  instead of it waiting on its own weekly cron.

## Tooling

The Rust workspace under `crates/`:

- **dumper** - extracts the metaclass vector out of a macOS League binary using binary regex
  patterns, and writes it as JSON.
- **meta-sync** - the orchestrator: discovers versions from `Morilli/riot-manifests`, pulls the build
  over the Riot CDN, extracts the macOS binary, runs the dumper, writes `dumps/{version}.json`.
  Its `download-binary` bin fetches one build's macOS binary on its own, for the dumper or for a
  disassembler:

  ```bash
  cargo run --release --bin download-binary -- 16.1.7374870          # live EUW1 by default
  cargo run --release --bin download-binary -- --region PBE1 --latest
  cargo run --release --bin download-binary -- --region PBE1 --list -n 10
  ```

  `--resolve` prints the version `--latest` would pick and exits, which is how `dump-version.yml`
  names its output before doing any work. Listing goes through the git trees API, not the contents
  API: contents truncates a directory at 1000 entries, and PBE1 is past that, so it reported nothing
  newer than 14.24 while the region was on 16.17. Set `GITHUB_TOKEN` to lift the API rate limit.
- **rman** - RMAN (Riot manifest) parsing and chunk downloading.
- **lol-meta-schema** - the shared serde types for a dump file.
- **hash-guesser** - guesses names for unresolved hashes by recombining words
  taken from names already known. Reads its targets from `dumps/`; see
  [Cracking unresolved hashes](#cracking-unresolved-hashes).

```bash
cargo build --release
cargo test -p meta-sync
```

## Contributing

Contributions are welcome - see the org-wide
[contributing guide](https://github.com/LeagueToolkit/.github/blob/main/CONTRIBUTING.md).

The one repo-specific rule: if a PR changes `dumps/` or `hashes/`, it must carry the regenerated
`db/` alongside. Run `python3 scripts/db_build.py` and commit the result, or Verify Generated DB
fails.

## Acknowledgments

The original meta dumper is [moonshadow](https://github.com/moonshadow565)'s work, and none of this
exists without it:

- [lolmetadumper3](https://github.com/moonshadow565/lolmetadumper3)
- [lolmetadumper2](https://github.com/moonshadow565/lolmetadumper2)
- [LeagueToolkit/LeagueHashes](https://github.com/LeagueToolkit/LeagueHashes)

This repository builds on that dumper and adds the versioned database, the hashtable maintenance
layer, and the automation that keeps both current.

- [CommunityDragon](https://communitydragon.org/) for maintaining the hashtables and tracking game
  files.
- [Morilli](https://github.com/Morilli) for `riot-manifests`, which is how new builds are
  discovered.

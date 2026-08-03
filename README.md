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
docs/           format specs for both database files
```

`dumps/` is the source of truth. Everything under `db/` is generated from it and is rebuilt from
scratch on every run, so never hand-edit those files.

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

Recasing is always safe - the bin-hash is FNV-1a over the *lowercased* name, so
`abilityHaste` and `AbilityHaste` are one hash. Separators are not, since they
are hashed like any other byte, so a name containing one cannot be normalized
and is rejected instead of rewritten.

The rule binds what we author. `hashes/hashes.*.txt` is upstream's file byte for
byte and stays as served, so the two layers can disagree about spelling - and an
override that differs from upstream only in casing is kept, not pruned: it is
the only thing making the database say `MapSSAO` where upstream says `MapSsao`.

### Cracking unresolved hashes

A hash with no name is a raw `0x...` in the database. Two tools try to fix that,
and they are one pipeline: split the names we know into words, then recombine
those words against the hashes we are missing.

```bash
# 1. the vocabulary, frequency-ordered, from every name the repo knows
python3 scripts/split_words.py hashes/hashes.*.txt hashes/overrides/*.txt > words.txt

# 2. substitute and insert each word at each position of each known name
cargo run --release -p hash-guesser -- binfields --words words.txt -o hits.txt

# 3. exhaustive instead, and exponential: cost is len(wordlist)^depth
cargo run --release -p hash-guesser -- bintypes --words words.txt \
    --mode force --depth 2
```

(`.cargo/config.toml` pins the workspace to `x86_64-unknown-linux-gnu` for the
dumper's C++ stubs, so on Windows add `--target x86_64-pc-windows-msvc` - the
guesser is pure Rust and builds anywhere.)

`hash-guesser` takes its target set straight from `dumps/` - every class and
property hash the game has used across every committed build, minus everything
already named - so there is no `all.*.txt` to keep up to date. Output is
`{hash} {name}`, PascalCase by construction and checked before printing, which
is what makes it feedable into `hashtool add`.

**Output is candidates, not cracks.** A 32-bit hash collides with plausible
names by chance, so nothing from here belongs in an override table until it has
been confirmed against shipped data. Record what confirmed it in the batch note
(`add -b <batch> -e "..."`).

Both are rewrites of the tools in
[LeagueToolkit/LeagueHashes](https://github.com/LeagueToolkit/LeagueHashes)
(`split_words.py` and `xguesser.cpp`); `crates/hash-guesser/src/main.rs` lists
what changed.

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

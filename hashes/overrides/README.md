# Local overrides

Names for hashes that upstream (see `../sources.toml`) does not have. Same
`hash name` line format as the tables themselves; an override wins on hash
collision.

These are **not** baked into the vendored `../hashes.<table>.txt` mirror - that
file is an exact copy of upstream. Overrides are layered over it at the point of
use, by `read_resolved_hashes` in `scripts/db_import.py`, so the mirror and this
local layer stay independently reviewable (`git diff ../hashes.*.txt` is pure
upstream drift; `git diff .` is your cracks).

Upstream is canonical in intent: everything here should also be submitted to
CDTB so the override can eventually be deleted - except what the ledger marks
`local` (see below), which stays here permanently by choice.
`update_hashes.py` warns when an entry has become redundant (upstream now
carries the identical name).

These are cracks upstream doesn't have yet. Entries awaiting an upstream merge
can be carried here too, so the names resolve now instead of after the PR lands;
once upstream ships the identical name the entry is redundant, and
`update_hashes.py` flags it. Remove flagged entries in one pass with
`python3 scripts/hashtool.py prune`.

(The Game Entity component batch from
[CommunityDragon/Data#35](https://github.com/CommunityDragon/Data/pull/35) - 136
entries - merged upstream and was pruned on 2026-07-24.)

## ledger.tsv

`{table}.txt` says what a hash resolves to and nothing else - not when it was
cracked, not what attests it, not whether it has been sent upstream. That gap is
how a crack sits here for months and is quietly forgotten, so the history lives
beside it in `ledger.tsv`: one row per crack, keyed by **(table, hash)** because
six names are both a class and a field and so share a hash.

    ledger.tsv    hash  table  name  batch  cracked  status  pr
    batches.tsv   batch  note

Both are split-out tables with no comments, no blank lines, a header on line 1
and a uniform column count, which is what GitHub's table viewer needs to render
them - `load()` enforces it. That is why the batch notes are a second file
rather than a comment block at the top of the first.

`status` is `pending` (cracked, not sent anywhere), `submitted` (in an open
upstream PR - put the link in `pr`), `merged` (upstream has it; `prune` will drop
the override, the row stays as history), or `local` (deliberately not going
upstream - see below).

`batch` is the campaign a crack came out of, and the unit an upstream PR is built
from. Rows are written grouped by it - as row order, not as a heading, since the
`batch` column already carries it - and the method and attestation live once per
batch in `batches.tsv` rather than repeated on every row; anything per-name
belongs in the reversing doc that note points at. Today:
`pre-ledger-backlog` (945, blame-dated, method not recoverable),
`vfx-driver-graph` (190), `season16-modes-ui` (96), `game-entity-sweep` (55). A
crack added without `-b` lands in `unsorted`.

    python3 scripts/hashtool.py add bintypes NewName -b my-campaign -e "method"
    python3 scripts/hashtool.py ledger                    # per-batch summary
    python3 scripts/hashtool.py ledger --list pending     # grouped by batch
    python3 scripts/hashtool.py ledger --batch vfx-driver-graph --status submitted --pr 42
    python3 scripts/hashtool.py ledger --reconcile        # sync against upstream
    python3 scripts/hashtool.py ledger --set 51e9b98e --status submitted --pr 42
    python3 scripts/hashtool.py ledger --match 'Monarch*' --status local

`--batch` and `--match` are the bulk selectors: `--batch` takes a whole campaign
(the unit a PR is built from), `--match` takes a name glob across campaigns, for
a family cracked over several sittings. They combine to narrow.

`--seed` records overrides that have no row yet, taking both the date and the
batch from the commit that introduced the line - the only attribution
recoverable after the fact.

`--reconcile` is the one that keeps this honest: it flips a row to `merged` when
the upstream mirror starts serving the identical name, and warns about rows with
no override and overrides with no row. Run it after `update_hashes.py`, before
`prune`. A `local` row it finds upstream is flipped too, but loudly: the name is
public whatever we intended, and the row was claiming otherwise.

### Local-only cracks

A crack we resolve here but do not want in CDragon gets `status = local`. It
still resolves - the override layer doesn't care about status - it just drops out
of `--list pending`, which is the "what still needs an upstream PR" view. This is
an exit from the submission lifecycle, not a stage in it, and the intent is
permanent; flip it back to `pending` if that changes.

    python3 scripts/hashtool.py ledger --match 'Monarch*' --status local
    python3 scripts/hashtool.py add bintypes NewSecret -b some-batch -l

The reason a name is held back is the part worth writing down - a bare `local`
with no explanation reads as a forgotten crack a year later. When a whole batch
is held back the reason goes in its `batches.tsv` note; when the hold-back cuts
across batches, as a name family cracked over several sittings does, record it
here:

- **`*Monarch*`** (58 rows, `pre-ledger-backlog` + `season16-modes-ui`) -
  the class vocabulary of an unshipped game mode. Nothing about it is public
  yet, and CDragon's tables are, so publishing the names there would leak the
  mode ahead of Riot. Kept for the wiki, not sent upstream. Revisit once the
  mode ships.

The other reason a crack can be unfit for upstream is that it isn't *proven* -
that's a different thing and stays `pending` under a batch whose note says so
(`meta-delta-16.14-unproven`). `local` means "solid, but ours"; an unproven name
means "not established enough to submit yet".

Nothing is pruned for being unused. A name we cracked costs a line and may
resolve something in a future dump, so entries stay even when no current dump
references the hash. Names are recorded exactly as they were cracked - note this
means the binfields entries keep the older PascalCase convention while upstream
now serves camelCase.

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
CDTB so the override can eventually be deleted. `update_hashes.py` warns when an
entry has become redundant (upstream now carries the identical name).

These are cracks upstream doesn't have yet. Entries awaiting an upstream merge
can be carried here too, so the names resolve now instead of after the PR lands;
once upstream ships the identical name the entry is redundant, and
`update_hashes.py` flags it. Remove flagged entries in one pass with
`python3 scripts/hashtool.py prune`.

(The Game Entity component batch from
[CommunityDragon/Data#35](https://github.com/CommunityDragon/Data/pull/35) - 136
entries - merged upstream and was pruned on 2026-07-24.)

Nothing is pruned for being unused. A name we cracked costs a line and may
resolve something in a future dump, so entries stay even when no current dump
references the hash. Names are recorded exactly as they were cracked - note this
means the binfields entries keep the older PascalCase convention while upstream
now serves camelCase.

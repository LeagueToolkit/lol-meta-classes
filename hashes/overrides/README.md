# Local overrides

Names for hashes that upstream (see `../sources.toml`) does not have, layered on
top of the fetched tables by `scripts/update_hashes.py`. Same `hash name` line
format as the tables themselves; an override wins on hash collision.

Upstream is canonical in intent: everything here should also be submitted to
CDTB so the override can eventually be deleted. `update_hashes.py` warns when an
entry has become redundant (upstream now carries the identical name).

Two kinds of entry live here:

- **Cracks upstream never had** - the bulk of the file.
- **Cracks awaiting an upstream merge** - carried locally so the names resolve
  now instead of after the PR lands. 135 entries (96 bintypes, 39 binfields)
  come from [CommunityDragon/Data#35](https://github.com/CommunityDragon/Data/pull/35),
  the Game Entity component system hashes. When that merges, `update_hashes.py`
  will flag them as redundant and they can be deleted in one pass.

Nothing is pruned for being unused. A name we cracked costs a line and may
resolve something in a future dump, so entries stay even when no current dump
references the hash. Names are recorded verbatim as they were cracked - note
this means the binfields entries keep the older PascalCase convention while
upstream now serves camelCase.

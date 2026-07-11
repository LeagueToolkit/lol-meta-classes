## Versioned database format (db/meta.db.json)

### Purpose
- **What it is**: The machine-readable source of truth for the LoL meta schema *across all game versions*. For every class and property it records which builds it existed in, and every distinct definition it ever had (type changes, base changes) as an ordered revision history.
- **What it solves**: `db/database.py` used to be an ever-growing aggregate of everything that ever existed — it couldn't tell you which definition is current, why a property appeared twice (type change), or that a class had been removed from the game. `meta.db.json` answers all of those directly.
- **Relationship to `db/database.py`**: `database.py` is now a human-diffable snapshot of the **latest build only**, generated from the same script. Consumers that need history should read `meta.db.json`.

### How it's generated
```bash
python3 scripts/db_build.py
```
- Folds **all** of `dumps/*.json` in build-number order into the interval model, resolves names from `hashes/`, and writes `db/meta.db.json` plus the `db/database.py` snapshot.
- The build is fully deterministic and stateless: the file is rebuilt from scratch from the dumps every run, so there is no incremental state that can drift or corrupt.
- CI: the `Sync LoL Meta Classes` workflow runs this whenever `dumps/` changes.

### Top-level structure
```jsonc
{
  "formatVersion": 1,
  "latest": 7915903,                 // build number of the newest dump
  "versions": [                      // every dump folded in, in order
    {"patch": "13.15", "build": 5229820},
    // ...
    {"patch": "16.13", "build": 7915903}
  ],
  "externalTypeNames": {             // names for hashes referenced in bases /
    "0x5566d3a3": "TagCollection"    // type tuples but never dumped as classes
  },
  "classes": { "<hex hash>": <Class>, ... }
}
```
- Builds are the unit of time everywhere. They are globally unique and (almost always) monotonic, unlike patch strings, which don't sort lexicographically (`13.2` vs `13.15`). Use `versions` to map a build to its patch for display.

### Class entry
```jsonc
"0x1003c990": {
  "name": "SomeResolvedName",        // omitted when the hash is uncracked
  "revisions": [
    {"from": 5229820, "to": 6442327, "bases": ["0x91873e8a"], "interface": false, "value": false},
    {"from": 6478644, "bases": ["0x91873e8a", "0xb0607142"], "interface": false, "value": false}
  ],
  "properties": { "<hex hash>": <Property>, ... }
}
```

### Property entry
```jsonc
"0xf0a363e3": {
  "name": "ColorblindTexturePath",
  "revisions": [
    {"from": 5229820, "to": 6442327, "type": ["Option", "0x0", "String", "0x0"], "default": null},
    {"from": 6478644, "type": ["String", "0x0", "0x0", "0x0"], "default": ""}
  ]
}
```
- `type` is the same 4-tuple as `database.py` fields — `(ft, kt, vt, kh)` — see `docs/database.md`. `kh` stays a raw hash here; resolve it via `classes[kh].name` or `externalTypeNames`.
- `default` is the most recent default value observed within that revision's range (revisions are keyed on the type tuple, so a default-only tweak updates the open revision in place rather than opening a new one).

### Revision semantics
- A revision is one distinct definition plus the build range it was observed in: `from` = first build seen, `to` = last build seen.
- **`to` is omitted on the open (current) revision** — i.e. the definition present in `latest`. This is deliberate: when a new build changes nothing, unchanged entities stay byte-identical, so the git diff for a quiet patch is a handful of lines.
- Derived answers:
  - **Current definition** — the last revision, iff it has no `to`.
  - **Removed from the game** — the last revision has a `to`. It was last seen in build `to`.
  - **Type/inheritance change** — adjacent revisions. Removed-then-re-added shows up as revisions with a gap between them.
- A new revision starts when either the definition changes **or** the entity was absent from the previous build (so a genuine remove + re-add is never masked, even if the definition matches).

### Identity and names
- Everything is keyed by FNV-1a hash, never by resolved name. Names from `hashes/` are attached as `name` metadata. This means a hash getting cracked later improves display names without ever creating false history (a rename is not a remove + add).

### File layout / diffing
- The writer is line-oriented on purpose: one line per property, one line per version entry, one line per class header. A type change in one property is a one-line diff; a new quiet build is ~5 lines (`latest` + `versions`).

### Extending the format
- Revisions, class entries, and property entries are open objects: new fields (e.g. `offset`, `size`, `alignment`, flags from the raw dumps) can be added additively without breaking consumers. Consumers must ignore unknown keys.
- `formatVersion` is bumped only for breaking changes (removed/renamed fields, changed semantics).
- Anything not captured here is still recoverable: `dumps/` keeps the full raw per-build data, and the whole file is a pure function of it.

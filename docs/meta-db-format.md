## Versioned database format (db/meta.db.json)

### Purpose
- **What it is**: The machine-readable source of truth for the LoL meta schema *across all game versions*. For every class and property it records which builds it existed in, and every distinct definition it ever had (type changes, base changes) as an ordered revision history.
- **What it solves**: `db/database.py` used to be an ever-growing aggregate of everything that ever existed - it couldn't tell you which definition is current, why a property appeared twice (type change), or that a class had been removed from the game. `meta.db.json` answers all of those directly.
- **Relationship to `db/database.py`**: `database.py` is now a human-diffable snapshot of the **latest build only**, generated from the same script. Consumers that need history should read `meta.db.json`.

### How it's generated
```bash
python3 scripts/db_build.py
```
- Folds **all** of `dumps/*.json` in build-number order into the interval model, resolves names from `hashes/`, and writes `db/meta.db.json` plus the `db/database.py` snapshot.
- The listing of `dumps/` is not recursive. The dump in `dumps/pbe/` is not part of the live history and only feeds `db/meta.pbe.json`, which the same run writes (see "PBE preview"). `--no-preview` skips that file, `--preview-dumps <dir>` reads the PBE dump from another directory, and `--preview-out <path>` writes the overlay elsewhere.
- The build is fully deterministic and stateless: the file is rebuilt from scratch from the dumps every run, so there is no incremental state that can drift or corrupt.
- CI: the `Sync LoL Meta Classes` workflow runs this whenever `dumps/` changes.

### Top-level structure
```jsonc
{
  "formatVersion": 1,
  "hashSource": {                    // optional: which name snapshot the `name`
    "fetchedAt": "2026-07-20T00:00:00Z",   // fields below were resolved against.
    "bintypes": {"url": "https://raw.communitydragon.org/data/hashes/lol/hashes.bintypes.txt", "entries": 3593},
    "binfields": {"url": "…", "entries": 10361}
  },
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
- `hashSource` mirrors `hashes/provenance.json` and is **omitted entirely** when that file is absent (a pre-refresh checkout still builds). Consumers should treat it as optional. It lets a downstream API state exactly which name snapshot the dataset was resolved against.
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
- `type` is the same 4-tuple as `database.py` fields - `(ft, kt, vt, kh)` - see `docs/database.md`. `kh` stays a raw hash here; resolve it via `classes[kh].name` or `externalTypeNames`.
- `default` is the most recent default value observed within that revision's range (revisions are keyed on the type tuple and the hasher, so a default-only tweak updates the open revision in place rather than opening a new one).

### Hasher of a `Hash` property
The `Hash` type gives neither the width of the stored value nor the hash function. Both belong to the hasher of the property. A revision carries `hasher` if the hasher is not the default:
```jsonc
"0x8d39bde6": {
  "name": "name",
  "revisions": [
    {"from": 5229820, "to": 8230722, "type": ["String", "0x0", "0x0", "0x0"], "default": ""},
    {"from": 8255794, "type": ["Hash", "0x0", "0x0", "0x0"], "hasher": {"width": 8, "algorithm": "Xxh3", "lowercased": true}, "default": "0x0"}
  ]
}
```
- `width`: bytes of the stored value, 4 or 8. A bin reader consumes `width` bytes for the value.
- `algorithm`: `Fnv1a32`, `Xxh64`, `Xxh3` (XXH3-64) or `Unknown`. A 64-bit algorithm with `width` 4 stores the low 32 bits.
- `lowercased`: `true` if the string is lowercased before it is hashed.
- A revision without `hasher` has the default: FNV-1a 32 over the lowercased string, stored in 4 bytes.
- A change of `width`, `algorithm` or `lowercased` starts a new revision.
- The values come from the `hashers` table of the dump (dump format version 3). A dump before 16.17.8104348 records no hasher and reads as the default.
- A `Hash` element of a `List`, `Option` or `Map` has no hasher in the dump and gets no `hasher` here.

### Revision semantics
- A revision is one distinct definition plus the build range it was observed in: `from` = first build seen, `to` = last build seen.
- **`to` is omitted on the open (current) revision** - i.e. the definition present in `latest`. This is deliberate: when a new build changes nothing, unchanged entities stay byte-identical, so the git diff for a quiet patch is a handful of lines.
- Derived answers:
  - **Current definition** - the last revision, iff it has no `to`.
  - **Removed from the game** - the last revision has a `to`. It was last seen in build `to`.
  - **Type/inheritance change** - adjacent revisions. Removed-then-re-added shows up as revisions with a gap between them.
- A new revision starts when either the definition changes **or** the entity was absent from the previous build (so a genuine remove + re-add is never masked, even if the definition matches).

### PBE preview (db/meta.pbe.json)
`db/meta.pbe.json` describes the newest PBE build as an overlay on the live database. It is the only place where a PBE build appears: `db/meta.db.json` and `db/database.py` are live only, and a PBE build never changes them.

```jsonc
{
  "formatVersion": 1,
  "channel": "pbe",
  "patch": "16.21",                  // null if no preview exists
  "build": 8255794,                  // null if no preview exists
  "base": 8230722,                   // the live build that the PBE build follows; null if no preview exists
  "externalTypeNames": {},           // names that `externalTypeNames` of meta.db.json lacks
  "classes": { "<hex hash>": <Class>, ... }
}
```
- **How it's built**: `db_build.py` folds the dump in `dumps/pbe/` as one more build on top of the live history, as if the PBE build followed `base`. `classes` holds every class entry that differs from the live entry after that fold, in the Class entry schema above. An entry is complete: it carries the full revision history of the class and of every property, not only the PBE revisions.
- **No preview**: the file always exists. If `dumps/pbe/` holds no dump, or the patch of the dump is not greater than the latest live patch, `patch`, `build` and `base` are `null` and `classes` is empty. Check `build` before reading anything else.
- **Pairing with meta.db.json**: `base` equals `latest` of the `meta.db.json` in the same commit. A consumer that fetches the two files separately must check that equality and treat a mismatch as "no preview": the two files then come from different commits.
- **Merged view**: a consumer that wants the database as of the PBE build applies four operations to a copy of the `meta.db.json` document:
  1. append `{"patch": patch, "build": build}` to `versions`
  2. set `latest` to `build`
  3. spread the overlay's `classes` over `classes`
  4. spread the overlay's `externalTypeNames` over `externalTypeNames`

  The result equals the database that `db_build.py` writes if the PBE dump is the last live dump. All revision semantics below apply to it unchanged.
- **Reading an entry** (after the merge, or directly from the overlay's `classes`):
  - A class or property that PBE adds has a last revision with `from` equal to `build` and no `to`.
  - A class or property that PBE removes has a last revision with `to` equal to `base`.
  - A definition that PBE changes has a revision that ends at `base`, followed by a revision that starts at `build`.
  - A property whose type is unchanged and whose default differs has the PBE default on its open revision.
- **Ordering is by channel, not by build number.** PBE and live builds come from separate branches, and their build numbers interleave: a live hotfix build can be greater than the overlay's `build`. Do not sort it into `versions` by number. It always follows `base`.
- **Lifetime**: the repository keeps one PBE dump, the newest. Each new PBE build replaces the overlay, so the overlay has no history of its own. A PBE build is not promoted as is: the live build of the same patch has a different build number and arrives as a normal entry in `versions`, at which point the overlay becomes empty or moves to the next patch.
- **Why a separate file**: a PBE build arrives on most days. In its own file it does not add a commit to the history of `meta.db.json`, and a consumer chooses whether a PBE build alone is worth a rebuild (see the `meta-pbe-updated` event of the Notify Wiki workflow).

### Identity and names
- Everything is keyed by FNV-1a hash, never by resolved name. Names from `hashes/` are attached as `name` metadata. This means a hash getting cracked later improves display names without ever creating false history (a rename is not a remove + add).

### File layout / diffing
- The writer is line-oriented on purpose: one line per property, one line per version entry, one line per class header. A type change in one property is a one-line diff; a new quiet build is ~5 lines (`latest` + `versions`).

### Extending the format
- Revisions, class entries, and property entries are open objects: new fields (e.g. `offset`, `size`, `alignment`, flags from the raw dumps) can be added additively without breaking consumers. Consumers must ignore unknown keys.
- `formatVersion` is bumped only for breaking changes (removed/renamed fields, changed semantics).
- Anything not captured here is still recoverable: `dumps/` keeps the full raw per-build data, and the whole file is a pure function of it.

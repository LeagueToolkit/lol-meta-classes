## Database file format (db/database.py)

### Purpose
- **What it is**: A stable, diff-friendly, Python-like text representation of the LoL meta schema (classes, inheritance, and properties) **as of the latest build only**. For version history (removed classes/properties, type changes, when things were added), use `db/meta.db.json` - see `docs/meta-db-format.md`.
- **What it is not**: Valid/executable Python. It is intentionally a simple text format that looks like Python so it’s easy to read and diff in Git.

### How it’s generated
- Via `scripts/db_build.py`, which:
  - Loads the latest dump from `dumps/*.json` and normalizes old/new meta formats.
  - Resolves known type and field hashes using the upstream mirror (`hashes/hashes.bintypes.txt`, `hashes/hashes.binfields.txt`) with `hashes/overrides/` layered on top at read time (`read_resolved_hashes`).
  - Writes `db/database.py` deterministically (alongside the versioned `db/meta.db.json`).
- CI: The `Sync LoL Meta Classes` workflow regenerates and commits both files when `dumps/` changes.

### File structure
- Header line: `#!python` (for editor hinting only).
- For each class:
  - Class line: `class <TypeName>(<Base1, Base2, ...>):`
    - `<TypeName>` is the resolved type name if known, otherwise the raw hex hash (e.g. `0xe75aad84`).
    - Base list contains the resolved primary base and any secondary bases (if any). - inheritance
  - Field lines (indented 4 spaces):
    - `FieldName: (ft, kt, vt, kh)` optionally followed by ` = <json-default>`
      - **FieldName**: resolved field name if known, otherwise the raw hex hash.
      - **ft** (field type): one of scalar or composite types, e.g. `Bool`, `I32`, `U32`, `F32`, `String`, `Hash`, `File`, `Flag`, or composite types `List`, `List2`, `Pointer`, `Embed`, `Link`, `Option`, `Map`.
      - **kt** and **vt** (auxiliary type parameters):
        - For `List`/`List2`/`Option` (container): `kt` is the fixed-size as hex (e.g. `0x0` if dynamic); `vt` is the container value type (e.g. `U32`).
        - For `Map`: `kt` is the map key type (e.g. `String`); `vt` is the map value type (e.g. `U32`).
        - For scalars and non-container composites: both are `0x0`.
      - **kh** (other-class/type reference):
        - For `Pointer`/`Embed`/`Link` and some composites, this is the referenced class. It is resolved to a known type name when available; otherwise it remains the raw hex hash. For non-referential types it is `0x0`.
      - **Default value (optional)**:
        - If a property has a default in the dump, it is appended using ` = ` followed by a compact JSON literal (e.g. `null`, `false`, `0`, `"text"`, `[]`, `{}` or nested objects). Keys are written with stable ordering.
  - Terminator: `pass`

Example:
```text
class ExampleClass(BaseType):
    ExampleList: (List2, 0x0, U32, 0x0) = []
    PointerToOther: (Pointer, 0x0, 0x0, OtherType) = null
    NameToCount: (Map, String, U32, 0x0) = {}
    ScalarValue: (I32, 0x0, 0x0, 0x0) = 0
    pass
```

### Name resolution and unknowns
- Type and field hashes are mapped using the files in `hashes/`.
- If a hash is unknown, it is left as its raw hex form in the output (no prefixing). This applies to both class names and field names, and to the `kh` referenced type.

#### Where the names come from
- **Source of truth**: [CommunityDragon's](https://raw.communitydragon.org/data/hashes/lol/) hashtables (CDTB). The exact URLs live in `hashes/sources.toml`; `url` wins, `repo` + `path` (+ optional `ref`) is shorthand for a raw.githubusercontent.com URL.
- **Refresh**: `python3 scripts/update_hashes.py`. It fetches each table, validates it, and writes `hashes/hashes.<table>.txt` as an exact mirror of upstream (LF, sorted by name), then records `hashes/provenance.json`. It does **not** bake overrides into the mirror - those are layered at build time (see below). The tables stay committed so builds are reproducible and offline-capable - this is a deliberate, reviewed operation, not a build-time download.
- **Drift**: the script does not compute one. The tables are committed, so `git diff -- hashes/` after a run *is* the drift, and the `db/database.py` diff after `db_build.py` shows what it changed downstream.
- **Overrides** (`hashes/overrides/<table>.txt`, same `hash name` line format): genuinely-local cracks that upstream doesn't have. They are layered over the upstream mirror by `read_resolved_hashes` when a build reads names - never baked into `hashes/hashes.*.txt` - so the two files diff independently. On conflict the override wins, but upstream is canonical in intent - anything kept here should also be submitted to CDTB so the override can eventually be deleted. `update_hashes.py` warns when an override has become redundant (upstream now serves the identical name).
- **Refreshes land as reviewed PRs**, never silent commits: a rename cascades into class/property names across the entire `db/meta.db.json` history and into downstream wiki URLs. The `Update Hashtables` workflow opens that PR weekly.

### Ordering
- Classes are written in alphabetical order by their display name (resolved type name if known, otherwise the raw hex). Ties are broken by class hash to ensure stability.
- Each class’s `bases` are sorted before printing.
- Each class’s `fields` are sorted before printing. Sorting uses `(FieldName, (ft, kt, vt, kh))` only; any default value does not affect ordering.

### Snapshot semantics
- The file is a pure snapshot of the newest dump: every class/property in it exists in the latest build, and nothing else. Removed entities and historical type variants live in `db/meta.db.json` instead.
- (Historical note: `db_import.py` used to merge dumps *into* the existing file, which made it an aggregate of everything that ever existed - with duplicate fields on type changes and no way to spot removals. `db_build.py` replaced that flow.)

### Regeneration (manual)
```bash
python3 scripts/db_build.py
git diff -- db/database.py | cat
```

### Why this format
- Readable, compact, and Git-diff–friendly.
- Strictly line-oriented and regex-parseable (`db_import.py` uses regex to read it back), so it’s easy to review and stable across runs.


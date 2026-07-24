# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust toolchain for extracting and tracking League of Legends metaclass definitions across game versions. It maintains a Git-driven database of LoL's internal class structures.

## Build Commands

```bash
# Build everything (release)
cargo build --release

# Build individual binaries
cargo build --release --bin dumper
cargo build --release --bin meta-sync

# Run meta-sync (requires dumper to be built first)
cargo run --release --bin meta-sync

# Run tests
cargo test -p meta-sync
```

**System dependencies** (Linux): `libc++1`, `build-essential`, `cmake`

## Architecture

### Crates

- **dumper** (`crates/dumper/`): Binary analysis tool that extracts metaclass information from macOS LoL executables. Uses binary regex patterns to locate metaclass vectors and outputs JSON.

- **meta-sync** (`crates/meta-sync/`): Orchestrator that discovers versions from GitHub (`Morilli/riot-manifests`), downloads binaries via Riot CDN, runs dumper, and saves results to `dumps/`.

- **rman** (`crates/rman/`): Library for parsing RMAN (Riot Manifest) files and downloading game binaries.

### Workflow

```
GitHub API → Version Discovery → RMAN Download → Binary Extraction → Dumper → JSON Output
```

1. Fetch version list from `Morilli/riot-manifests` GitHub repo
2. Download RMAN manifest for each new version
3. Extract macOS binary (`LeagueofLegends.app/Contents/MacOS/LeagueofLegends`)
4. Run dumper to extract metaclasses
5. Save to `dumps/{version}.json`

### Key Configuration

Located in `crates/meta-sync/src/config.rs`:
- **CDN_URL**: Riot CDN for game files
- **LEGACY_CUTOFF**: `13.14.5227601` - versions at or below use incompatible format
- **DUMPER_PATH**: Environment variable to override dumper location

### CI Notes

The CI builds with `--target x86_64-unknown-linux-gnu`, placing binaries in `target/x86_64-unknown-linux-gnu/release/`. The `DUMPER_PATH` env var must be set when running meta-sync in CI.

## Data Files

- `dumps/`: JSON metaclass dumps per version (the raw source of truth)
- `db/meta.db.json`: Generated, versioned history of every class/property across all dumps (see `docs/meta-db-format.md`)
- `db/database.py`: Generated, diff-friendly snapshot of the latest build only
- `hashes/hashes.bintypes.txt` / `hashes/hashes.binfields.txt`: exact mirrors of the upstream CDragon tables (type/field hash → name). Only `update_hashes.py` writes them.
- `hashes/overrides/{bintypes,binfields}.txt`: local cracks upstream doesn't have. **Not** baked into the mirror above - layered on top at build time by `read_resolved_hashes` (db_import.py), so the two diff independently.

## Scripts

```bash
# Rebuild db/meta.db.json + db/database.py from all dumps
# (reads the upstream mirror + overrides; fully offline)
python3 scripts/db_build.py

# Convert dump to C++-like structs
python3 scripts/dump_meta.py dumps/<version>.json > /tmp/meta.hpp

# Refresh the upstream mirror from CDragon (network; opens a reviewed PR in CI)
python3 scripts/update_hashes.py

# Hash-table maintenance: fnv / add / sort / lookup / check
python3 scripts/hashtool.py add bintypes SomeCrackedClassName
python3 scripts/hashtool.py check names.txt
```

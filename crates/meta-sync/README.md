# meta-sync

Automated synchronization tool for League of Legends metaclass information across game versions.

## Overview

`meta-sync` automatically discovers, downloads, and processes League of Legends game binaries to extract metaclass definitions. It tracks versions, handles manifest parsing, and coordinates with the `dumper` tool to produce structured JSON output.

## Quick Start

```bash
# Build both meta-sync and dumper
cargo build --release --bin meta-sync
cargo build --release --bin dumper

# Run meta-sync
cargo run --release --bin meta-sync

# Run the PBE pass
cargo run --release --bin meta-sync -- --channel pbe
```

## What It Does

1. **Version Discovery**: Queries GitHub (`Morilli/riot-manifests`) for available LoL versions
2. **Manifest Processing**: Downloads and parses RMAN (Riot Manifest) files
3. **Binary Extraction**: Downloads the macOS League of Legends binary
4. **Metaclass Extraction**: Runs the `dumper` tool to extract class definitions
5. **Output**: Saves structured JSON to `dumps/{version}.json`

## Output Format

Each version produces a JSON file like:

```json
{
  "version": "15.1.123456",
  "classes": [
    {
      "hash": "0x12345678",
      "name": "Champion",
      "bases": ["0xabcdef12"],
      "fields": [
        {
          "hash": "0x87654321",
          "name": "health",
          "type": "F32"
        }
      ]
    }
  ]
}
```

## Configuration

The tool uses sensible defaults but can be customized through `config.rs`:

- **CDN_URL**: Riot CDN for downloading game files
- **GITHUB_OWNER/REPO**: Source repository for version manifests
- **MANIFEST_PATH**: Path to version manifests in the repo
- **TARGET_BINARY**: Which binary to extract from manifests
- **LEGACY_CUTOFF**: Oldest version to process (13.14.5227601)

## Version Processing

### Cutoff Logic

Versions ≤ 13.14.5227601 use a legacy metaclass format and are skipped. The tool processes versions from newest to oldest, stopping at the cutoff.

### Caching

The tool checks if `dumps/{version}.json` exists before processing:

- ✅ Exists: Skip version (already processed)
- ❌ Missing: Process version

This makes the tool idempotent and safe to re-run.

## PBE Pass

`--channel pbe` replaces the live pass with the PBE pass (`preview.rs`). The pass keeps the newest PBE1 build as the only dump in `dumps/pbe/`, which `scripts/db_build.py` turns into the `preview` key of `db/meta.db.json`.

1. Reads the latest live patch from the file names in `dumps/`.
2. Removes every dump in `dumps/pbe/` whose patch is not greater than the latest live patch, and every dump except the newest.
3. Asks sieve for the newest PBE1 build. The manifest archive is not used: it lags sieve by up to a day and drops superseded builds.
4. Skips the build if its patch is not greater than the latest live patch, or if `dumps/pbe/` holds that build or a newer build.
5. Dumps the build into `temp/`, checks the size and the class count, moves the dump to `dumps/pbe/{version}.json`, and removes the dump that it replaces.

If step 3 or step 5 fails, the pass exits with code 1 and `dumps/pbe/` keeps the dump that it held.

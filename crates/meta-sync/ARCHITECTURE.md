# meta-sync Architecture

## Overview

`meta-sync` is an automated tool that synchronizes League of Legends metaclass information across game versions. It fetches version manifests from GitHub, downloads game binaries, and extracts metaclass definitions.

## Workflow

```
GitHub API → Version Discovery → Manifest Download → Binary Extraction → Dumper Execution → JSON Output
```

### Step-by-step Process

1. **Version Discovery** (`github.rs`)

   - Queries `Morilli/riot-manifests` GitHub repository
   - Looks for `LoL/EUW1/macos/lol-game-client/*.txt` files
   - Each file represents a game version (e.g., `15.1.123456.txt`)
   - Sorts versions using semantic versioning
   - Filters out versions below cutoff (13.14.5227601)

2. **Manifest Processing** (`manifest.rs`)

   - Downloads version manifest URL from GitHub file content
   - Fetches RMAN (Riot Manifest) file from CDN
   - Parses RMAN to find the `LeagueofLegends` macOS binary
   - Downloads binary chunks and reconstructs the file

3. **Binary Analysis** (`dumper.rs`)

   - Executes the `dumper` tool on the downloaded binary
   - Dumper uses pattern matching to find metaclass vector in binary
   - Extracts class definitions, inheritance, and field types
   - Outputs structured JSON to `dumps/{version}.json`

4. **Output Structure**
   ```json
   {
     "version": "15.1.123456",
     "classes": [
       {
         "hash": "0x12345678",
         "name": "Champion",
         "bases": ["0xabcdef12"],
         "fields": [...]
       }
     ]
   }
   ```

### PBE pass

`meta-sync --channel pbe` runs `preview.rs` instead of the steps above.

- Version discovery goes through sieve (`sieve.rs`), which returns the build that PBE1 serves together with its manifest URL. The live pass reads the manifest archive because it needs history; the PBE pass needs only the newest build.
- The output is `dumps/pbe/{version}.json`, in reduced form: the layout fields that the database build does not read are dropped. The directory holds one dump. `scripts/db_build.py` lists `dumps/` non-recursively, so the PBE dump is not part of the live history. It feeds `db/meta.pbe.json` only.
- A PBE dump is kept only while its patch is greater than the latest live patch. PBE and live build numbers interleave, so the comparison uses the patch, not the build number.
- The dumper writes the full dump to `temp/pbe/`. The reduced dump replaces the previous PBE dump only after the size and class count checks pass.
- `--plan` stops before the download and changes no file. `--outputs` writes the result as `key=value` lines for the workflow (`outputs.rs`), so the workflow does not derive it from `git status`.

## Key Design Decisions

### Why macOS binaries?

- macOS binaries use Mach-O format which is easier to parse
- The dumper tool has mature Mach-O support
- Metaclass information is identical across platforms

### Version Cutoff

- Versions ≤ 13.14.5227601 use a legacy format
- These require different parsing logic not yet implemented
- Cutoff prevents processing incompatible versions

### Caching Strategy

- Checks if `dumps/{version}.json` exists before processing
- Skips already-processed versions
- No cleanup of old versions (manual maintenance)

### Error Recovery

- Network errors are reported but don't stop processing
- Failed versions are logged and skipped
- Temp files are cleaned up on success

## LoL Meta Classes

This repository tracks LoL meta class information across versions.

### Layout
- `dumps/`: meta dumps per version (e.g. `15.19.7151836.json`).
- `scripts/`: helper scripts for reading/printing/importing meta.
- `db/database.py`: generated, diff-friendly representation of the merged schema.
- `hashes/`: name mappings for types and fields (used during generation). Vendored
  from [CommunityDragon](https://raw.communitydragon.org/data/hashes/lol/); the
  source is declared in `hashes/sources.toml` and refreshed with
  `python3 scripts/update_hashes.py`. See [docs/database.md](docs/database.md#name-resolution-and-unknowns).
- `docs/`: documentation, including detailed database format notes.

### Database file
- The generated database is in `db/database.py`.
- It’s deterministic and optimized for Git diffs.
- See full documentation: [docs/database.md](docs/database.md)

### Regenerate database locally
```bash
python3 scripts/db_import.py db/database.py dumps/<version>.json
git diff -- db/database.py | cat
```

### CI automation
- The "Sync LoL Meta Classes" workflow updates `dumps/` when new data is available.
- After updating dumps, it regenerates `db/database.py` and commits any changes.
- Powered by the `meta-sync` tool (see `crates/meta-sync/`)

### Inspect as C++-like structs
```bash
python3 scripts/dump_meta.py dumps/<version>.json > /tmp/meta.hpp
```

### Tools
- **meta-sync**: Automated version discovery and metaclass extraction (`crates/meta-sync/`)
- **dumper**: Binary analysis tool for extracting metaclasses (`crates/dumper/`)
- **db_import.py**: Merges dumps into the database
- **dump_meta.py**: Converts dumps to C++-like format

### Notes
- Unknown hashes remain as `0x...` in outputs until mappings exist.

### Acknowledgements
The original meta dumper was created by [moonshadow](https://github.com/moonshadow565):
- [lolmetadumper3](https://github.com/moonshadow565/lolmetadumper3)
- [lolmetadumper2](https://github.com/moonshadow565/lolmetadumper2)
- [LeagueToolkit/LeagueHashes](https://github.com/LeagueToolkit/LeagueHashes)

This repository is a step up from that work and packs a ton of additions, most
notably a full devops setup for automatic dumping and syncing of the database
and hashes.

Credits:
- Huge thank you to moon for the original project - none of this would be possible otherwise.
- [CommunityDragon](https://communitydragon.org/) for maintaining their hashtables and keeping track of game files.
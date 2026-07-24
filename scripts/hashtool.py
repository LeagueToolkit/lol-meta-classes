#!/bin/env python
"""Operate on the vendored hash-name tables in hashes/.

Companion to update_hashes.py (which only fetches upstream): this is for the
hand-maintenance around hashes/overrides/ - hashing a name, adding a crack,
re-sorting a file, resolving a hash or name, and diffing an external name list
against what the repo already resolves.

The bin-hash algorithm is FNV-1a-32 of the lowercased name; the file format is
`{hash:08x} {name}` sorted by (name, hash), matching update_hashes.render().

Usage:
    python3 scripts/hashtool.py fnv GameEntityPrefab AnchorHierarchy
    python3 scripts/hashtool.py add bintypes GameEntityPrefab FooBarBaz
    python3 scripts/hashtool.py add binfields envMesh
    python3 scripts/hashtool.py sort                 # renormalize both overrides
    python3 scripts/hashtool.py lookup 2b949af2      # hash -> name (any table)
    python3 scripts/hashtool.py lookup GameEntityPrefab
    python3 scripts/hashtool.py check names.txt      # what's missing from the repo

`check` accepts a plain one-name-per-line list, or the sectioned
`Classes:` / `Fields:` format (blank lines and `# comments` ignored); a
`Classes:`/`Fields:` header routes following names to bintypes/binfields, and an
unsectioned list is checked against both tables.
"""

import argparse
import os
import sys

# Same directory on sys.path[0] when run as a script, so this import is the one
# source of truth for line format, parsing, and canonical ordering.
from update_hashes import RE_LINE, parse_table, render, write_if_changed

TABLES = ("bintypes", "binfields")
# Section header in a name list -> which override table it feeds.
SECTION_TABLE = {"classes": "bintypes", "fields": "binfields"}


def fnv1a_32(name):
    """The bin-hash: FNV-1a over the lowercased UTF-8 bytes, 32-bit."""
    h = 2166136261
    for b in name.lower().encode("utf-8"):
        h = ((h ^ b) * 16777619) & 0xFFFFFFFF
    return h


def override_path(hashes_dir, table):
    return os.path.join(hashes_dir, "overrides", f"{table}.txt")


def merged_path(hashes_dir, table):
    return os.path.join(hashes_dir, f"hashes.{table}.txt")


def load(path):
    """{hash: name} for a table file, or {} if it doesn't exist yet."""
    if not os.path.exists(path):
        return {}
    with open(path, encoding="utf-8") as f:
        return parse_table(f.read(), path)


def load_tables(hashes_dir, table):
    """Returns (overrides, merged) hash->name maps for one table.

    `merged` is the built hashes.<table>.txt (upstream + overrides applied); it's
    what actually resolves today. `overrides` is only the local layer.
    """
    return load(override_path(hashes_dir, table)), load(merged_path(hashes_dir, table))


def parse_name_list(path):
    """A name list -> [(name, table_or_None)]. Section headers set the table;
    an unsectioned list yields table=None (check against both)."""
    out = []
    table = None
    with open(path, encoding="utf-8") as f:
        for raw in f:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            key = line.rstrip(":").lower()
            if line.endswith(":") and key in SECTION_TABLE:
                table = SECTION_TABLE[key]
                continue
            # Tolerate "- name" bullet lists and "hash name" lines alike.
            name = line.lstrip("-").strip()
            if " " in name:
                name = name.split()[-1]
            out.append((name, table))
    return out


def cmd_fnv(args):
    for name in args.names:
        print(f"{fnv1a_32(name):08x} {name}")
    return 0


def cmd_add(args):
    if args.table not in TABLES:
        print(f"[error] table must be one of {TABLES}", file=sys.stderr)
        return 2

    over_path = override_path(args.hashes, args.table)
    overrides, merged = load_tables(args.hashes, args.table)

    added, skipped = [], []
    for name in args.names:
        h = f"{fnv1a_32(name):08x}"
        if overrides.get(h) == name:
            skipped.append((h, name, "already an override"))
        elif merged.get(h) == name:
            # Upstream already resolves it; an override would be redundant.
            skipped.append((h, name, "already upstream"))
        elif h in overrides:
            print(f"[error] {h}: collision - override has {overrides[h]!r}, "
                  f"refusing to overwrite with {name!r}", file=sys.stderr)
            return 1
        else:
            overrides[h] = name
            added.append((h, name))
            if h in merged and merged[h] != name:
                print(f"[warn] {h} {name}: upstream resolves this hash to "
                      f"{merged[h]!r} - double-check the crack")

    if added and write_if_changed(over_path, render(overrides)):
        for h, name in added:
            print(f"[add] {args.table}: {h} {name}")
    for h, name, why in skipped:
        print(f"[skip] {args.table}: {h} {name} ({why})")
    if added:
        print(f"[ok] {over_path}: {len(added)} added, now {len(overrides)} entries "
              f"- rebuild with `python3 scripts/db_build.py` (or update_hashes.py)")
    return 0


def cmd_sort(args):
    tables = [args.table] if args.table else TABLES
    for table in tables:
        path = override_path(args.hashes, table)
        entries = load(path)
        if not entries:
            continue
        if write_if_changed(path, render(entries)):
            print(f"[ok] {path}: renormalized ({len(entries)} entries)")
        else:
            print(f"[ok] {path}: already canonical ({len(entries)} entries)")
    return 0


def cmd_prune(args):
    tables = [args.table] if args.table else TABLES
    total = 0
    for table in tables:
        over_path = override_path(args.hashes, table)
        overrides = load(over_path)
        # merged_path is the upstream mirror now (overrides aren't baked in), so
        # an entry the mirror already resolves to the same name is dead weight.
        mirror = load(merged_path(args.hashes, table))
        redundant = {h: n for h, n in overrides.items() if mirror.get(h) == n}
        if not redundant:
            print(f"[ok] {over_path}: no redundant entries")
            continue
        kept = {h: n for h, n in overrides.items() if h not in redundant}
        write_if_changed(over_path, render(kept))
        total += len(redundant)
        print(f"[prune] {table}: removed {len(redundant)} now served by upstream, "
              f"{len(kept)} remain")
        if args.verbose:
            for h, n in sorted(redundant.items(), key=lambda kv: (kv[1], kv[0])):
                print(f"    - {h} {n}")
    print(f"[ok] pruned {total} redundant override(s)")
    return 0


def cmd_lookup(args):
    query = args.query
    hit = False
    is_hash = all(c in "0123456789abcdefABCDEF" for c in query) and query
    for table in TABLES:
        overrides, merged = load_tables(args.hashes, table)
        # Layer overrides on top so a just-added crack resolves before the
        # merged hashes.<table>.txt has been rebuilt.
        merged = {**merged, **overrides}
        if is_hash:
            key = f"{int(query, 16):08x}"
            if key in merged:
                print(f"{key} {merged[key]}  [{table}]")
                hit = True
        else:
            key = f"{fnv1a_32(query):08x}"
            name = merged.get(key)
            mark = "==" if name == query else "!=" if name else "  "
            status = f"{name} (stored)" if name else "unresolved"
            print(f"{key} {query}  [{table}] {mark} {status}")
            hit = hit or name is not None
    return 0 if hit else 1


def cmd_check(args):
    names = parse_name_list(args.file)
    # Preload both tables once.
    tabs = {t: load_tables(args.hashes, t) for t in TABLES}

    missing, in_over, upstream = [], [], []
    for name, table in names:
        candidates = [table] if table else list(TABLES)
        h = f"{fnv1a_32(name):08x}"
        where = None
        for t in candidates:
            overrides, merged = tabs[t]
            if overrides.get(h) == name:
                where = ("override", t)
                break
            if merged.get(h) == name:
                where = ("upstream", t)
                break
        if where is None:
            missing.append((h, name, table))
        elif where[0] == "override":
            in_over.append((h, name, where[1]))
        else:
            upstream.append((h, name, where[1]))

    if in_over:
        print(f"# already in overrides ({len(in_over)})")
        for h, name, t in in_over:
            print(f"  {h} {name}  [{t}]")
    if upstream:
        print(f"# resolved upstream, no override needed ({len(upstream)})")
        for h, name, t in upstream:
            print(f"  {h} {name}  [{t}]")
    if missing:
        print(f"# MISSING - not resolved anywhere ({len(missing)})")
        for h, name, t in missing:
            hint = t or "bintypes?/binfields?"
            print(f"  {h} {name}  -> add to {hint}")
    else:
        print("# nothing missing - every name resolves")
    return 1 if missing else 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--hashes", default="hashes", help="directory of hash name lists")
    sub = parser.add_subparsers(dest="cmd", required=True)

    p = sub.add_parser("fnv", help="print FNV-1a-32 of each name")
    p.add_argument("names", nargs="+")
    p.set_defaults(func=cmd_fnv)

    p = sub.add_parser("add", help="add cracked name(s) to an override table")
    p.add_argument("table", help="bintypes | binfields")
    p.add_argument("names", nargs="+")
    p.set_defaults(func=cmd_add)

    p = sub.add_parser("sort", help="renormalize override file(s) in place")
    p.add_argument("table", nargs="?", help="bintypes | binfields (default both)")
    p.set_defaults(func=cmd_sort)

    p = sub.add_parser("prune", help="drop overrides upstream now serves identically")
    p.add_argument("table", nargs="?", help="bintypes | binfields (default both)")
    p.add_argument("-v", "--verbose", action="store_true", help="list what was removed")
    p.set_defaults(func=cmd_prune)

    p = sub.add_parser("lookup", help="resolve a hash or a name against the merged tables")
    p.add_argument("query")
    p.set_defaults(func=cmd_lookup)

    p = sub.add_parser("check", help="report which names in a list aren't resolved yet")
    p.add_argument("file")
    p.set_defaults(func=cmd_check)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())

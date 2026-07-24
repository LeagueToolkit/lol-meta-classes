#!/bin/env python
"""Refresh the vendored hashtables in hashes/ from their configured source.

Sources live in hashes/sources.toml. hashes/hashes.<table>.txt is an **exact
mirror** of the fetched upstream table - this script writes it byte-for-byte as
served, so `git diff` on it is pure upstream drift.

Genuinely-local cracks that upstream does not have live in
hashes/overrides/<table>.txt and are NOT baked in here. They are layered over
the mirror at the point of use (db_import.read_resolved_hashes), so the two
committed files stay independently reviewable. This script still reads the
overrides to renormalize them and to warn when one has become redundant
(upstream now serves the identical name).

The tables are committed, so `git diff` after a run is the drift report; this
script does not compute one. Refreshes are meant to land as reviewed PRs,
because a rename cascades into class/property names across the whole
db/meta.db.json history.

Usage:
    python3 scripts/update_hashes.py
    python3 scripts/update_hashes.py --hashes hashes
"""

import argparse
import json
import os
import re
import sys
import tomllib
import urllib.error
import urllib.request
from datetime import datetime, timezone
from email.utils import parsedate_to_datetime

USER_AGENT = "lol-meta-classes update_hashes.py (+https://github.com/Crauzer/lol-meta-classes)"
RE_LINE = re.compile(r"^([0-9a-f]{1,16}) (\S+)$")
TABLES = ("bintypes", "binfields")


def iso_utc(dt):
    """One timestamp format everywhere in provenance.json."""
    return dt.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def source_url(name, cfg):
    """`url` wins; `repo` + `path` (+ `ref`) is raw.githubusercontent sugar."""
    if "url" in cfg:
        return cfg["url"]
    if "repo" in cfg and "path" in cfg:
        ref = cfg.get("ref", "master")
        return f"https://raw.githubusercontent.com/{cfg['repo']}/{ref}/{cfg['path']}"
    raise ValueError(f"[{name}] needs either `url` or `repo` + `path`")


def parse_table(text, origin):
    """Text -> {hash: name}, rejecting malformed or duplicated lines."""
    entries = {}
    for lineno, line in enumerate(text.splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        m = RE_LINE.match(line)
        if not m:
            raise ValueError(f"{origin}:{lineno}: malformed line: {line!r}")
        h, name = m.group(1), m.group(2)
        key = f"{int(h, 16):08x}"
        if key in entries and entries[key] != name:
            raise ValueError(f"{origin}:{lineno}: duplicate hash {key} "
                             f"({entries[key]!r} vs {name!r})")
        entries[key] = name
    return entries


def fetch(url):
    """Returns (text, last_modified). No ETag: CDragon serves nginx's weak
    validator (mtime-hex + size), which is derived from Last-Modified, differs
    per mirror for identical bytes, and has no consumer here -- we compare
    content, not validators."""
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    with urllib.request.urlopen(req, timeout=60) as resp:
        text = resp.read().decode("utf-8")
        raw_modified = resp.headers.get("Last-Modified")

    # The header is RFC 7231 IMF-fixdate ("Fri, 17 Jul 2026 20:54:33 GMT");
    # store it as ISO 8601 UTC so provenance.json has one timestamp format.
    modified = iso_utc(parsedate_to_datetime(raw_modified)) if raw_modified else None
    return text, modified


def read_overrides(path):
    if not os.path.exists(path):
        return {}
    with open(path, encoding="utf-8") as f:
        return parse_table(f.read(), path)


def render(entries):
    """Upstream (CDragon) orders by name, ASCII; match it so a refresh diff is
    only the actual changes. Ties broken by hash for stability."""
    lines = sorted(entries.items(), key=lambda kv: (kv[1], kv[0]))
    return "".join(f"{h} {name}\n" for h, name in lines)


def write_if_changed(path, text):
    # Compare with newlines normalized: a CRLF checkout (core.autocrlf=true on
    # Windows) must not read as drift, or every local run would rewrite the
    # tables and restamp provenance.
    if os.path.exists(path):
        with open(path, encoding="utf-8") as f:
            if f.read() == text:
                return False
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--hashes", default="hashes", help="directory of hash name lists")
    args = parser.parse_args()

    config_path = os.path.join(args.hashes, "sources.toml")
    with open(config_path, "rb") as f:
        config = tomllib.load(f)

    provenance = {"fetchedAt": None, "tables": {}}
    changed = False

    for table in TABLES:
        if table not in config:
            print(f"[error] {config_path}: missing [{table}] section", file=sys.stderr)
            return 1
        url = source_url(table, config[table])
        print(f"[..] {table}: {url}")

        try:
            text, last_modified = fetch(url)
            entries = parse_table(text, url)
        except (urllib.error.URLError, ValueError) as e:
            # Abort without writing: a partial or malformed table would silently
            # un-resolve names across the entire database.
            print(f"[error] {table}: {e}", file=sys.stderr)
            return 1

        override_path = os.path.join(args.hashes, "overrides", f"{table}.txt")
        overrides = read_overrides(override_path)
        # Normalize the override file itself, so a hand-edit (or a `sort` run
        # under a different locale) is re-canonicalized here instead of
        # surfacing later as a pile of no-op line moves.
        if overrides and write_if_changed(override_path, render(overrides)):
            print(f"[ok] {override_path}: renormalized")
        # Overrides are layered at build time, not baked into the mirror, so an
        # entry that upstream now serves identically is dead weight - flag it for
        # deletion. (No `entries.update(overrides)`: the written table below is an
        # exact upstream mirror.)
        redundant = [h for h, name in overrides.items() if entries.get(h) == name]
        for h in redundant:
            print(f"[warn] {override_path}: {h} {overrides[h]} matches upstream "
                  f"exactly - the override can be deleted")

        out_path = os.path.join(args.hashes, f"hashes.{table}.txt")
        if write_if_changed(out_path, render(entries)):
            changed = True
            print(f"[ok] {out_path}: {len(entries)} upstream entries (updated)")
        else:
            print(f"[ok] {out_path}: {len(entries)} upstream entries (unchanged)")

        provenance["tables"][table] = {
            "url": url,
            "lastModified": last_modified,
            "entries": len(entries),
            "overrides": len(overrides),
        }

    # Provenance describes the vendored snapshot, so it is only rewritten when
    # that snapshot moves. Restamping `fetchedAt` on every run would dirty the
    # tree and open an empty PR every week.
    prov_path = os.path.join(args.hashes, "provenance.json")
    if changed or not os.path.exists(prov_path):
        provenance["fetchedAt"] = iso_utc(datetime.now(timezone.utc))
        write_if_changed(prov_path, json.dumps(provenance, indent=2) + "\n")

    if changed:
        print("[ok] tables changed - review `git diff hashes/` and rebuild with "
              "`python3 scripts/db_build.py`")
    else:
        print("[ok] no changes")
    return 0


if __name__ == "__main__":
    sys.exit(main())

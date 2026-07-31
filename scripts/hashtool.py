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
    python3 scripts/hashtool.py add bintypes GameEntityPrefab -b vfx-driver-graph
    python3 scripts/hashtool.py add binfields envMesh -e "Map+80; attested Map12/22/33"
    python3 scripts/hashtool.py sort                 # renormalize both overrides
    python3 scripts/hashtool.py lookup 2b949af2      # hash -> name (any table)
    python3 scripts/hashtool.py lookup GameEntityPrefab
    python3 scripts/hashtool.py check names.txt      # what's missing from the repo
    python3 scripts/hashtool.py ledger               # backlog, by batch
    python3 scripts/hashtool.py ledger --reconcile   # sync status against upstream
    python3 scripts/hashtool.py ledger --set 51e9b98e --status submitted --pr 42
    python3 scripts/hashtool.py ledger --batch vfx-driver-graph --list
    python3 scripts/hashtool.py ledger --batch vfx-driver-graph --status submitted --pr 42
    python3 scripts/hashtool.py ledger --match 'Monarch*' --status local

`check` accepts a plain one-name-per-line list, or the sectioned
`Classes:` / `Fields:` format (blank lines and `# comments` ignored); a
`Classes:`/`Fields:` header routes following names to bintypes/binfields, and an
unsectioned list is checked against both tables.

Every `add` also writes a row to hashes/overrides/ledger.tsv, which is what
records when a crack happened and whether it has been sent upstream yet - see
ledger.py. Overrides alone can't answer "what still needs a CDragon PR?". The
per-batch method and attestation go beside it in batches.tsv.

Cracks are grouped into batches (`-b` on `add`): the campaign a name came out of,
and the unit an upstream PR is built from. `--batch` is the ledger's selector -
it filters `--list`, and with `--status/--pr` it marks the whole campaign
submitted in one command. `--match GLOB` selects by name instead, across
batches, for a family that was cracked over several sittings.

Not every crack should go upstream: `--status local` (or `add -l`) holds a name
back permanently - it still resolves here, but drops out of `--list pending`,
which is the "what still needs a CDragon PR" view.
"""

import argparse
import datetime
import fnmatch
import os
import sys

import ledger as ledger_mod
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
    batch = args.batch or ledger_mod.UNSORTED
    try:
        ledger_mod.check_slug(batch)
    except ValueError as e:
        print(f"[error] {e}", file=sys.stderr)
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
        # A crack with no ledger row is a crack that gets forgotten, so this is
        # not optional and not a separate command.
        led_path = ledger_mod.ledger_path(args.hashes)
        led = ledger_mod.load(led_path)
        today = datetime.date.today().isoformat()
        status = ledger_mod.LOCAL if args.local else "pending"
        for h, name in added:
            led.rows[(args.table, h)] = ledger_mod.make_row(
                h, args.table, name, today, batch=batch, status=status)
        if args.evidence:
            led.batches[batch] = args.evidence
        led.batches.setdefault(batch, "")
        ledger_mod.save(args.hashes, led, write_if_changed)
        print(f"[ok] {led_path}: {len(added)} row(s) {status} in batch {batch}")
        if args.local:
            print("[note] marked local-only: these resolve here but are held "
                  "back from upstream, so they won't show up in `--list pending`")
        if batch == ledger_mod.UNSORTED:
            print("[note] no -b/--batch given; these rows sit in `unsorted` "
                  "until attributed")
        elif not led.batches[batch]:
            why = ("say why it's held back - that reason is the only thing "
                   "stopping someone submitting it later"
                   if args.local else
                   "it's what the upstream PR description is built from")
            print(f"[note] batch {batch!r} has no evidence note; add one with "
                  f"`ledger --batch {batch} -e \"...\"` - {why}")
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


def select_rows(led, batch=None, match=None):
    """Rows in a batch, whose name matches a glob, or both. Glob is
    case-sensitive: these names are recorded as they were cracked, and casing is
    a claim about the name, not a formatting detail."""
    return [r for r in led.rows.values()
            if (batch is None or r["batch"] == batch)
            and (match is None or fnmatch.fnmatchcase(r["name"], match))]


def print_batches(led):
    """The default view: one line per campaign, not one per crack."""
    order = led.order()
    if not order:
        print("  (empty ledger)")
        return
    width = max(len(s) for s in order)
    cols = [ledger_mod.ABBREV[s] for s in ledger_mod.STATUSES]
    print(f"  {'batch'.ljust(width)}  rows  " + "  ".join(cols) + "  evidence")
    for slug in order:
        c = led.counts(slug)
        note = led.batches.get(slug, "") or "(none)"
        cells = "  ".join(f"{c[s]:>{len(ledger_mod.ABBREV[s])}}"
                          for s in ledger_mod.STATUSES)
        print(f"  {slug.ljust(width)}  {len(led.of_batch(slug)):4}  {cells}  "
              f"{note[:56]}")


def cmd_ledger(args):
    led_path = ledger_mod.ledger_path(args.hashes)
    led = ledger_mod.load(led_path)
    rows = led.rows
    overrides = {t: load(override_path(args.hashes, t)) for t in TABLES}
    mirrors = {t: load(merged_path(args.hashes, t)) for t in TABLES}
    dirty = False

    if args.batch:
        try:
            ledger_mod.check_slug(args.batch)
        except ValueError as e:
            print(f"[error] {e}", file=sys.stderr)
            return 2
    if args.set and (args.batch or args.match):
        print("[error] --set selects one row; --batch/--match select many. "
              "Use one or the other", file=sys.stderr)
        return 2
    if args.evidence and not args.batch:
        # The note is stored per batch, so there's nowhere to put it otherwise.
        print("[error] -e/--evidence needs --batch", file=sys.stderr)
        return 2

    if args.seed:
        # Entries predating the ledger: date them from git blame of the override
        # line, so the history is real rather than all-today. The commit that
        # introduced the line also names the batch - one sitting, one commit,
        # one campaign - which is the only attribution recoverable after the
        # fact. The batch note stays blank: that isn't recoverable, and
        # inventing it would be worse than an honest gap.
        repo_root = os.path.dirname(os.path.abspath(args.hashes)) or "."
        today = datetime.date.today().isoformat()
        seeded = rebatched = 0
        for table in TABLES:
            rel = os.path.join(args.hashes, "overrides", f"{table}.txt")
            blame = ledger_mod.blame_lines(repo_root, rel.replace("\\", "/"))
            for h, name in overrides[table].items():
                date, slug = blame.get(f"{h} {name}", (today, ledger_mod.UNSORTED))
                if (table, h) not in rows:
                    rows[(table, h)] = ledger_mod.make_row(
                        h, table, name, date or today, batch=slug)
                    seeded += 1
                elif rows[(table, h)]["batch"] == ledger_mod.UNSORTED:
                    # An older row, or one added without -b: attribute it now
                    # but leave its recorded crack date alone.
                    rows[(table, h)]["batch"] = slug
                    rebatched += slug != ledger_mod.UNSORTED
        print(f"[seed] {seeded} pre-existing override(s) recorded as pending"
              if seeded else "[seed] nothing to seed - every override has a row")
        if rebatched:
            print(f"[seed] {rebatched} unsorted row(s) attributed to a batch "
                  f"from the commit that introduced them")
        dirty = dirty or bool(seeded or rebatched)

    if args.batch and args.batch not in led.order():
        # A mistyped selector must not read as "that campaign is empty".
        print(f"[error] no batch {args.batch!r} - known: "
              f"{', '.join(led.order()) or '(none)'}", file=sys.stderr)
        return 1

    if args.set:
        h = f"{int(args.set, 16):08x}"
        matches = [k for k in rows if k[1] == h
                   and (args.table is None or k[0] == args.table)]
        if not matches:
            print(f"[error] {h}: no ledger row (add the crack first)", file=sys.stderr)
            return 1
        if len(matches) > 1:
            # The 6 class-and-field names land here; make the caller pick.
            print(f"[error] {h} is in {', '.join(t for t, _ in matches)} - "
                  f"disambiguate with --table", file=sys.stderr)
            return 2
        if not (args.status or args.pr):
            print("[error] --set needs --status and/or --pr", file=sys.stderr)
            return 2
        r = rows[matches[0]]
        if args.status:
            r["status"] = args.status
        if args.pr:
            r["pr"] = args.pr
        print(f"[set] {h} {r['name']} [{r['table']}]: "
              f"status={r['status']} pr={r['pr']}")
        dirty = True

    elif (args.batch or args.match) and (args.status or args.pr or args.evidence):
        # The payoff of batching: a campaign goes upstream as one PR, so it
        # flips to submitted as one command. --match cuts the other way, across
        # batches: a name family (`Monarch*`) can be spread over several
        # campaigns and still be one decision about what leaves this repo.
        target = select_rows(led, args.batch, args.match)
        if not target and (args.status or args.pr):
            print(f"[error] no rows match {args.match!r}"
                  + (f" in batch {args.batch}" if args.batch else ""),
                  file=sys.stderr)
            return 1
        for r in target:
            if args.status:
                r["status"] = args.status
            if args.pr:
                r["pr"] = args.pr
        if args.evidence:
            led.batches[args.batch] = args.evidence
        where = args.batch or f"match {args.match!r}"
        print(f"[batch] {where}: {len(target)} row(s)"
              + (f" status={args.status}" if args.status else "")
              + (f" pr={args.pr}" if args.pr else "")
              + (" evidence set" if args.evidence else ""))
        if args.status == ledger_mod.LOCAL:
            # A bare `local` with no reason is indistinguishable from a crack
            # someone forgot to submit, and reads as one a year later.
            where_why = (f"the batch note (`-e`)" if args.batch else
                         "hashes/overrides/README.md, under Local-only cracks "
                         "- this selection crosses batches, so the per-batch "
                         "note is the wrong place")
            print(f"[note] local-only: held back from upstream on purpose. "
                  f"Record why in {where_why}")
        dirty = True

    if args.reconcile:
        # Upstream serving the identical name is the one signal that a crack
        # actually landed, whatever the row claimed.
        landed = [r for r in rows.values()
                  if mirrors[r["table"]].get(r["hash"]) == r["name"]
                  and r["status"] != "merged"]
        for r in landed:
            if r["status"] == ledger_mod.LOCAL:
                # Held back here, public anyway. Worth saying out loud: it means
                # either someone submitted it, or the name reached upstream by
                # another route - and the row was claiming otherwise.
                print(f"[warn] {r['hash']} {r['name']}: marked local-only, but "
                      f"upstream now serves this name - it is public; "
                      f"recording it merged")
            else:
                print(f"[merged] {r['hash']} {r['name']} - upstream serves it; "
                      f"`prune` will drop the override")
            r["status"] = "merged"
        orphan = [r for r in rows.values()
                  if r["hash"] not in overrides[r["table"]] and r["status"] != "merged"]
        for r in orphan:
            print(f"[warn] {r['hash']} {r['name']}: ledger row but no override "
                  f"in {r['table']} - was it removed by hand?")
        untracked = [(t, h, n) for t in TABLES
                     for h, n in overrides[t].items() if (t, h) not in rows]
        for t, h, n in untracked:
            print(f"[warn] {h} {n} [{t}]: override with no ledger row - "
                  f"run `ledger --seed`")
        if not (landed or orphan or untracked):
            print("[ok] ledger and override tables agree")
        dirty = dirty or bool(landed)

    if dirty:
        ledger_mod.save(args.hashes, led, write_if_changed)
        print(f"[ok] {led_path}: {len(rows)} row(s) in {len(led.order())} batch(es)")

    print_batches(led)
    counts = led.counts()
    print("  total: " + "  ".join(f"{s} {counts[s]}" for s in ledger_mod.STATUSES))

    want = args.list
    if want:
        # Listing stays grouped: a flat 1200-line dump by date is what the
        # batches exist to replace.
        for slug in led.order():
            if args.batch and slug != args.batch:
                continue
            listed = [r for r in select_rows(led, slug, args.match)
                      if want == "all" or r["status"] == want]
            if not listed:
                continue
            print(f"\n# {slug} ({len(listed)} {want})")
            for r in sorted(listed, key=lambda r: (r["cracked"], r["name"])):
                pr = "" if r["pr"] == ledger_mod.NO_PR else f"  {r['pr']}"
                print(f"  {r['cracked']}  {r['hash']}  {r['name']}  "
                      f"[{r['table']}/{r['status']}]{pr}")
    return 0


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
    p.add_argument("-b", "--batch", default=None,
                   help="batch slug: the campaign these cracks came from, and "
                        "the unit the upstream PR is built from "
                        "(default: unsorted)")
    p.add_argument("-e", "--evidence", default="",
                   help="one line: the batch's method and what attests it "
                        "(stored per batch in batches.tsv, reused in the "
                        "upstream PR)")
    p.add_argument("-l", "--local", action="store_true",
                   help="mark these local-only: resolved here, held back from "
                        "upstream (unreleased content, etc.)")
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

    p = sub.add_parser("ledger", help="crack history by batch + submission status")
    p.add_argument("--list", nargs="?", const="pending", default=None,
                   choices=(*ledger_mod.STATUSES, "all"),
                   help="list rows with this status, grouped by batch "
                        "(default pending)")
    p.add_argument("--seed", action="store_true",
                   help="record every override that has no row yet, dating and "
                        "batching it from the commit that introduced it")
    p.add_argument("--reconcile", action="store_true",
                   help="mark rows merged once upstream serves the name, and "
                        "warn about rows and overrides that disagree")
    p.add_argument("--set", metavar="HASH", help="select one row")
    p.add_argument("--batch", metavar="SLUG",
                   help="select a whole batch: filters --list, and applies the "
                        "edits below to every row in it")
    p.add_argument("--match", metavar="GLOB",
                   help="select by name across batches, case-sensitively "
                        "(e.g. 'Monarch*'); combines with --batch to narrow")
    p.add_argument("--table", choices=TABLES,
                   help="with --set: disambiguate a hash present in both tables")
    p.add_argument("--status", choices=ledger_mod.STATUSES,
                   help="edit: set status on the selected row(s); `local` holds "
                        "them back from upstream for good")
    p.add_argument("--pr", help="edit: upstream PR link or number")
    p.add_argument("-e", "--evidence",
                   help="with --batch: set the batch's evidence note")
    p.set_defaults(func=cmd_ledger)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())

#!/bin/env python
"""Turn a guesser sweep into a page you can actually review.

A `{hash} {name}` line carries no evidence at all: the hash is why the name was
proposed, so reading the pair tells you nothing you did not already know. What
makes a candidate judgeable is everything *around* it - what the class derives
from, whether the meta calls it an interface, what its fields are named, which
classes own a field, and which run found it and at what noise. All of that is in
`db/meta.db.json` and `MANIFEST.tsv` already; this joins them.

Reads a sweep directory written by `guesser_sweep.py` and emits:

    candidates.json   the joined data, for anything else that wants it
    candidates.html   a self-contained page: filter, search, expand a row

The page is `scripts/templates/candidates.html` with the JSON substituted for
`__PAYLOAD__`. It is a *fragment* - no doctype, html, head or body tags - because
the Artifact publisher wraps it in that skeleton. Opening it in a browser
directly works anyway; browsers infer the skeleton.

To restyle the page, edit the template. Nothing here generates markup.

Usage:
    python3 scripts/guesser_report.py
    python3 scripts/guesser_report.py --out-dir hash-guesser-out -o review.html
"""

import argparse
import collections
import csv
import json
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
TEMPLATE = ROOT / "scripts" / "templates" / "candidates.html"
TABLES = ("bintypes", "binfields")
# Runs whose expected noise is low enough to read line by line. Kept in step
# with guesser_sweep.RUNS; a run named here that did not run is simply absent.
HI_RUNS = {"identity", "delete", "force2", "chain4", "suffix2.800"}
# How many field names / owning classes to carry into the page before eliding.
DETAIL_CAP = 6


def read_pairs(path):
    """`{hash} {name}` lines -> [(hash, name)], comments and junk skipped."""
    out = []
    if not path.exists():
        return out
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split(None, 1)
        if len(parts) == 2:
            out.append((parts[0].lower(), parts[1].strip()))
    return out


def load_meta(db_path):
    """The meta, indexed the two ways this needs it."""
    db = json.loads(db_path.read_text(encoding="utf-8"))
    classes = db["classes"]
    ext = db.get("externalTypeNames", {})
    patch = {v["build"]: v["patch"] for v in db["versions"]}

    def nameof(h):
        c = classes.get(h)
        if c and c.get("name"):
            return c["name"]
        return ext.get(h, h)

    # A field hash is declared by one or more classes, and knowing which is most
    # of what makes a field-name guess judgeable.
    owners = collections.defaultdict(list)
    for ch, c in classes.items():
        for ph in c.get("properties", {}):
            owners[ph].append(ch)
    return classes, nameof, owners, patch


def main():
    ap = argparse.ArgumentParser(
        description="Build the candidate review page from a guesser sweep")
    ap.add_argument("-d", "--out-dir", default="hash-guesser-out",
                    type=pathlib.Path, help="sweep directory to read and write")
    ap.add_argument("-o", "--html", type=pathlib.Path,
                    help="where to write the page "
                         "(default: <out-dir>/candidates.html)")
    ap.add_argument("--db", default="db/meta.db.json", type=pathlib.Path)
    ap.add_argument("--stamp", default="regenerated sweep",
                    help="eyebrow text on the page")
    args = ap.parse_args()

    out = (ROOT / args.out_dir).resolve()
    raw = out / "raw"
    if not (out / "MANIFEST.tsv").exists():
        sys.exit(f"no MANIFEST.tsv in {out}; run scripts/guesser_sweep.py first")
    classes, nameof, owners, patch = load_meta(ROOT / args.db)

    ledger = []
    with (out / "MANIFEST.tsv").open(encoding="utf-8", newline="") as f:
        for r in csv.DictReader(f, delimiter="\t"):
            ledger.append({
                "tier": r["tier"], "run": r["run"], "table": r["table"],
                "probes": int(r["probes"]), "states": int(r["states"]),
                "fp": float(r["expected_fp"]), "hits": int(r["hits"]),
            })

    # (table, hash, name) -> the runs that produced it.
    found_in = collections.defaultdict(set)
    for f in sorted(raw.glob("*.txt")):
        stem = f.name[: -len(".txt")]
        run, _, table = stem.rpartition(".")
        if table not in TABLES:
            continue
        for h, n in read_pairs(f):
            found_in[(table, h, n)].add(run)

    data = {t: [] for t in TABLES}
    for table in TABLES:
        by_hash = collections.defaultdict(list)
        for h, n in read_pairs(out / f"candidates.{table}.txt"):
            by_hash[h].append(n)
        for h, found in sorted(by_hash.items()):
            key = "0x" + h
            rec = {"h": h, "n": sorted(set(found))}
            if len(rec["n"]) > 1:
                rec["clash"] = True
            runs = set()
            for n in rec["n"]:
                runs |= found_in.get((table, h, n), set())
            rec["r"] = sorted(runs)
            if runs & HI_RUNS:
                rec["hi"] = True

            c = classes.get(key)
            if table == "bintypes" and c:
                revs = c["revisions"]
                rec["if"] = any(r.get("interface") for r in revs)
                rec["val"] = any(r.get("value") for r in revs)
                bases = []
                for r in revs:
                    bases.extend(r.get("bases", []))
                rec["b"] = [nameof(b) for b in sorted(set(bases))]
                props = c.get("properties", {})
                rec["np"] = len(props)
                named = [p["name"] for p in props.values() if p.get("name")]
                if named:
                    rec["p"] = sorted(named)[:DETAIL_CAP]
                rec["from"] = patch.get(revs[0]["from"], "")
                rec["live"] = "to" not in revs[-1]
            elif table == "binfields":
                own = owners.get(key, [])
                rec["own"] = sorted({nameof(o) for o in own})[:DETAIL_CAP]
                rec["nown"] = len(own)
                # The declared type, from whichever class declares it - a field
                # named ...Path that is a U32 is a wrong guess on sight.
                for o in own:
                    p = classes[o]["properties"].get(key, {})
                    tup = (p.get("revisions") or [{}])[-1].get("type")
                    if tup:
                        rec["ty"] = " ".join(
                            x if not str(x).startswith("0x") else nameof(x)
                            for x in tup if x not in ("0x0", None))
                        break
            data[table].append(rec)

    payload = json.dumps({"c": data, "l": ledger, "stamp": args.stamp},
                         separators=(",", ":"))
    (out / "candidates.json").write_text(payload, encoding="utf-8", newline="\n")

    html = args.html or (out / "candidates.html")
    html.write_text(
        TEMPLATE.read_text(encoding="utf-8").replace("__PAYLOAD__", payload),
        encoding="utf-8", newline="\n")

    for t in TABLES:
        hi = sum(1 for r in data[t] if r.get("hi"))
        clash = sum(1 for r in data[t] if r.get("clash"))
        print(f"[ok] {t}: {len(data[t])} hash(es), {hi} high-confidence, "
              f"{clash} with competing names")
    print(f"[ok] {html} ({html.stat().st_size // 1024} KB)")
    print("[..] publish it with the Artifact tool, or open it in a browser")


if __name__ == "__main__":
    main()

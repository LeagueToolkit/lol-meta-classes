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

Two joins go beyond the meta itself:

  - **Calibration.** `--baseline` takes a previous sweep's `candidates.json`.
    Any hash it proposed a name for that has been *resolved since* is a scored
    prediction, so the page can state a measured hit rate per run beside the
    modelled noise. This is the only number here that is evidence rather than
    prior. Read the caveat the page carries with it: the campaigns that landed
    those names read the sweep, so the denominator is proposals someone chose
    to work, not a blind sample.
  - **Families.** A bintypes candidate is easier to judge knowing which unnamed
    subtree it sits in, and the census says where a campaign is worth aiming
    next. Both come from `guesser_families.rank`, joined by class hash.

The page is `scripts/templates/candidates.html` with the JSON substituted for
`__PAYLOAD__`. It is a *fragment* - no doctype, html, head or body tags - because
the Artifact publisher wraps it in that skeleton. Opening it in a browser
directly works anyway; browsers infer the skeleton.

To restyle the page, edit the template. Nothing here generates markup.

Usage:
    # copy the last sweep aside first, so the new one can be scored against it
    cp hash-guesser-out/candidates.json hash-guesser-out/baseline.json
    python3 scripts/guesser_sweep.py --merge-ref sequencer-actions
    python3 scripts/guesser_report.py --stamp "sweep 2026-08-11"
"""

import argparse
import collections
import csv
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import guesser_families as fam_mod  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
TEMPLATE = ROOT / "scripts" / "templates" / "candidates.html"
TABLES = ("bintypes", "binfields")
# Runs whose expected noise is low enough to read line by line. Kept in step
# with guesser_sweep.RUNS; a run named here that did not run is simply absent.
HI_RUNS = {"identity", "delete", "force2", "chain4", "suffix2.800"}
# How many field names / owning classes to carry into the page before eliding.
DETAIL_CAP = 6
# Families smaller than this are not worth a campaign of their own, which is
# the same floor `guesser_families.py rank` defaults to.
FAM_MIN = 8
# How many families a single candidate lists. Subtrees nest, so a class sits in
# every family above it; the most specific few are the informative ones.
FAM_PER_ROW = 3


def dbkey(h):
    """`0f3f357f` -> `0xf3f357f`.

    The guesser writes `%08x`; the db writes hashes unpadded. 320 classes and
    1,323 field hashes begin with a zero, and looking those up padded silently
    returns nothing - the row renders with no base, no shape and no fields,
    which reads as "the meta knows nothing about it" rather than as a miss.
    """
    return "0x%x" % int(h, 16)


def padded(key):
    """The inverse: a db hash key back to the `%08x` the guesser emits."""
    return "%08x" % int(key, 16)


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


def resolved_names(classes):
    """table -> {hash without 0x: name}, for everything the db can now name.

    A class name is on the class; a field name is on whichever owner declares
    it, so the first owner that carries one settles it.
    """
    out = {t: {} for t in TABLES}
    for ch, c in classes.items():
        if c.get("name"):
            out["bintypes"][padded(ch)] = c["name"]
        for ph, p in c.get("properties", {}).items():
            if p.get("name"):
                out["binfields"].setdefault(padded(ph), p["name"])
    return out


def calibrate(baseline, now_named):
    """Score a previous sweep against the names that have landed since.

    Every hash the baseline proposed a name for was unresolved when it was
    proposed. The ones carrying a name today are the only scored predictions
    this exercise ever gets - one per name a campaign landed - so they are worth
    reporting run by run beside the modelled false-positive counts.

    What this is not: a blind test. The campaigns read the sweep, so a hash
    enters the denominator because someone chose to work it. It measures how
    often a worked proposal was the spelling that survived attestation, and the
    misses are the ones a different line of evidence cracked against the guess.
    """
    rows = {t: [] for t in TABLES}
    runs = collections.defaultdict(lambda: [0, 0])
    for table in TABLES:
        named = now_named[table]
        for r in baseline.get("c", {}).get(table, []):
            actual = named.get(r["h"])
            if not actual:
                continue
            ok = any(n.lower() == actual.lower() for n in r["n"])
            rows[table].append({"h": r["h"], "a": actual, "n": r["n"],
                                "ok": ok, "hi": bool(r.get("hi")),
                                "r": sorted(r.get("r", []))})
            for run in r.get("r", []):
                runs[run][1] += 1
                runs[run][0] += ok
    if not any(rows.values()):
        return None
    scored = sum(len(v) for v in rows.values())
    correct = sum(1 for v in rows.values() for r in v if r["ok"])
    hi = [r for v in rows.values() for r in v if r["hi"]]
    return {
        "stamp": baseline.get("stamp", "the previous sweep"),
        "rows": rows, "scored": scored, "correct": correct,
        "hi": len(hi), "hiok": sum(1 for r in hi if r["ok"]),
        "runs": sorted(
            ({"run": k, "tier": "hi" if k in HI_RUNS else "br",
              "ok": v[0], "n": v[1]} for k, v in runs.items()),
            key=lambda r: (-r["n"], r["run"])),
    }


def families(classes, nameof, cand_hashes):
    """The unnamed-family census, with each family's share of this sweep.

    `guesser_families.rank` answers where the unnamed surface is; joining the
    sweep onto it answers the next question, which is whether a generator has
    anything to say about that surface. A family of 74 with two candidates and
    a family of 23 with twenty are different propositions for a campaign.
    """
    children, _ = fam_mod.build_indexes(classes)
    unnamed = {h for h, c in classes.items()
               if fam_mod.is_live(c) and not c.get("name")}
    rows, member_of = [], collections.defaultdict(list)
    for root in set(children):
        sub = fam_mod.descendants(root, children, classes, True)
        un = [h for h in sub if h in unnamed]
        if len(un) < FAM_MIN:
            continue
        name = nameof(root)
        hit = sorted(h for h in un if padded(h) in cand_hashes)
        rows.append({"root": root, "name": name, "un": len(un),
                     "named": len(sub) - len(un), "cand": len(hit)})
        for h in un:
            member_of[padded(h)].append((len(un), name))
    rows.sort(key=lambda r: (-r["un"], r["name"]))
    # Most specific first: the smallest subtree containing the class says the
    # most about it, and the roots above it are all in the census anyway.
    return rows, {h: [n for _, n in sorted(v)][:FAM_PER_ROW]
                  for h, v in member_of.items()}


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
    ap.add_argument("--baseline", type=pathlib.Path, metavar="JSON",
                    help="a previous candidates.json to score this one against "
                         "(default: <out-dir>/baseline.json when it exists)")
    ap.add_argument("--no-baseline", action="store_true",
                    help="skip the calibration section entirely")
    args = ap.parse_args()

    out = (ROOT / args.out_dir).resolve()
    raw = out / "raw"
    if not (out / "MANIFEST.tsv").exists():
        sys.exit(f"no MANIFEST.tsv in {out}; run scripts/guesser_sweep.py first")
    classes, nameof, owners, patch = load_meta(ROOT / args.db)

    base_path = args.baseline or (out / "baseline.json")
    baseline = None
    if not args.no_baseline and base_path.exists():
        baseline = json.loads(base_path.read_text(encoding="utf-8"))
    elif args.baseline:
        sys.exit(f"no baseline at {base_path}")
    seen_before = {(t, r["h"]) for t in TABLES
                   for r in (baseline or {}).get("c", {}).get(t, [])}

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
            key = dbkey(h)
            rec = {"h": h, "n": sorted(set(found))}
            if len(rec["n"]) > 1:
                rec["clash"] = True
            runs = set()
            for n in rec["n"]:
                runs |= found_in.get((table, h, n), set())
            rec["r"] = sorted(runs)
            if runs & HI_RUNS:
                rec["hi"] = True
            if baseline is not None and (table, h) not in seen_before:
                rec["new"] = True

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

    fam_rows, fam_of = families(
        classes, nameof, {r["h"] for r in data["bintypes"]})
    for rec in data["bintypes"]:
        f = fam_of.get(rec["h"])
        if f:
            rec["f"] = f

    doc = {"c": data, "l": ledger, "stamp": args.stamp, "fam": fam_rows}
    if baseline is not None:
        doc["cal"] = calibrate(baseline, resolved_names(classes))
    payload = json.dumps(doc, separators=(",", ":"))
    (out / "candidates.json").write_text(payload, encoding="utf-8", newline="\n")

    html = args.html or (out / "candidates.html")
    html.write_text(
        TEMPLATE.read_text(encoding="utf-8").replace("__PAYLOAD__", payload),
        encoding="utf-8", newline="\n")

    for t in TABLES:
        hi = sum(1 for r in data[t] if r.get("hi"))
        clash = sum(1 for r in data[t] if r.get("clash"))
        fresh = sum(1 for r in data[t] if r.get("new"))
        print(f"[ok] {t}: {len(data[t])} hash(es), {hi} high-confidence, "
              f"{clash} with competing names"
              + (f", {fresh} not in the baseline" if baseline else ""))
    cal = doc.get("cal")
    if cal:
        print(f"[ok] calibration vs {base_path.name}: {cal['correct']}/"
              f"{cal['scored']} resolved-since proposals correct "
              f"({cal['hiok']}/{cal['hi']} of the high-confidence ones)")
    print(f"[ok] {len(fam_rows)} unnamed famil(ies) of {FAM_MIN}+ in the census")
    print(f"[ok] {html} ({html.stat().st_size // 1024} KB)")
    print("[..] publish it with the Artifact tool, or open it in a browser")


if __name__ == "__main__":
    main()

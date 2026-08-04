#!/bin/env python
"""Run a full hash-guesser sweep and record what each run cost in noise.

One guesser run answers very little on its own. What is worth keeping is a
*spread* of runs at different depths and wordlist sizes, each labelled with the
number of its own hits that chance accounts for, so the output can be read
tier by tier instead of as one undifferentiated pile.

Everything here is bookkeeping around that idea:

  - **A merged corpus.** The guesser subtracts the names it can see, which are
    the ones on the branch it runs from. Names sitting on another branch are
    invisible to it and get re-guessed. `--merge-ref` folds those override
    tables into a scratch copy of `hashes/` so they count as known, and their
    vocabulary feeds the wordlist too.
  - **Suffix lists per table.** Class names and field names have different
    tails - Data/Instance/Controller against Icon/Name/Text/Button - so each
    table gets the top tails from its own half of the corpus.
  - **A manifest.** `MANIFEST.tsv` records probes, target states and expected
    false positives per run. That is the number to read before the hits: a run
    whose expected noise approaches its hit count found nothing you can trust.

Output is candidates, never cracks. See README, "Cracking unresolved hashes".

Usage:
    # the sweep as run for the current lists
    python3 scripts/guesser_sweep.py --merge-ref sequencer-actions

    # a quick look with only the quiet runs
    python3 scripts/guesser_sweep.py --tier hi

    # then build the review page
    python3 scripts/guesser_report.py
"""

import argparse
import collections
import csv
import os
import pathlib
import re
import shutil
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import names as names_mod  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
TABLES = ("bintypes", "binfields")

# (tier, name, extra args). `SUFFIXES` is replaced with the table's suffix list.
#
# `hi` runs are quiet enough to read line by line; `br` runs are broad and part
# chance by construction. The split is a judgement about noise, not about mode:
# what puts a run in `hi` is that its expected false positives stay in single
# digits, which is why force2 (full wordlist, depth 2) qualifies and force3
# (depth 3, top 400) does not.
SUFFIXES = object()
RUNS = [
    ("hi", "identity", ["--mode", "identity"]),
    ("hi", "force2", ["--mode", "force", "--depth", "2"]),
    ("hi", "suffix2.800",
     ["--mode", "force", "--depth", "2", "--top", "800", SUFFIXES]),
    ("br", "mutate.500", ["--mode", "mutate", "--top", "500"]),
    ("br", "force3.400", ["--mode", "force", "--depth", "3", "--top", "400"]),
    ("br", "suffix2.2000",
     ["--mode", "force", "--depth", "2", "--top", "2000", SUFFIXES]),
    ("br", "suffix3.150",
     ["--mode", "force", "--depth", "3", "--top", "150", SUFFIXES]),
]
HI_RUNS = {name for tier, name, _ in RUNS if tier == "hi"}

RE_PROBES = re.compile(r"^\[ok\] (\d+) probe")
RE_STATES = re.compile(r"^\[\.\.\] (\d+) target state")
RE_FP = re.compile(r"= ([\d.]+) expected false positive")


def find_exe(explicit):
    """The release binary, wherever cargo put it.

    `.cargo/config.toml` pins a linux target, so on Windows the build needs
    `--target x86_64-pc-windows-msvc` and the binary lands one directory deeper
    than usual. Rather than encode that, ask cargo where its target directory
    is and look in every profile under it.
    """
    if explicit:
        p = pathlib.Path(explicit)
        if not p.exists():
            sys.exit(f"no guesser at {p}")
        return p
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT, capture_output=True, text=True)
    roots = []
    if out.returncode == 0:
        import json
        roots.append(pathlib.Path(json.loads(out.stdout)["target_directory"]))
    roots.append(ROOT / "target")
    for root in roots:
        for exe in ("hash-guesser", "hash-guesser.exe"):
            hits = sorted(root.glob(f"**/release/{exe}"))
            if hits:
                return hits[0]
    sys.exit(
        "no hash-guesser binary found. Build it with:\n"
        "  cargo build --release -p hash-guesser"
        + (" --target x86_64-pc-windows-msvc" if os.name == "nt" else "")
        + "\nor pass --exe <path>.")


def build_corpus(dest, refs):
    """A copy of hashes/ with `refs`' override tables folded in.

    Only the overrides: the vendored mirror is the same on every branch, and
    copying it once keeps this cheap.
    """
    if dest.exists():
        shutil.rmtree(dest)
    (dest / "overrides").mkdir(parents=True)
    for t in TABLES:
        shutil.copy(ROOT / "hashes" / f"hashes.{t}.txt", dest)
        rows = {}
        # Refs first, the working tree last, so on a hash both carry the local
        # spelling wins. This is not cosmetic. A ref that branched before
        # `hashtool lint --fix` still holds pre-rule casing, and camelCase costs
        # the wordlist a word: the splitter reads the `uv` of `uvAnimation` as a
        # Hungarian prefix and drops it, where `UvAnimation` yields Uv +
        # Animation. Same hash either way - the spelling only matters for the
        # vocabulary, which is the whole point of the rule. See scripts/names.py.
        sources = []
        for ref in refs:
            got = subprocess.run(
                ["git", "show", f"{ref}:hashes/overrides/{t}.txt"],
                cwd=ROOT, capture_output=True, text=True)
            if got.returncode != 0:
                sys.exit(f"cannot read hashes/overrides/{t}.txt at {ref!r}")
            sources.append(got.stdout)
        sources.append((ROOT / "hashes" / "overrides" / f"{t}.txt").read_text(
            encoding="utf-8", errors="replace"))
        for text in sources:
            for line in text.splitlines():
                line = line.strip()
                if line and not line.startswith("#"):
                    parts = line.split(None, 1)
                    if len(parts) == 2:
                        rows[parts[0].lower()] = parts[1].strip()
        (dest / "overrides" / f"{t}.txt").write_text(
            "".join(f"{h} {n}\n" for h, n in sorted(rows.items())),
            encoding="utf-8", newline="\n")
    return dest


def table_names(corpus, table):
    out = []
    for p in (corpus / f"hashes.{table}.txt",
              corpus / "overrides" / f"{table}.txt"):
        for line in p.read_text(encoding="utf-8", errors="replace").splitlines():
            parts = line.split(None, 1)
            if len(parts) == 2 and not line.lstrip().startswith("#"):
                out.append(parts[1].strip())
    return out


def write_suffixes(corpus, table, dest, top):
    """The commonest final word of names in this table.

    Per table because the tails genuinely differ: classes end in Data,
    Instance, Controller; fields end in Icon, Name, Text, Button. Using one
    list for both would spend half the target states on tails that table never
    uses.
    """
    counts = collections.Counter()
    for name in table_names(corpus, table):
        words = names_mod.split_words(name, 0)
        if words:
            counts[words[-1]] += 1
    picked = [w for w, _ in counts.most_common(top)]
    dest.write_text("\n".join(picked) + "\n", encoding="utf-8", newline="\n")
    return counts.most_common(top)


def main():
    ap = argparse.ArgumentParser(
        description="Run a hash-guesser sweep and record its noise budget")
    ap.add_argument("-o", "--out-dir", default="hash-guesser-out", type=pathlib.Path,
                    help="where to write raw runs, unions and MANIFEST.tsv")
    ap.add_argument("--merge-ref", action="append", default=[], metavar="REF",
                    help="git ref whose override tables also count as known; "
                         "repeatable")
    ap.add_argument("--tier", choices=("hi", "br"), action="append", default=[],
                    help="only run this tier; repeatable (default: both)")
    ap.add_argument("--table", choices=TABLES, action="append", default=[],
                    help="only this table; repeatable (default: both)")
    ap.add_argument("--suffix-top", type=int, default=12,
                    help="how many suffix families to anchor on (default 12)")
    ap.add_argument("--exe", help="path to the hash-guesser binary")
    ap.add_argument("--dumps", default="dumps", type=pathlib.Path)
    args = ap.parse_args()

    exe = find_exe(args.exe)
    out = (ROOT / args.out_dir).resolve()
    raw = out / "raw"
    raw.mkdir(parents=True, exist_ok=True)
    work = out / ".corpus"
    tiers = set(args.tier) or {"hi", "br"}
    tables = args.table or list(TABLES)

    print(f"[..] guesser {exe}")
    corpus = build_corpus(work, args.merge_ref)
    if args.merge_ref:
        print(f"[..] corpus merged with {', '.join(args.merge_ref)}")

    wordlist = work / "words.txt"
    subprocess.run(
        [sys.executable, str(ROOT / "scripts" / "split_words.py"),
         str(corpus / "hashes.bintypes.txt"),
         str(corpus / "hashes.binfields.txt"),
         str(corpus / "overrides" / "bintypes.txt"),
         str(corpus / "overrides" / "binfields.txt"),
         "-o", str(wordlist)], check=True)

    manifest = []
    for table in tables:
        suffixes = work / f"suffixes.{table}.txt"
        top = write_suffixes(corpus, table, suffixes, args.suffix_top)
        print(f"[..] {table} tails: "
              + ", ".join(f"{w} {k}" for w, k in top[:6]) + " ...")

        for tier, name, extra in RUNS:
            if tier not in tiers:
                continue
            dest = raw / f"{name}.{table}.txt"
            cmd = [str(exe), table, "--hashes", str(corpus),
                   "--dumps", str(ROOT / args.dumps), "-o", str(dest)]
            if any(a is SUFFIXES for a in extra) or "identity" not in extra:
                cmd += ["--words", str(wordlist)]
            for a in extra:
                cmd += ["--suffix-list", str(suffixes)] if a is SUFFIXES else [a]
            log = subprocess.run(cmd, capture_output=True, text=True).stderr

            def grab(rx, default=""):
                m = next(filter(None, (rx.search(l) for l in log.splitlines())),
                         None)
                return m.group(1) if m else default

            hits = [l for l in dest.read_text(encoding="utf-8").splitlines()
                    if l.strip()] if dest.exists() else []
            row = {
                "tier": tier, "run": name, "table": table,
                "probes": grab(RE_PROBES, "0"), "states": grab(RE_STATES, "0"),
                "expected_fp": grab(RE_FP, "0"), "hits": str(len(hits)),
                "hashes": str(len({l.split()[0] for l in hits})),
            }
            manifest.append(row)
            print(f"  {tier:<3} {name:<14} {table:<9} "
                  f"{row['probes']:>12} probes  {row['expected_fp']:>8} fp  "
                  f"{row['hits']:>5} hits")

    with (out / "MANIFEST.tsv").open("w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, delimiter="\t", lineterminator="\n",
                           fieldnames=["tier", "run", "table", "probes",
                                       "states", "expected_fp", "hits",
                                       "hashes"])
        w.writeheader()
        w.writerows(manifest)

    # Unions. `highconf` is the quiet tier only; `candidates` is everything that
    # ran. Sorted by hash so a rerun diffs cleanly against the last one.
    for table in tables:
        for label, keep in (("highconf", HI_RUNS), ("candidates", None)):
            lines = set()
            for f in raw.glob(f"*.{table}.txt"):
                run = f.name[: -len(f".{table}.txt")]
                if keep is not None and run not in keep:
                    continue
                lines |= {l for l in f.read_text(encoding="utf-8").splitlines()
                          if l.strip()}
            (out / f"{label}.{table}.txt").write_text(
                "\n".join(sorted(lines)) + "\n", encoding="utf-8", newline="\n")
            print(f"[ok] {label}.{table}.txt: {len(lines)} line(s)")
    shutil.copy(wordlist, out / "words.txt")
    print(f"\n[ok] manifest at {out / 'MANIFEST.tsv'}")
    print("[..] build the review page with scripts/guesser_report.py")


if __name__ == "__main__":
    main()

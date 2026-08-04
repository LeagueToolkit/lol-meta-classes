#!/bin/env python
"""Context packs for the semantic pass, and the checker that closes its loop.

The guesser recombines known words, so it is blind to two things: words the
corpus has never used, and meaning. `UiMetricKills` sitting next to an unnamed
sibling is an invitation to guess `UiMetricDeaths`, but no wordlist encodes
that Kills and Deaths belong together - a person (or a model) does. This
script feeds that judgement and then verifies it, which is the whole loop:

    propose from context -> check the hash -> attest in shipped data

The economics are why this pass exists at all. Expected noise is
`probes x targets / 2^32`, and a proposed list is a few hundred probes against
a few thousand targets - arithmetic zero. Recombination cannot reach a name
containing a word the corpus has never used, however deep it searches; this can.

`emit` writes one pack per family: the base class, the named siblings (the
naming pattern), and each unnamed member with its fields, types, lifespan, and
the fields elsewhere that point at it. Everything a name can be judged against
except the hash itself, pulled from `db/meta.db.json`. Unresolved *fields* get
the same treatment grouped by declaring class.

`check` reads proposed names and reports what each one hits: an unresolved
class, an unresolved field, a name the tables already carry, or nothing. A
match is a candidate, not a crack - the README's rule that nothing reaches an
override table without attestation applies with full force, because a name
this pass proposes is exactly the kind of plausible that collides.

Usage:
    python3 scripts/guesser_families.py emit
    python3 scripts/guesser_families.py check proposals.txt
    ... check --matches-out only.txt   # feed the hits to --only, or review
"""

import argparse
import collections
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import names as names_mod  # noqa: E402
from hashtool import fnv1a_32, load_tables  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
TABLES = ("bintypes", "binfields")
# Caps keep a pack readable; every elision says what it dropped.
SIBLING_CAP = 24
FIELD_CAP = 20
REF_CAP = 8


def load_db(db_path):
    db = json.loads(db_path.read_text(encoding="utf-8"))
    classes = db["classes"]
    ext = db.get("externalTypeNames", {})
    patch = {v["build"]: v["patch"] for v in db["versions"]}

    def nameof(h):
        c = classes.get(h)
        if c and c.get("name"):
            return c["name"]
        return ext.get(h, h)

    return db, classes, nameof, patch


def render_type(tup, nameof):
    """A property's type tuple, hashes resolved: `List Pointer VfxDriver`."""
    return " ".join(
        x if not str(x).startswith("0x") else nameof(x)
        for x in tup
        if x not in ("0x0", None)
    )


def lifespan(revisions, patch):
    first = patch.get(revisions[0].get("from"), "?")
    if "to" in revisions[-1]:
        return f"{first}..{patch.get(revisions[-1]['to'], '?')}"
    return f"{first}..live"


def field_name_map(classes):
    """Field hash -> name, from whichever class knows it. A field hash is one
    global name; a class that lost the name still shares it."""
    out = {}
    for c in classes.values():
        for fh, p in c.get("properties", {}).items():
            if p.get("name"):
                out.setdefault(fh, p["name"])
    return out


def build_indexes(classes):
    """children: base hash -> class hashes; refs: class hash -> (owner, field)
    pairs whose declared type mentions it."""
    children = collections.defaultdict(list)
    refs = collections.defaultdict(list)
    for ch, c in classes.items():
        bases = set()
        for r in c["revisions"]:
            bases.update(r.get("bases", []))
        for b in sorted(bases):
            children[b].append(ch)
        for fh, p in c.get("properties", {}).items():
            tup = (p.get("revisions") or [{}])[-1].get("type") or ()
            for x in tup:
                if str(x).startswith("0x") and x != "0x0":
                    refs[x].append((ch, fh))
    return children, refs


def flag_text(revisions):
    out = []
    if any(r.get("interface") for r in revisions):
        out.append("INTERFACE")
    if any(r.get("value") for r in revisions):
        out.append("value")
    return "  ".join(out)


def describe_class(ch, classes, nameof, patch, fnames, children, refs, lines):
    """One unnamed class, with everything a name for it can be judged by."""
    c = classes[ch]
    revs = c["revisions"]
    flags = flag_text(revs)
    lines.append(f"### `{ch}`  {lifespan(revs, patch)}" + (f"  {flags}" if flags else ""))
    props = c.get("properties", {})
    if props:
        shown = sorted(
            props.items(), key=lambda kv: (fnames.get(kv[0], "") == "", kv[0])
        )[:FIELD_CAP]
        for fh, p in shown:
            ty = render_type((p.get("revisions") or [{}])[-1].get("type") or (), nameof)
            lines.append(f"- .{fnames.get(fh, fh)}: {ty or '?'}")
        if len(props) > FIELD_CAP:
            lines.append(f"- ... {len(props) - FIELD_CAP} more field(s)")
    kids = [nameof(k) for k in children.get(ch, []) if classes[k].get("name")]
    if kids:
        lines.append(f"- derived classes: {', '.join(sorted(kids)[:REF_CAP])}")
    pointers = [
        f"{nameof(o)}.{fnames.get(f, f)}"
        for o, f in refs.get(ch, [])
        if classes[o].get("name") or fnames.get(f)
    ]
    if pointers:
        lines.append(f"- referenced by: {', '.join(sorted(set(pointers))[:REF_CAP])}")
    lines.append("")


def emit(args):
    db, classes, nameof, patch = load_db(ROOT / args.db)
    fnames = field_name_map(classes)
    children, refs = build_indexes(classes)
    out = (ROOT / args.out_dir).resolve()
    (out / "types").mkdir(parents=True, exist_ok=True)
    (out / "fields").mkdir(parents=True, exist_ok=True)

    # Class families: unnamed classes grouped under their base. The base picks
    # the naming pattern and usually the first word; the siblings show both.
    fams = []
    for base, kids in children.items():
        unnamed = [k for k in kids if not classes[k].get("name")]
        named = [k for k in kids if classes[k].get("name")]
        if len(unnamed) >= args.min_unnamed:
            fams.append((base, unnamed, named))
    fams.sort(key=lambda f: (-len(f[1]), nameof(f[0])))

    index = ["# Semantic-pass context packs", ""]
    index.append(
        "Propose names from these, then `guesser_families.py check` them. A "
        "hash match is a candidate; only attestation in shipped data makes it "
        "a crack (README, \"Cracking unresolved hashes\")."
    )
    index.append("")
    index.append("## Class families (types/)")
    index.append("")
    for i, (base, unnamed, named) in enumerate(fams[: args.limit]):
        bname = nameof(base)
        slug = re.sub(r"[^A-Za-z0-9]", "", bname)[:40] or "Unknown"
        path = out / "types" / f"{i:02d}.{slug}.md"
        lines = [f"# Family: {bname} ({base})", ""]
        lines.append(f"- unnamed: {len(unnamed)}, named siblings: {len(named)}")
        if named:
            sibs = sorted(nameof(k) for k in named)
            lines.append(f"- siblings: {', '.join(sibs[:SIBLING_CAP])}")
            if len(sibs) > SIBLING_CAP:
                lines.append(f"  ... {len(sibs) - SIBLING_CAP} more")
        lines.append("")
        for ch in sorted(unnamed):
            describe_class(ch, classes, nameof, patch, fnames, children, refs, lines)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
        index.append(
            f"- [{bname}](types/{path.name}): {len(unnamed)} unnamed, "
            f"{len(named)} named sibling(s)"
        )
    if len(fams) > args.limit:
        index.append(f"- ... {len(fams) - args.limit} smaller famil(ies) not emitted")

    # Unresolved fields, grouped by the class that declares them. The class
    # name, the named fields beside them, and the declared type are the
    # context; a field named like its type is the common case.
    holders = []
    for ch, c in classes.items():
        missing = [fh for fh in c.get("properties", {}) if fh not in fnames]
        if missing and c.get("name"):
            holders.append((ch, missing))
    holders.sort(key=lambda h: (-len(h[1]), nameof(h[0])))
    index.append("")
    index.append("## Unresolved fields (fields/)")
    index.append("")
    for i, (ch, missing) in enumerate(holders[: args.limit]):
        cname = nameof(ch)
        slug = re.sub(r"[^A-Za-z0-9]", "", cname)[:40] or "Unknown"
        path = out / "fields" / f"{i:02d}.{slug}.md"
        c = classes[ch]
        lines = [f"# Fields of {cname} ({ch})  {lifespan(c['revisions'], patch)}", ""]
        named_here = sorted(
            (fnames[fh], fh, p) for fh, p in c["properties"].items() if fh in fnames
        )
        for n, fh, p in named_here[:FIELD_CAP]:
            ty = render_type((p.get("revisions") or [{}])[-1].get("type") or (), nameof)
            lines.append(f"- .{n}: {ty or '?'}")
        lines.append("")
        lines.append("Unresolved:")
        for fh in sorted(missing):
            p = c["properties"][fh]
            ty = render_type((p.get("revisions") or [{}])[-1].get("type") or (), nameof)
            others = sorted(
                {
                    nameof(o)
                    for o, oc in classes.items()
                    if o != ch and fh in oc.get("properties", {}) and oc.get("name")
                }
            )
            line = f"- `{fh}`: {ty or '?'}"
            if others:
                line += f"  (also on {', '.join(others[:REF_CAP])})"
            lines.append(line)
        path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
        index.append(f"- [{cname}](fields/{path.name}): {len(missing)} unresolved")
    if len(holders) > args.limit:
        index.append(f"- ... {len(holders) - args.limit} more class(es) not emitted")

    (out / "INDEX.md").write_text("\n".join(index) + "\n", encoding="utf-8",
                                  newline="\n")
    print(f"[ok] {min(len(fams), args.limit)} family pack(s), "
          f"{min(len(holders), args.limit)} field pack(s) -> {out}")
    print(f"[..] start at {out / 'INDEX.md'}")


def unresolved_sets(classes, fnames):
    """(class hashes, field hashes) with no name anywhere, as ints."""
    types = {int(h, 16) for h, c in classes.items() if not c.get("name")}
    fields = set()
    for c in classes.values():
        for fh in c.get("properties", {}):
            if fh not in fnames:
                fields.add(int(fh, 16))
    return types, fields


def check(args):
    _, classes, nameof, patch = load_db(ROOT / args.db)
    fnames = field_name_map(classes)
    types, fields = unresolved_sets(classes, fnames)
    # Overrides first: `hashes.<table>.txt` is only rebuilt by update_hashes.py,
    # so between an `add` and that rebuild it is the override layer that knows a
    # name. Reading only the merged file reports our own fresh cracks as misses.
    merged = {}
    for t in TABLES:
        ovr, m = load_tables(str(ROOT / "hashes"), t)
        for src in (ovr, m):
            for h, n in src.items():
                merged.setdefault(int(h, 16), n)

    proposals = []
    files = args.files or ["-"]
    for f in files:
        text = sys.stdin.read() if f == "-" else pathlib.Path(f).read_text(
            encoding="utf-8", errors="replace")
        for line in text.splitlines():
            line = line.split("#", 1)[0].strip()
            if not line:
                continue
            # `Name` alone, or `hash Name` as the guesser and the tables write.
            parts = line.split()
            proposals.append(parts[1] if len(parts) > 1 and re.fullmatch(
                r"(0x)?[0-9a-fA-F]{1,8}", parts[0]) else parts[0])

    counts = collections.Counter()
    matches = []
    # Spellings of the same name, keyed case-folded. The hash cannot separate
    # them - it lowercases - so a generator that emits both `TFTFoo` and `TftFoo`
    # gets one hit and the reported casing is decided by input order, which is
    # no evidence at all. Collect them instead and say so: the choice belongs to
    # whatever attests the name, or failing that to the house majority.
    spellings = collections.defaultdict(list)
    for name in proposals:
        spellings[name.lower()].append(name)
    seen = set()
    for name in proposals:
        if name.lower() in seen:
            continue
        seen.add(name.lower())
        alts = [s for s in dict.fromkeys(spellings[name.lower()]) if s != name]
        if not names_mod.is_valid_name(name):
            print(f"REJECT   {name}  ({names_mod.why_invalid(name)})")
            counts["rejected"] += 1
            continue
        h = fnv1a_32(name)
        key = f"0x{h:x}"
        if h in merged:
            verdict = "same casing" if merged[h].lower() == name.lower() else \
                f"tables say {merged[h]}"
            print(f"KNOWN    {h:08x} {name}  ({verdict})")
            counts["already named"] += 1
        elif h in types:
            c = classes[key]
            bases = sorted({b for r in c["revisions"] for b in r.get("bases", [])})
            ctx = f"base {', '.join(nameof(b) for b in bases)}" if bases else "no base"
            flags = flag_text(c["revisions"])
            print(f"TYPE     {h:08x} {name}  ({ctx}; "
                  f"{lifespan(c['revisions'], patch)}"
                  + (f"; {flags}" if flags else "") + ")")
            counts["unresolved type"] += 1
            matches.append((h, name))
        elif h in fields:
            owners = sorted(
                nameof(ch) for ch, c in classes.items()
                if key in c.get("properties", {}) and c.get("name"))
            ctx = f"on {', '.join(owners[:4])}" if owners else "unnamed owners only"
            print(f"FIELD    {h:08x} {name}  ({ctx})")
            counts["unresolved field"] += 1
            matches.append((h, name))
        else:
            print(f"miss     {h:08x} {name}")
            counts["no match"] += 1
            continue
        if alts:
            counts["casing undecided"] += 1
            print(f"         ^ casing NOT decided by this hit - also proposed "
                  f"{', '.join(alts)}")

    total = sum(counts.values()) - counts["casing undecided"]
    print(f"[ok] {total} proposal(s): "
          + ", ".join(f"{v} {k}" for k, v in counts.most_common()
                      if k != "casing undecided"))
    if matches:
        print("[note] a hash match is a candidate, not a crack - attest each "
              "against shipped data before `hashtool add`")
    if counts["casing undecided"]:
        print(f"[warn] {counts['casing undecided']} hit(s) had more than one "
              f"spelling proposed. The hash cannot choose between them; pick the "
              f"one something attests, or the house majority.")
    if args.matches_out and matches:
        pathlib.Path(args.matches_out).write_text(
            "".join(f"{h:08x} {n}\n" for h, n in sorted(matches)),
            encoding="utf-8", newline="\n")
        print(f"[ok] {len(matches)} match(es) -> {args.matches_out}")


def main():
    ap = argparse.ArgumentParser(
        description="Emit semantic-pass context packs, and check proposals")
    sub = ap.add_subparsers(dest="cmd", required=True)

    e = sub.add_parser("emit", help="write per-family context packs")
    e.add_argument("-o", "--out-dir", default="hash-guesser-out/families",
                   help="pack directory (default hash-guesser-out/families)")
    e.add_argument("--db", default="db/meta.db.json", type=pathlib.Path)
    e.add_argument("--limit", type=int, default=80,
                   help="most packs per section (default 80)")
    e.add_argument("--min-unnamed", type=int, default=1,
                   help="skip families with fewer unnamed members")
    e.set_defaults(fn=emit)

    c = sub.add_parser("check", help="verify proposed names against the meta")
    c.add_argument("files", nargs="*",
                   help="files of names, `Name` or `hash Name` per line; "
                        "stdin when none")
    c.add_argument("--db", default="db/meta.db.json", type=pathlib.Path)
    c.add_argument("--matches-out", metavar="FILE",
                   help="also write the hits as `hash name` lines")
    c.set_defaults(fn=check)

    args = ap.parse_args()
    args.fn(args)


if __name__ == "__main__":
    main()

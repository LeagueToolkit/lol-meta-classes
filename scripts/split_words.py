#!/bin/env python
"""Names in, wordlist out - the input the guesser recombines.

A rewrite of LeagueHashes' split_words.py. Cracking a hash is guessing that its
name is some arrangement of words that already appear in names we know, so the
quality of this output sets the ceiling on what `hash-guesser` can find.

Reads hashtables (`{hash} {name}`) and plain name lists alike, splits each name
into words with the shared splitter in names.py, and prints the deduplicated
result. Frequency-ordered by default: a word seen in 200 names is a better bet
than one seen once, and the guesser tries the list in order.

What it does that the original didn't:

  - recovers the leading word of a camelCase name. The original skipped every
    leading lowercase character, so `abilityHaste` gave up "Ability" and
    upstream's binfields table - thousands of camelCase names - contributed only
    its tails. This is also why names *we* add are PascalCase by rule; see
    names.py.
  - keeps acronyms whole. `AIGenericCommon` is AI + Generic + Common, not
    A + I + Generic + Common.
  - splits on separators rather than tripping over them, so `Obj_InfoPoint`
    contributes Obj and InfoPoint.
  - still drops the Hungarian prefix (the `m` of mCoefficient), because "M" as
    a word matches everything and means nothing. `--keep-prefix` turns that off.

Usage:
    # the wordlist the guesser wants: every name the repo knows
    python3 scripts/split_words.py hashes/hashes.*.txt hashes/overrides/*.txt > words.txt

    # how productive is each word, and where does the tail start
    python3 scripts/split_words.py --count hashes/hashes.bintypes.txt | head -50

    # check what the splitter does to a name
    echo GameEntityPrefab | python3 scripts/split_words.py --sentences -
"""

import argparse
import glob
import re
import sys
from collections import Counter

import names

# `{hash} {name}`, the hashtable line format. Anything else on a line is taken
# as a bare name, so plain lists and tables can be mixed in one invocation
# without a flag saying which is which.
RE_TABLE_LINE = re.compile(r"^[0-9a-fA-F]{1,16} (\S+)$")


def name_of(line):
    """One input line -> the name on it, or None if there isn't one."""
    line = line.strip()
    if not line or line.startswith("#"):
        return None
    m = RE_TABLE_LINE.match(line)
    if m:
        return m.group(1)
    # A bare list may still be bulleted or indented; a line with interior
    # whitespace that isn't a table line is not a name.
    line = line.lstrip("-*").strip()
    return line if line and " " not in line else None


def iter_names(paths):
    """Names from every path, `-` meaning stdin. Globs are expanded here so the
    tool behaves the same under a shell that doesn't expand them (cmd.exe) as
    under one that does."""
    for arg in paths:
        if arg == "-":
            for line in sys.stdin:
                yield line
            continue
        matched = glob.glob(arg, recursive=True)
        if not matched:
            print(f"[warn] no files match {arg!r}", file=sys.stderr)
        for path in sorted(matched):
            with open(path, encoding="utf-8", errors="replace") as f:
                yield from f


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("paths", nargs="+", metavar="FILE",
                   help="hashtables or name lists; globs allowed, `-` is stdin")
    p.add_argument("--sort", choices=("freq", "alpha", "seen"), default="freq",
                   help="freq: most productive first, which is the order the "
                        "guesser should try (default); alpha: diffable; "
                        "seen: first-appearance, as the original tool printed")
    p.add_argument("--count", action="store_true",
                   help="prefix each word with how many names it came from")
    p.add_argument("--min-len", type=int, default=1, metavar="N",
                   help="drop words shorter than N characters (default 1)")
    p.add_argument("--min-freq", type=int, default=1, metavar="N",
                   help="drop words seen in fewer than N names (default 1)")
    p.add_argument("--stems", action="store_true",
                   help="also emit the digit-stripped form of a word, so "
                        "Detection2 contributes Detection as well")
    p.add_argument("--keep-prefix", action="store_true",
                   help="keep the leading Hungarian prefix (the `m` of "
                        "mCoefficient) instead of dropping it")
    p.add_argument("--no-fold-case", dest="fold_case", action="store_false",
                   help="keep every spelling of a word instead of collapsing "
                        "Vfx/VFX onto the commoner one")
    p.add_argument("--sentences", action="store_true",
                   help="print each name's words on one line instead of "
                        "building a wordlist - for checking the splitter")
    p.add_argument("-o", "--out", metavar="FILE", default=None,
                   help="write here instead of stdout")
    args = p.parse_args()

    prefix_max = 0 if args.keep_prefix else names.PREFIX_MAX
    counts = Counter()
    order = {}
    lines = []
    total = 0

    for raw in iter_names(args.paths):
        name = name_of(raw)
        if name is None:
            continue
        total += 1
        words = names.split_words(name, prefix_max=prefix_max)
        if args.sentences:
            lines.append(" ".join(words))
            continue
        # Counted once per name, not once per occurrence: a word repeated
        # inside one name ("PointToPoint") is one piece of evidence that it is
        # a real word, not two.
        for word in set(words):
            if args.stems:
                s = names.stem(word)
                if s:
                    counts[s] += 1
                    order.setdefault(s, len(order))
            counts[word] += 1
            order.setdefault(word, len(order))

    if not args.sentences and args.fold_case:
        # Two spellings of one word are two identical guesses: the bin-hash is
        # FNV over the lowercased name, so `VFXSystem` and `VfxSystem` are the
        # same hash and the second is pure waste - and in force mode the waste
        # is compounded, once per position. Keep the commoner spelling and give
        # it the combined weight; ties go to the alphabetically first so the
        # choice is deterministic.
        folded, best = Counter(), {}
        for word, count in counts.items():
            key = word.lower()
            folded[key] += count
            if key not in best or (-count, word) < (-counts[best[key]], best[key]):
                best[key] = word
        counts = Counter({best[k]: c for k, c in folded.items()})
        order = {w: order[w] for w in counts}

    if not args.sentences:
        keep = [w for w, c in counts.items()
                if len(w) >= args.min_len and c >= args.min_freq]
        if args.sort == "freq":
            keep.sort(key=lambda w: (-counts[w], w))
        elif args.sort == "alpha":
            keep.sort()
        else:
            keep.sort(key=lambda w: order[w])
        lines = [f"{counts[w]:>6} {w}" if args.count else w for w in keep]

    out = open(args.out, "w", encoding="utf-8", newline="\n") if args.out else sys.stdout
    try:
        for line in lines:
            print(line, file=out)
    finally:
        if args.out:
            out.close()

    what = "sentence(s)" if args.sentences else "word(s)"
    print(f"[ok] {total} name(s) -> {len(lines)} {what}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())

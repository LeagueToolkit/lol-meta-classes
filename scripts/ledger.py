#!/bin/env python
"""The crack ledger: hashes/overrides/ledger.tsv.

The override tables say *what* a hash resolves to, not when it was cracked or
whether it has been sent upstream - so a crack can sit locally for months and be
forgotten. This is that bookkeeping, in a sibling TSV so the override tables stay
byte-comparable with upstream's format.

    hash  table  name  batch  cracked  status  pr

Rows are keyed by (table, hash), not hash alone: six names are both a class and a
field and so share a hash.

`batch` is the campaign a crack came out of, and the unit an upstream PR is built
from. Rows are written grouped by it, and it is where the method and attestation
live - one `#:` note per batch in the header, rather than a paragraph repeated on
every row. Anything per-name belongs in the reversing doc that note points at.

`status` also carries the one thing the override tables cannot: that a name is
resolved here on purpose and is *not* to be sent upstream (see LOCAL).
"""

import datetime
import os
import re
import subprocess

TABLES = ("bintypes", "binfields")
COLUMNS = ("hash", "table", "name", "batch", "cracked", "status", "pr")

# pending   - cracked locally, not sent anywhere yet
# submitted - in an open upstream PR (`pr` should carry the link)
# merged    - upstream has it; `prune` will drop the override, the row stays
# local     - deliberately not going upstream (see LOCAL)
STATUSES = ("pending", "submitted", "merged", "local")

# Not a stage of the submission lifecycle but an exit from it: a name we resolve
# here and do not intend to publish upstream - unreleased-content names that
# would leak by appearing in CDragon, and cracks kept back for any other reason.
# Terminal like `merged` in the only sense the backlog cares about: `--list
# pending` is "what still needs a CDragon PR", and these never will.
LOCAL = "local"

# Column headers for the per-batch summary; keyed by status so the two can't drift.
ABBREV = {"pending": "pend", "submitted": "subm", "merged": "merg", "local": "local"}

RE_HASH = re.compile(r"^[0-9a-f]{8}$")
RE_SLUG = re.compile(r"^[a-z0-9][a-z0-9.-]*$")
NO_PR = "-"

# Where a crack lands when no campaign was given. Sorts last: it's the
# "attribute me" pile, not a resting place.
UNSORTED = "unsorted"
WORKING_TREE = "working-tree"  # blame slug for uncommitted override lines

BATCH_PREFIX = "#: "
HEADER = "# Crack ledger, grouped by batch. See hashes/overrides/README.md."


class Ledger:
    """rows: {(table, hash): row}; batches: {slug: note}."""

    def __init__(self, rows=None, batches=None):
        self.rows = rows or {}
        self.batches = batches or {}

    def of_batch(self, slug):
        return [r for r in self.rows.values() if r["batch"] == slug]

    def order(self):
        """Batch slugs in file order: oldest campaign first, UNSORTED last. A
        batch that has lost every row is dropped unless it still has a note."""
        slugs = {r["batch"] for r in self.rows.values()}
        slugs |= {s for s, note in self.batches.items() if note}
        dated = sorted(s for s in slugs if s != UNSORTED)
        dated.sort(key=lambda s: min((r["cracked"] for r in self.of_batch(s)),
                                     default="9999-99-99"))
        return dated + ([UNSORTED] if UNSORTED in slugs else [])

    def counts(self, slug=None):
        rows = self.of_batch(slug) if slug is not None else self.rows.values()
        out = {s: 0 for s in STATUSES}
        for r in rows:
            out[r["status"]] += 1
        return out


def ledger_path(hashes_dir):
    return os.path.join(hashes_dir, "overrides", "ledger.tsv")


def _clean(value):
    """TSV has no quoting, so tabs and newlines can't survive in a field."""
    return " ".join(str(value).split())


def slugify(text, fallback=UNSORTED):
    """Commit subject -> batch slug; the conventional-commit prefix is noise
    here ("feat: add game entity hashes" is the *hashes*, not the feat)."""
    text = re.sub(r"^\w+(\([^)]*\))?!?:\s*", "", text.strip())
    slug = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    slug = "-".join(slug.split("-")[:5])[:48].strip("-")
    return slug if RE_SLUG.match(slug or "") else fallback


def check_slug(slug):
    if not RE_SLUG.match(slug or ""):
        raise ValueError(f"bad batch slug {slug!r} - lowercase letters, digits, "
                         f"'-' and '.', starting with a letter or digit")
    return slug


def load(path):
    """Path -> Ledger. Missing file is an empty ledger; a row missing the
    trailing columns of a newer format lands in UNSORTED / pending."""
    if not os.path.exists(path):
        return Ledger()
    rows, batches = {}, {}
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.rstrip("\n").rstrip("\r")
            if not line:
                continue
            if line.startswith(BATCH_PREFIX):
                slug, _, note = line[len(BATCH_PREFIX):].partition("\t")
                batches[check_slug(slug.strip())] = note.strip()
                continue
            if line.startswith("#"):  # header or group heading
                continue
            parts = line.split("\t")
            if parts[0] == "hash":  # column header
                continue
            row = dict(zip(COLUMNS, parts))
            if len(parts) < len(COLUMNS):
                raise ValueError(
                    f"{path}:{lineno}: expected {len(COLUMNS)} tab-separated "
                    f"fields, got {len(parts)}: {line!r}")
            if not RE_HASH.match(row["hash"]):
                raise ValueError(f"{path}:{lineno}: bad hash {row['hash']!r}")
            if row["table"] not in TABLES:
                raise ValueError(f"{path}:{lineno}: bad table {row['table']!r}")
            if row["status"] not in STATUSES:
                raise ValueError(f"{path}:{lineno}: bad status {row['status']!r}")
            row["batch"] = check_slug(row["batch"].strip() or UNSORTED)
            key = (row["table"], row["hash"])
            if key in rows:
                raise ValueError(f"{path}:{lineno}: duplicate {row['table']} {key[1]}")
            rows[key] = row
    return Ledger(rows, batches)


def key_of(row):
    return (row["table"], row["hash"])


def render(led):
    """Header notes, then rows grouped by batch. Within a batch: by name, then
    hash, then table (which breaks the tie for a name that is class and field),
    so a row still lands next to nothing else when added."""
    out = [HEADER]
    for slug in led.order():
        out.append(f"{BATCH_PREFIX}{slug}\t{_clean(led.batches.get(slug, ''))}")
    out += ["", "\t".join(COLUMNS)]
    for slug in led.order():
        out += ["", f"# {slug}"]
        for row in sorted(led.of_batch(slug),
                          key=lambda r: (r["name"], r["hash"], r["table"])):
            out.append("\t".join(_clean(row[c]) for c in COLUMNS))
    return "\n".join(out) + "\n"


def make_row(h, table, name, cracked, batch=UNSORTED, status="pending", pr=NO_PR):
    return {"hash": h, "table": table, "name": name, "batch": batch or UNSORTED,
            "cracked": cracked, "status": status, "pr": pr or NO_PR}


def blame_lines(repo_root, rel_path):
    """{line_text: (YYYY-MM-DD, batch_slug)} from `git blame` of an override
    table. The commit that introduced a line is the best record of both when a
    name was cracked and which sitting cracked it, and the only attribution
    recoverable after the fact. {} if git isn't available or the file is
    untracked - the caller falls back to a default rather than failing."""
    try:
        out = subprocess.run(
            ["git", "blame", "--line-porcelain", "--", rel_path],
            cwd=repo_root, capture_output=True, text=True, check=True,
            encoding="utf-8", errors="replace",
        ).stdout
    except (OSError, subprocess.CalledProcessError):
        return {}

    info = {}
    author_time = summary = sha = None
    for line in out.splitlines():
        m = re.match(r"^([0-9a-f]{40}) \d+ \d+", line)
        if m:
            sha = m.group(1)
        elif line.startswith("author-time "):
            author_time = int(line.split()[1])
        elif line.startswith("summary "):
            summary = line[len("summary "):]
        elif line.startswith("\t"):
            date = "" if author_time is None else datetime.datetime.fromtimestamp(
                author_time, datetime.timezone.utc).strftime("%Y-%m-%d")
            # An all-zero sha is an uncommitted line: no commit subject to name
            # it after, and its batch is whatever the current sitting is.
            slug = WORKING_TREE if sha == "0" * 40 else slugify(summary or "")
            info[line[1:].strip()] = (date, slug)
            author_time = summary = sha = None
    return info

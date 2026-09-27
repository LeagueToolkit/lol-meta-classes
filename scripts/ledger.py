#!/bin/env python
"""The crack ledger, in hashes/overrides/.

The override tables say *what* a hash resolves to, not when it was cracked or
whether it has been sent upstream - so a crack can sit locally for months and be
forgotten. This is that bookkeeping, in sibling JSONL files so the override
tables stay byte-comparable with upstream's format.

    ledger.bintypes.jsonl    {hash, name, batch, cracked, status, pr}
    ledger.binfields.jsonl   {hash, name, batch, cracked, status, pr}
    batches.jsonl            {batch, note}

One ledger per table, named after the override table it shadows, so the pair
moves together and a diff is confined to the table that changed. There is no
`table` key: the filename is it. In memory the two are one Ledger keyed by
(table, hash), not by hash alone - six names are both a class and a field and so
share a hash, which is exactly what the split file names disambiguate.

`batch` is the campaign a crack came out of, and the unit an upstream PR is built
from. Rows are written grouped by it, and it is where the method and attestation
live - one note per batch, in batches.jsonl, rather than a paragraph repeated on
every row. Anything per-name belongs in the reversing doc that note points at.
Batches cross tables, so batches.jsonl stays single.

Every line is one JSON object with exactly the keys in COLUMNS (BATCH_COLUMNS),
all strings, written in that key order. No blank lines, no comments - JSON has
none - and no header, since each object names its own fields. load() enforces
all of it rather than leaving it as a convention to erode.

`status` also carries the one thing the override tables cannot: that a name is
resolved here on purpose and is *not* to be sent upstream (see LOCAL).

Row names obey the repo naming rule (names.py): PascalCase, bar a single leading
lowercase letter, checked here on load rather than only at `add`, so a
hand-edited row is caught by the next command that touches the file.
"""

import datetime
import json
import os
import re
import subprocess

import names as names_mod

TABLES = ("bintypes", "binfields")
COLUMNS = ("hash", "name", "batch", "cracked", "status", "pr")
BATCH_COLUMNS = ("batch", "note")

# A batch note is an abstract, not the write-up. It says what the campaign was
# and what makes its names believable, in one line someone can read in a diff;
# the derivation, the per-name evidence and the tables go in a doc under docs/
# that the note points at. The limit is what keeps that split honest - without
# it the note grows into the doc, one justified sentence at a time, and the
# file stops being scannable as a table at all.
NOTE_MAX = 200

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


def ledger_path(hashes_dir, table):
    return os.path.join(hashes_dir, "overrides", f"ledger.{table}.jsonl")


def batches_path(hashes_dir):
    return os.path.join(hashes_dir, "overrides", "batches.jsonl")


def _line(values, columns):
    """One rendered line: the columns in order, as a single JSON object.

    Whitespace runs collapse to one space, so a value stays one line in a diff
    even if it arrived with a newline in it. ensure_ascii=False keeps any
    non-ASCII in a note readable rather than \\u-escaped."""
    return json.dumps({c: " ".join(str(values[c]).split()) for c in columns},
                      ensure_ascii=False)


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


def _rows_of(path, columns):
    """Lines of a JSONL file -> [(lineno, {column: value})].

    Each line must be one JSON object carrying exactly `columns`, every value a
    string. Anything else is an error naming the line, not a row skipped."""
    if not os.path.exists(path):
        return []
    out = []
    with open(path, encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.rstrip("\n").rstrip("\r")
            if not line.strip():
                raise ValueError(f"{path}:{lineno}: blank line")
            try:
                obj = json.loads(line)
            except json.JSONDecodeError as e:
                raise ValueError(f"{path}:{lineno}: not JSON ({e.msg}): "
                                 f"{line!r}") from None
            if not isinstance(obj, dict) or set(obj) != set(columns):
                got = sorted(obj) if isinstance(obj, dict) else type(obj).__name__
                raise ValueError(f"{path}:{lineno}: expected an object with keys "
                                 f"{list(columns)}, got {got}")
            bad = [c for c in columns if not isinstance(obj[c], str)]
            if bad:
                raise ValueError(f"{path}:{lineno}: {bad} must be strings")
            out.append((lineno, {c: obj[c].strip() for c in columns}))
    return out


def load(hashes_dir, strict=True):
    """hashes/ -> Ledger: both per-table ledgers plus the shared batch notes,
    merged into one keyed set of rows. Missing files are an empty ledger.

    `table` is reattached to each row from the file it came from, so everything
    downstream sees the same row shape the single-file ledger had.

    strict=False skips the naming-rule check, and exists for exactly one caller:
    `hashtool lint --fix`, which has to read a file full of offending names in
    order to repair them. Everything else wants the check."""
    rows = {}
    for table in TABLES:
        path = ledger_path(hashes_dir, table)
        for lineno, cells in _rows_of(path, COLUMNS):
            row = dict(cells, table=table)
            if not RE_HASH.match(row["hash"]):
                raise ValueError(f"{path}:{lineno}: bad hash {row['hash']!r}")
            if row["status"] not in STATUSES:
                raise ValueError(f"{path}:{lineno}: bad status {row['status']!r}")
            # The naming rule is checked on the way in, not only at `add`, so a
            # hand-edited row cannot smuggle a camelCase name past it. See
            # names.py: this is what keeps the wordlist worth building.
            if strict:
                try:
                    names_mod.check_name(row["name"])
                except ValueError as e:
                    raise ValueError(
                        f"{path}:{lineno}: {e}\n"
                        f"  repair the file with `python3 scripts/hashtool.py "
                        f"lint --fix`") from None
            row["batch"] = check_slug(row["batch"].strip() or UNSORTED)
            key = (table, row["hash"])
            if key in rows:
                raise ValueError(f"{path}:{lineno}: duplicate {table} {key[1]}")
            rows[key] = row

    b_path = batches_path(hashes_dir)
    batches = {}
    for lineno, cells in _rows_of(b_path, BATCH_COLUMNS):
        slug = check_slug(cells["batch"])
        if slug in batches:
            raise ValueError(f"{b_path}:{lineno}: duplicate batch {slug}")
        batches[slug] = cells["note"]
    return Ledger(rows, batches)


def key_of(row):
    return (row["table"], row["hash"])


def render(led, table):
    """One table's ledger: its rows grouped by batch, and within a batch by
    name then hash, so a row lands next to nothing else when added.

    The grouping has no marker in the file - the `batch` key carries it. It
    survives as row order, which is what keeps a diff local to the campaign
    that changed.

    Batch order is the ledger's, not this table's, so the two files stay in step
    with each other and with batches.jsonl; a batch with no rows in this table
    simply contributes none."""
    out = []
    for slug in led.order():
        rows = [r for r in led.of_batch(slug) if r["table"] == table]
        for row in sorted(rows, key=lambda r: (r["name"], r["hash"])):
            out.append(_line(row, COLUMNS) + "\n")
    return "".join(out)


def render_batches(led):
    """batches.jsonl: one row per campaign, in the same order as the ledger. A
    batch with no note yet still gets a row - the gap is the point, it's what
    `add` nags about."""
    return "".join(
        _line({"batch": slug, "note": led.batches.get(slug, "")},
              BATCH_COLUMNS) + "\n"
        for slug in led.order())


def save(hashes_dir, led, write):
    """Write every part: one ledger per table, plus batches.jsonl. `write` is
    update_hashes.write_if_changed - passed in rather than imported, so this
    module stays free of the hashtable pipeline. Returns True if any file
    changed; every file is written either way, so a split can't half-apply."""
    changed = [write(ledger_path(hashes_dir, t), render(led, t)) for t in TABLES]
    changed.append(write(batches_path(hashes_dir), render_batches(led)))
    return any(changed)


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

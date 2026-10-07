#!/usr/bin/env python3
"""Build a versioned meta class database from all dumps.

Folds every dump in dumps/ (in build-number order) into db/meta.db.json,
an interval-based history of every class and property:

  - Each class/property has a list of *revisions*. A revision is one distinct
    definition (bases for classes, type tuple for properties) together with
    the range of builds it was observed in: {"from": <build>, "to": <build>}.
  - A revision without "to" is current (present in the latest build).
  - An entity whose last revision has "to" set has been removed from the game.
  - A type change shows up as adjacent revisions instead of ambiguous
    duplicate entries.

Everything is keyed by FNV-1a hash; resolved names (from hashes/) are attached
as metadata so improving hash lists never creates false history. The whole
file is rebuilt deterministically from dumps/ on every run - there is no
incremental state to corrupt.

Also regenerates db/database.py as a snapshot of the *latest* build only
(the previous behaviour of importing dumps into the existing file made it an
unversioned aggregate of everything that ever existed).

Also writes db/meta.pbe.json, the PBE overlay. If dumps/pbe/ holds a PBE dump
whose patch is greater than the latest live patch, the PBE dump is folded as
one more build on top of the live history and the class entries that differ are
written to the overlay. Otherwise the overlay states that no preview exists.
db/meta.db.json and db/database.py never include the PBE build.

See docs/meta-db-format.md for the full format description.

Usage:
    python3 scripts/db_build.py
    python3 scripts/db_build.py --dumps dumps --hashes hashes \
        --out db/meta.db.json --py db/database.py
    python3 scripts/db_build.py --no-preview
"""

import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import db_import
from db_import import read_resolved_hashes, read_meta, rehex_fnv1a

FORMAT_VERSION = 1
RE_DUMP = re.compile(r"^(\d+)\.(\d+)\.(\d+)\.json$")
PREVIEW_CHANNEL = "pbe"
PREVIEW_FILE = "meta.pbe.json"
# The hasher of every `Hash` property before 16.17: (width, algorithm, lowercased).
DEFAULT_HASHER = (4, "Fnv1a32", True)


def hash_key(h):
    return int(h, 16)


def list_dumps(dumps_dir):
    """Returns the dumps directly inside `dumps_dir`, unordered. The listing is
    not recursive, so dumps/pbe/ does not enter the live corpus."""
    dumps = []
    for filename in os.listdir(dumps_dir):
        m = RE_DUMP.match(filename)
        if not m:
            continue
        major, minor, build = (int(g) for g in m.groups())
        dumps.append({
            "patch": f"{major}.{minor}",
            "build": build,
            "path": os.path.join(dumps_dir, filename),
            "_mm": (major, minor),
        })
    return dumps


def discover_dumps(dumps_dir):
    dumps = list_dumps(dumps_dir)
    # Build numbers increase monotonically across patches and are the only
    # reliable global ordering ("13.2" vs "13.15" breaks lexicographic sorts).
    dumps.sort(key=lambda d: d["build"])
    prev = None
    for d in dumps:
        if prev and d["_mm"] < prev["_mm"]:
            print(f"[warn] build order disagrees with patch order: "
                  f"{d['patch']}.{d['build']} after {prev['patch']}.{prev['build']}",
                  file=sys.stderr)
        prev = d
    return dumps


def discover_preview(preview_dir, dumps):
    """Returns the PBE dump that the preview is built from, or None.

    Returns None if `preview_dir` does not exist or holds no dump. Returns None
    if the patch of the newest PBE dump is not greater than the latest live
    patch. If the directory holds several dumps, the dump with the greatest
    patch and build is used."""
    if not os.path.isdir(preview_dir):
        return None
    candidates = list_dumps(preview_dir)
    if not candidates:
        return None
    dump = max(candidates, key=lambda d: (d["_mm"], d["build"]))
    live = max(dumps, key=lambda d: d["_mm"])
    if dump["_mm"] <= live["_mm"]:
        print(f"[..] no preview: PBE {dump['patch']}.{dump['build']} is not ahead of "
              f"live patch {live['patch']}")
        return None
    return dump


def field_type_tuple(field):
    # Same 4-tuple as database.py fields: (ft, kt, vt, kh)
    if field["container"] and field["map"]:
        raise ValueError("container/map conflict!")
    ftype = [field["value_type"]]
    if field["container"]:
        ftype.append(hex(field["container"]["fixed_size"] or 0))
        ftype.append(field["container"]["value_type"])
    elif field["map"]:
        ftype.append(field["map"]["key_type"])
        ftype.append(field["map"]["value_type"])
    else:
        ftype.append(hex(0))
        ftype.append(hex(0))
    if field["other_class"]:
        ftype.append(rehex_fnv1a(field["other_class"]))
    else:
        ftype.append(hex(0))
    return tuple(ftype)


def field_hasher(field, hashers):
    """Returns the (width, algorithm, lowercased) of the hasher of a `Hash`
    property, or None.

    Returns None if the dump records no hasher for the property, which is every
    dump before format version 3. Returns None if the hasher is the default:
    FNV-1a 32 over the lowercased string, stored in 4 bytes."""
    key = field.get("hasher")
    if not key:
        return None
    hasher = hashers[key]
    function = hasher["hash_function"]
    sig = (hasher["storage_width"], function["algorithm"], function["lowercased"])
    return None if sig == DEFAULT_HASHER else sig


def class_signature(klass):
    bases = set()
    if klass.get("base"):
        bases.add(rehex_fnv1a(klass["base"]))
    for base in (klass.get("secondary_bases") or {}):
        bases.add(rehex_fnv1a(base))
    flags = klass.get("is") or {}
    return (tuple(sorted(bases, key=hash_key)),
            bool(flags.get("interface")), bool(flags.get("value")))


def advance(revisions, sig, build, prev_build):
    """Extend the open revision if the signature is unchanged and the entity
    was also present in the previous build; otherwise start a new revision."""
    if revisions:
        last = revisions[-1]
        if last["_sig"] == sig and last["_last"] == prev_build:
            last["_last"] = build
            return last
    rev = {"_sig": sig, "_from": build, "_last": build, "payload": {}}
    revisions.append(rev)
    return rev


def fold_dump(classes, d, prev_build):
    """Folds one dump into `classes` as the build that follows `prev_build`."""
    build = d["build"]
    meta = read_meta(d["path"])
    hashers = meta.get("hashers") or {}
    for kname, klass in meta["classes"].items():
        khash = rehex_fnv1a(kname)
        entry = classes.setdefault(khash, {"revisions": [], "properties": {}})
        sig = class_signature(klass)
        rev = advance(entry["revisions"], sig, build, prev_build)
        rev["payload"] = {
            "bases": list(sig[0]),
            "interface": sig[1],
            "value": sig[2],
        }
        defaults = klass.get("defaults")
        for fname, field in klass["properties"].items():
            fhash = rehex_fnv1a(fname)
            prop = entry["properties"].setdefault(fhash, {"revisions": []})
            tsig = field_type_tuple(field)
            hsig = field_hasher(field, hashers)
            frev = advance(prop["revisions"], (tsig, hsig), build, prev_build)
            frev["payload"]["type"] = list(tsig)
            if hsig:
                frev["payload"]["hasher"] = {
                    "width": hsig[0],
                    "algorithm": hsig[1],
                    "lowercased": hsig[2],
                }
            # Revisions are keyed on the type tuple and the hasher; the
            # default carried by a revision is the most recent one observed
            # within its range.
            if isinstance(defaults, dict) and fname in defaults:
                frev["payload"]["default"] = defaults[fname]


def build_history(dumps):
    classes = {}
    prev_build = None
    for d in dumps:
        fold_dump(classes, d, prev_build)
        prev_build = d["build"]
    return classes


def finalize(classes, latest_build, h_types, h_fields):
    def translate_default_keys(obj):
        # Nested default objects use field hashes as keys; resolve them for
        # display, like database.py does. Values are left untouched.
        if isinstance(obj, dict):
            return {h_fields.get(k, k): translate_default_keys(v) for k, v in obj.items()}
        if isinstance(obj, list):
            return [translate_default_keys(x) for x in obj]
        return obj

    def revs_out(revisions):
        out = []
        for rev in revisions:
            r = {"from": rev["_from"]}
            # "to" is only written for closed revisions; the open revision of a
            # currently-existing entity has none. This keeps unchanged entities
            # byte-identical between builds, so git diffs stay minimal.
            if rev["_last"] != latest_build:
                r["to"] = rev["_last"]
            r.update(rev["payload"])
            if "default" in r:
                r["default"] = translate_default_keys(r["default"])
            out.append(r)
        return out

    result = {}
    for khash in sorted(classes, key=hash_key):
        entry = classes[khash]
        klass = {}
        if khash in h_types:
            klass["name"] = h_types[khash]
        klass["revisions"] = revs_out(entry["revisions"])
        klass["properties"] = {}
        for fhash in sorted(entry["properties"], key=hash_key):
            prop = {}
            if fhash in h_fields:
                prop["name"] = h_fields[fhash]
            prop["revisions"] = revs_out(entry["properties"][fhash]["revisions"])
            klass["properties"][fhash] = prop
        result[khash] = klass
    return result


def external_type_names(classes_out, h_types):
    """Names for type hashes referenced in bases/type tuples but never dumped
    as classes themselves - consumers can't resolve those via classes[hash]."""
    refs = set()
    for klass in classes_out.values():
        for rev in klass["revisions"]:
            refs.update(rev["bases"])
        for prop in klass["properties"].values():
            for rev in prop["revisions"]:
                refs.add(rev["type"][3])
    return {h: h_types[h] for h in sorted(refs, key=hash_key)
            if h not in classes_out and h in h_types and h != "0x0"}


def is_open(entity):
    return "to" not in entity["revisions"][-1]


def build_preview(classes, live_out, live_external, base, dump, h_types, h_fields):
    """Builds the overlay for the PBE dump `dump`.

    Folds `dump` into `classes` as the build that follows the live build `base`
    and keeps the class entries that differ from `live_out`. `classes` is
    modified, so `live_out` must be finalized before the call."""
    fold_dump(classes, dump, base["build"])
    merged = finalize(classes, dump["build"], h_types, h_fields)
    delta = {h: k for h, k in merged.items() if live_out.get(h) != k}
    external = {h: name for h, name in external_type_names(merged, h_types).items()
                if h not in live_external}
    return {
        "channel": PREVIEW_CHANNEL,
        "patch": dump["patch"],
        "build": dump["build"],
        "base": base["build"],
        "externalTypeNames": external,
        "classes": delta,
    }


def preview_counts(preview, live_out):
    """Returns the (added, removed, changed) class counts of `preview`. A class
    is added if the latest live build does not have it. A class is removed if
    the latest live build has it and the PBE build does not."""
    added = removed = changed = 0
    for khash, klass in preview["classes"].items():
        live = live_out.get(khash)
        if live is None or not is_open(live):
            added += 1
        elif not is_open(klass):
            removed += 1
        else:
            changed += 1
    return added, removed, changed


def compact(obj):
    return json.dumps(obj, separators=(",", ":"))


def read_hash_source(hashes_dir):
    """Which name snapshot the names below were resolved against, from
    hashes/provenance.json. Absent on pre-refresh checkouts."""
    path = os.path.join(hashes_dir, "provenance.json")
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8") as f:
        prov = json.load(f)
    out = {"fetchedAt": prov.get("fetchedAt")}
    for table, info in prov.get("tables", {}).items():
        out[table] = {"url": info.get("url"), "entries": info.get("entries")}
    return out


def write_external(f, external):
    f.write('"externalTypeNames": {\n')
    ext_keys = list(external)
    for i, h in enumerate(ext_keys):
        comma = "," if i < len(ext_keys) - 1 else ""
        f.write(f"{json.dumps(h)}: {json.dumps(external[h])}{comma}\n")
    f.write("},\n")


def write_classes(f, classes_out):
    f.write('"classes": {\n')
    class_keys = list(classes_out)
    for ci, khash in enumerate(class_keys):
        klass = classes_out[khash]
        head = f"{json.dumps(khash)}: {{"
        if "name" in klass:
            head += f'"name": {json.dumps(klass["name"])}, '
        head += f'"revisions": {compact(klass["revisions"])}, "properties": {{'
        f.write(head + "\n")
        prop_keys = list(klass["properties"])
        for pi, fhash in enumerate(prop_keys):
            comma = "," if pi < len(prop_keys) - 1 else ""
            f.write(f" {json.dumps(fhash)}: {compact(klass['properties'][fhash])}{comma}\n")
        comma = "," if ci < len(class_keys) - 1 else ""
        f.write("}}" + comma + "\n")
    f.write("}\n")


def write_db_json(path, versions, latest_build, classes_out, external, hash_source=None):
    """Hand-rolled layout: one line per property, one line per version entry.
    A schema change in one property diffs as a single-line change."""
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("{\n")
        f.write(f'"formatVersion": {FORMAT_VERSION},\n')
        if hash_source:
            f.write(f'"hashSource": {compact(hash_source)},\n')
        f.write(f'"latest": {latest_build},\n')
        f.write('"versions": [\n')
        for i, v in enumerate(versions):
            comma = "," if i < len(versions) - 1 else ""
            f.write(compact({"patch": v["patch"], "build": v["build"]}) + comma + "\n")
        f.write("],\n")
        write_external(f, external)
        write_classes(f, classes_out)
        f.write("}\n")


def write_preview_json(path, preview):
    """Writes the PBE overlay file in the layout of `write_db_json`.

    If `preview` is None, the file states that no preview exists: "patch",
    "build" and "base" are null and "classes" is empty. The file is written in
    both cases, so a stale overlay never stays on disk."""
    if preview is None:
        preview = {"channel": PREVIEW_CHANNEL, "patch": None, "build": None, "base": None,
                   "externalTypeNames": {}, "classes": {}}
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("{\n")
        f.write(f'"formatVersion": {FORMAT_VERSION},\n')
        for key in ("channel", "patch", "build", "base"):
            f.write(f'"{key}": {json.dumps(preview[key])},\n')
        write_external(f, preview["externalTypeNames"])
        write_classes(f, preview["classes"])
        f.write("}\n")


def write_snapshot_py(py_path, latest_dump_path, h_types, h_fields):
    """database.py as a view of the latest build only, in the existing format."""
    db = {}
    db_import.import_database(db, read_meta(latest_dump_path))
    # write_databse resolves names through module-level hash maps
    db_import.h_types = h_types
    db_import.h_fields = h_fields
    db_import.write_databse(py_path, db)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--dumps", default="dumps", help="directory of per-build dumps")
    parser.add_argument("--hashes", default="hashes", help="directory of hash name lists")
    parser.add_argument("--out", default="db/meta.db.json", help="versioned database output")
    parser.add_argument("--py", default="db/database.py",
                        help="latest-build snapshot output (existing .py format)")
    parser.add_argument("--skip-py", action="store_true",
                        help="do not regenerate the database.py snapshot")
    parser.add_argument("--preview-dumps", default=None,
                        help="directory of the PBE dump (default: <dumps>/pbe)")
    parser.add_argument("--preview-out", default=None,
                        help=f"PBE overlay output (default: {PREVIEW_FILE} next to --out)")
    parser.add_argument("--no-preview", action="store_true",
                        help="do not read the PBE dump and do not write the PBE overlay")
    args = parser.parse_args()

    dumps = discover_dumps(args.dumps)
    if not dumps:
        print(f"[error] no dumps found in {args.dumps}", file=sys.stderr)
        return 1
    latest = dumps[-1]
    print(f"[..] folding {len(dumps)} dumps "
          f"({dumps[0]['patch']}.{dumps[0]['build']} -> {latest['patch']}.{latest['build']})")

    h_types = read_resolved_hashes(args.hashes, "bintypes")
    h_fields = read_resolved_hashes(args.hashes, "binfields")

    classes = build_history(dumps)
    classes_out = finalize(classes, latest["build"], h_types, h_fields)
    external = external_type_names(classes_out, h_types)

    # The PBE fold modifies `classes`, so it runs after the live finalize.
    preview = None
    if not args.no_preview:
        preview_dir = args.preview_dumps or os.path.join(args.dumps, PREVIEW_CHANNEL)
        preview_dump = discover_preview(preview_dir, dumps)
        if preview_dump:
            preview = build_preview(classes, classes_out, external, latest, preview_dump,
                                    h_types, h_fields)

    versions = [{"patch": d["patch"], "build": d["build"]} for d in dumps]
    hash_source = read_hash_source(args.hashes)
    write_db_json(args.out, versions, latest["build"], classes_out, external, hash_source)

    # Sanity check: the hand-rolled writer must produce valid JSON.
    with open(args.out, encoding="utf-8") as f:
        json.load(f)

    preview_out = args.preview_out or os.path.join(os.path.dirname(args.out), PREVIEW_FILE)
    if not args.no_preview:
        write_preview_json(preview_out, preview)
        with open(preview_out, encoding="utf-8") as f:
            json.load(f)

    total_props = sum(len(k["properties"]) for k in classes_out.values())
    removed_classes = sum(1 for k in classes_out.values() if "to" in k["revisions"][-1])
    removed_props = sum(1 for k in classes_out.values()
                        for p in k["properties"].values() if "to" in p["revisions"][-1])
    multi_rev = sum(1 for k in classes_out.values()
                    for p in k["properties"].values() if len(p["revisions"]) > 1)
    print(f"[ok] {args.out}: {len(classes_out)} classes ({removed_classes} removed), "
          f"{total_props} properties ({removed_props} removed, {multi_rev} with >1 revision)")
    if preview:
        added, removed, changed = preview_counts(preview, classes_out)
        print(f"[ok] {preview_out}: {preview['channel']} {preview['patch']}.{preview['build']} over "
              f"{preview['base']}: {len(preview['classes'])} classes "
              f"({added} added, {removed} removed, {changed} changed)")
    elif not args.no_preview:
        print(f"[ok] {preview_out}: no preview")

    if not args.skip_py:
        write_snapshot_py(args.py, latest["path"], h_types, h_fields)
        print(f"[ok] {args.py}: snapshot of {latest['patch']}.{latest['build']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

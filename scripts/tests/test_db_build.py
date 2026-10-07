#!/usr/bin/env python3
"""Tests for the preview overlay of scripts/db_build.py.

Usage:
    python3 -m unittest discover -s scripts/tests
"""

import contextlib
import io
import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import db_build

KEPT = "0x1"
CHANGED = "0x2"
REMOVED = "0x3"
ADDED = "0x4"
FIELD_A = "0xa"
FIELD_B = "0xb"

H_TYPES = {KEPT: "Kept", ADDED: "Added"}
H_FIELDS = {FIELD_A: "FieldA"}


def prop(value_type):
    return {"other_class": None, "value_type": value_type, "container": None, "map": None}


def klass(properties=None, base=None):
    return {
        "base": base,
        "secondary_bases": {},
        "is": {"interface": False, "value": False},
        "properties": {name: prop(t) for name, t in (properties or {}).items()},
        "defaults": None,
    }


LIVE_CLASSES = {
    KEPT: klass({FIELD_A: "U32"}),
    CHANGED: klass({FIELD_A: "U32", FIELD_B: "String"}),
    REMOVED: klass({FIELD_A: "Bool"}),
}

# FIELD_A of CHANGED changes type, FIELD_B of CHANGED is removed.
PBE_CLASSES = {
    KEPT: klass({FIELD_A: "U32"}),
    CHANGED: klass({FIELD_A: "F32"}),
    ADDED: klass({FIELD_A: "U8"}, base=KEPT),
}


def write_dump(directory, version, classes):
    os.makedirs(directory, exist_ok=True)
    path = os.path.join(directory, f"{version}.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump({"version": version, "classes": classes}, f)
    return path


class Corpus:
    """Two live dumps in `dumps` and a PBE directory `pbe`."""

    def __init__(self, root):
        self.dumps_dir = os.path.join(root, "dumps")
        self.pbe_dir = os.path.join(self.dumps_dir, "pbe")
        write_dump(self.dumps_dir, "16.18.100", LIVE_CLASSES)
        write_dump(self.dumps_dir, "16.19.200", LIVE_CLASSES)

    def add_pbe(self, version, classes=None):
        return write_dump(self.pbe_dir, version, PBE_CLASSES if classes is None else classes)

    def build(self):
        """Returns (live classes, live external names, preview or None)."""
        dumps = db_build.discover_dumps(self.dumps_dir)
        latest = dumps[-1]
        classes = db_build.build_history(dumps)
        live_out = db_build.finalize(classes, latest["build"], H_TYPES, H_FIELDS)
        external = db_build.external_type_names(live_out, H_TYPES)
        with contextlib.redirect_stdout(io.StringIO()):
            dump = db_build.discover_preview(self.pbe_dir, dumps)
        preview = None
        if dump:
            preview = db_build.build_preview(classes, live_out, external, latest, dump,
                                             H_TYPES, H_FIELDS)
        return live_out, external, preview


class PreviewTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.corpus = Corpus(self.tmp.name)

    def test_discover_dumps_skips_pbe_directory(self):
        self.corpus.add_pbe("16.21.300")
        builds = [d["build"] for d in db_build.discover_dumps(self.corpus.dumps_dir)]
        self.assertEqual(builds, [100, 200])

    def test_discover_preview_returns_none_if_directory_is_missing(self):
        _, _, preview = self.corpus.build()
        self.assertIsNone(preview)

    def test_discover_preview_returns_none_if_patch_equals_live_patch(self):
        self.corpus.add_pbe("16.19.150")
        _, _, preview = self.corpus.build()
        self.assertIsNone(preview)

    def test_discover_preview_returns_dump_if_build_is_below_live_build(self):
        # Ordering is by channel: a live hotfix build can be greater than the
        # newest PBE build.
        self.corpus.add_pbe("16.20.150")
        _, _, preview = self.corpus.build()
        self.assertEqual((preview["patch"], preview["build"], preview["base"]),
                         ("16.20", 150, 200))

    def test_discover_preview_returns_greatest_dump_if_directory_holds_several(self):
        self.corpus.add_pbe("16.20.900")
        self.corpus.add_pbe("16.21.300")
        self.corpus.add_pbe("16.21.250")
        _, _, preview = self.corpus.build()
        self.assertEqual((preview["patch"], preview["build"]), ("16.21", 300))

    def test_build_preview_keeps_only_classes_that_differ(self):
        self.corpus.add_pbe("16.21.300")
        _, _, preview = self.corpus.build()
        self.assertEqual(list(preview["classes"]), [CHANGED, REMOVED, ADDED])
        self.assertEqual(preview["channel"], "pbe")
        self.assertEqual(preview["base"], 200)

    def test_build_preview_closes_removed_class_at_base(self):
        self.corpus.add_pbe("16.21.300")
        _, _, preview = self.corpus.build()
        revisions = preview["classes"][REMOVED]["revisions"]
        self.assertEqual((revisions[-1]["from"], revisions[-1]["to"]), (100, 200))

    def test_build_preview_closes_removed_property_at_base(self):
        self.corpus.add_pbe("16.21.300")
        _, _, preview = self.corpus.build()
        properties = preview["classes"][CHANGED]["properties"]
        self.assertEqual(properties[FIELD_B]["revisions"][-1]["to"], 200)
        self.assertEqual(
            [(r["from"], r.get("to"), r["type"][0]) for r in properties[FIELD_A]["revisions"]],
            [(100, 200, "U32"), (300, None, "F32")])

    def test_build_preview_opens_added_class_at_pbe_build(self):
        self.corpus.add_pbe("16.21.300")
        _, _, preview = self.corpus.build()
        added = preview["classes"][ADDED]
        self.assertEqual(added["name"], "Added")
        self.assertEqual(added["revisions"],
                         [{"from": 300, "bases": [KEPT], "interface": False, "value": False}])

    def test_build_preview_returns_no_classes_if_pbe_equals_live(self):
        self.corpus.add_pbe("16.21.300", LIVE_CLASSES)
        _, _, preview = self.corpus.build()
        self.assertEqual(preview["classes"], {})

    def test_build_preview_does_not_modify_live_output(self):
        before, _, _ = Corpus(tempfile.mkdtemp(dir=self.tmp.name)).build()
        self.corpus.add_pbe("16.21.300")
        live_out, _, _ = self.corpus.build()
        self.assertEqual(live_out, before)

    def test_preview_counts_returns_added_removed_changed(self):
        self.corpus.add_pbe("16.21.300")
        live_out, _, preview = self.corpus.build()
        self.assertEqual(db_build.preview_counts(preview, live_out), (1, 1, 1))

    def test_merged_preview_equals_fold_with_pbe_as_last_build(self):
        pbe_path = self.corpus.add_pbe("16.21.300")
        live_out, external, preview = self.corpus.build()

        dumps = db_build.discover_dumps(self.corpus.dumps_dir)
        pbe = {"patch": "16.21", "build": 300, "path": pbe_path, "_mm": (16, 21)}
        classes = db_build.build_history(dumps + [pbe])
        full = db_build.finalize(classes, 300, H_TYPES, H_FIELDS)

        self.assertEqual({**live_out, **preview["classes"]}, full)
        self.assertEqual({**external, **preview["externalTypeNames"]},
                         db_build.external_type_names(full, H_TYPES))


class MainTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.corpus = Corpus(self.tmp.name)
        self.hashes_dir = os.path.join(self.tmp.name, "hashes")
        os.makedirs(self.hashes_dir)
        for table, names in (("bintypes", H_TYPES), ("binfields", H_FIELDS)):
            path = os.path.join(self.hashes_dir, f"hashes.{table}.txt")
            with open(path, "w", encoding="utf-8") as f:
                for h, name in names.items():
                    f.write(f"{int(h, 16):08x} {name}\n")
        self.out_dir = os.path.join(self.tmp.name, "db")
        self.out = os.path.join(self.out_dir, "meta.db.json")
        self.overlay = os.path.join(self.out_dir, "meta.pbe.json")

    def run_main(self, *extra):
        argv = ["db_build.py", "--dumps", self.corpus.dumps_dir, "--hashes", self.hashes_dir,
                "--out", self.out, "--skip-py", *extra]
        old_argv = sys.argv
        sys.argv = argv
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(db_build.main(), 0)
        finally:
            sys.argv = old_argv

    def read(self, path):
        with open(path, "rb") as f:
            return f.read()

    def read_overlay(self, path=None):
        return json.loads(self.read(path or self.overlay))

    def test_main_writes_empty_overlay_if_no_pbe_dump_exists(self):
        self.run_main()
        self.assertEqual(self.read_overlay(), {
            "formatVersion": 1, "channel": "pbe", "patch": None, "build": None, "base": None,
            "externalTypeNames": {}, "classes": {}})

    def test_main_writes_overlay_without_changing_meta_db(self):
        self.run_main()
        live = self.read(self.out)
        self.corpus.add_pbe("16.21.300")
        self.run_main()
        self.assertEqual(self.read(self.out), live)
        self.assertNotIn("preview", json.loads(live))
        overlay = self.read_overlay()
        self.assertEqual(list(overlay), ["formatVersion", "channel", "patch", "build", "base",
                                         "externalTypeNames", "classes"])
        self.assertEqual((overlay["patch"], overlay["build"], overlay["base"]),
                         ("16.21", 300, 200))
        self.assertEqual(list(overlay["classes"]), [CHANGED, REMOVED, ADDED])

    def test_main_writes_empty_overlay_if_pbe_dump_is_removed(self):
        path = self.corpus.add_pbe("16.21.300")
        self.run_main()
        self.assertEqual(self.read_overlay()["build"], 300)
        os.remove(path)
        self.run_main()
        self.assertIsNone(self.read_overlay()["build"])
        self.assertEqual(self.read_overlay()["classes"], {})

    def test_main_does_not_write_overlay_if_no_preview_flag_is_set(self):
        self.corpus.add_pbe("16.21.300")
        self.run_main("--no-preview")
        self.assertFalse(os.path.exists(self.overlay))

    def test_main_reads_preview_dumps_directory_if_given(self):
        other = os.path.join(self.tmp.name, "other")
        write_dump(other, "16.22.400", PBE_CLASSES)
        self.run_main("--preview-dumps", other)
        self.assertEqual(self.read_overlay()["build"], 400)

    def test_main_writes_overlay_to_preview_out_if_given(self):
        self.corpus.add_pbe("16.21.300")
        path = os.path.join(self.tmp.name, "elsewhere.json")
        self.run_main("--preview-out", path)
        self.assertEqual(self.read_overlay(path)["build"], 300)
        self.assertFalse(os.path.exists(self.overlay))


if __name__ == "__main__":
    unittest.main()

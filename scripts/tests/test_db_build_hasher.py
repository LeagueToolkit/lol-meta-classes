#!/usr/bin/env python3
"""Tests for the hasher of `Hash` properties in scripts/db_build.py.

Usage:
    python3 -m unittest discover -s scripts/tests
"""

import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import db_build

CLASS = "0x1"
FIELD = "0xa"

FNV = {"storage_width": 4, "hash_function": {"algorithm": "Fnv1a32", "lowercased": True}}
XXH3_LOW32 = {"storage_width": 4, "hash_function": {"algorithm": "Xxh3", "lowercased": True}}
XXH3_WIDE = {"storage_width": 8, "hash_function": {"algorithm": "Xxh3", "lowercased": True}}


def dump(value_type="Hash", hasher=None, hashers=None, default="0x0"):
    """Returns a dump with one class that has one property. `hashers` is
    omitted if None, like a dump before format version 3."""
    prop = {"other_class": None, "value_type": value_type, "container": None, "map": None}
    if hashers is not None:
        prop["hasher"] = hasher
    out = {"classes": {CLASS: {
        "base": None,
        "secondary_bases": {},
        "is": {"interface": False, "value": False},
        "properties": {FIELD: prop},
        "defaults": {FIELD: default},
    }}}
    if hashers is not None:
        out["hashers"] = hashers
    return out


class HasherTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def revisions(self, *dumps):
        """Folds `dumps` as builds 100, 200, ... and returns the revisions of
        the property."""
        for i, content in enumerate(dumps, start=1):
            path = os.path.join(self.tmp.name, f"16.{i}.{i * 100}.json")
            with open(path, "w", encoding="utf-8") as f:
                json.dump(content, f)
        found = db_build.discover_dumps(self.tmp.name)
        classes = db_build.build_history(found)
        out = db_build.finalize(classes, found[-1]["build"], {}, {})
        return out[CLASS]["properties"][FIELD]["revisions"]

    def test_field_hasher_returns_none_if_property_has_no_hasher_key(self):
        field = dump()["classes"][CLASS]["properties"][FIELD]
        self.assertIsNone(db_build.field_hasher(field, {}))

    def test_field_hasher_returns_none_if_hasher_is_null(self):
        field = dump(value_type="U32", hasher=None, hashers={})["classes"][CLASS]["properties"][FIELD]
        self.assertIsNone(db_build.field_hasher(field, {}))

    def test_field_hasher_returns_none_if_hasher_is_default(self):
        field = dump(hasher="0x10", hashers={"0x10": FNV})["classes"][CLASS]["properties"][FIELD]
        self.assertIsNone(db_build.field_hasher(field, {"0x10": FNV}))

    def test_field_hasher_returns_width_algorithm_lowercased(self):
        field = dump(hasher="0x20", hashers={"0x20": XXH3_WIDE})["classes"][CLASS]["properties"][FIELD]
        self.assertEqual(db_build.field_hasher(field, {"0x20": XXH3_WIDE}), (8, "Xxh3", True))

    def test_build_history_omits_hasher_if_hasher_is_default(self):
        revisions = self.revisions(dump(hasher="0x10", hashers={"0x10": FNV}))
        self.assertEqual(revisions,
                         [{"from": 100, "type": ["Hash", "0x0", "0x0", "0x0"], "default": "0x0"}])

    def test_build_history_writes_hasher_between_type_and_default(self):
        revisions = self.revisions(dump(hasher="0x20", hashers={"0x20": XXH3_WIDE}))
        self.assertEqual(list(revisions[0]), ["from", "type", "hasher", "default"])
        self.assertEqual(revisions[0]["hasher"],
                         {"width": 8, "algorithm": "Xxh3", "lowercased": True})

    def test_build_history_keeps_revision_if_dump_gains_default_hasher(self):
        # The dump before format version 3 records no hasher. The next dump
        # records the default hasher. The definition did not change.
        revisions = self.revisions(dump(), dump(hasher="0x10", hashers={"0x10": FNV}))
        self.assertEqual(len(revisions), 1)

    def test_build_history_keeps_revision_if_hasher_vtable_changes(self):
        revisions = self.revisions(dump(hasher="0x10", hashers={"0x10": XXH3_WIDE}),
                                   dump(hasher="0x99", hashers={"0x99": XXH3_WIDE}))
        self.assertEqual(len(revisions), 1)

    def test_build_history_starts_revision_if_hasher_changes(self):
        revisions = self.revisions(dump(hasher="0x10", hashers={"0x10": FNV}),
                                   dump(hasher="0x20", hashers={"0x20": XXH3_LOW32}),
                                   dump(hasher="0x30", hashers={"0x30": XXH3_WIDE}),
                                   dump(hasher="0x10", hashers={"0x10": FNV}))
        self.assertEqual(
            [(r["from"], r.get("to"), r.get("hasher", {}).get("width"),
              r.get("hasher", {}).get("algorithm")) for r in revisions],
            [(100, 100, None, None), (200, 200, 4, "Xxh3"), (300, 300, 8, "Xxh3"),
             (400, None, None, None)])

    def test_build_history_starts_revision_if_string_becomes_wide_hash(self):
        revisions = self.revisions(dump(value_type="String", default=""),
                                   dump(hasher="0x20", hashers={"0x20": XXH3_WIDE}))
        self.assertEqual(revisions, [
            {"from": 100, "to": 100, "type": ["String", "0x0", "0x0", "0x0"], "default": ""},
            {"from": 200, "type": ["Hash", "0x0", "0x0", "0x0"],
             "hasher": {"width": 8, "algorithm": "Xxh3", "lowercased": True}, "default": "0x0"},
        ])


if __name__ == "__main__":
    unittest.main()

#!/bin/env python
"""Shader reflection attestation: compiled DXBC -> RDEF names -> unresolved hashes.

`docs/plans/shader-reflection.md` is the plan; `docs/shader-reflection.md` is
what it measured. The guesser proposes a name and the hash fails to contradict
it. This runs the other way, like `exe-strings`: the probe set is nothing but
identifiers the game shipped, so a hit is a name Riot wrote against a hash the
game registered.

Compiled shaders keep their reflection metadata. Every DXBC container carries an
`RDEF` chunk naming its constant buffers, every member in declaration order with
type and offset, its resource bindings, and - only in the shader-model-5
extended type descriptor - the declared name of each struct type. That last one
is where `LightRegionRenderData` came from, so the parser handles it or the
single highest-value field is silently absent.

**Every permutation, unioned.** Windows ships no HLSL source, so a permutation
can only be selected, never constructed, and selection is the wrong operation
anyway: `#ifdef` blocks add whole constant buffers, so feature A's block may
never co-occur with feature B's and a "fattest" build is a local maximum rather
than a union. We walk the container regardless, so parsing all of them is free
and strictly dominates any single one. Deduplicating on the RDEF bytes before
parsing collapses ~77k containers onto ~45k distinct reflection blocks.

## The three probe tiers

Each is derived from the one before it and reported separately, because they are
not equally strong:

- **literal** - the identifier exactly as reflected. The bin hash is FNV-1a over
  the *lowercased* name, so a shipped `modelHeight` and a written `ModelHeight`
  are one hash and the recasing is the house rule's, not a new claim. This is
  the `exe-strings` standard: the shipped string is what hashes.
- **undecorated** - the suffix the effect builder appends to a resource binding
  removed (`BAKED_DIFFUSE_TEXTURE__TX` -> `BAKED_DIFFUSE_TEXTURE`). Still an
  engine identifier, just without engine punctuation.
- **deunderscored** - `SUN_PENUMBRA_SATURATION` -> `SunPenumbraSaturation`.
  **This tier is a reconstruction, not the shipped string.** An underscore is
  hashed like any other byte, so the shipped identifier does not hash to the
  target; what hashes is the word sequence with the separators removed. The
  shader attests the concept and the words in order, and the hash confirms the
  join, but this is not the standard the literal tier meets. Land it in its own
  batch and say so.

## Reading a hit

A hash match is where the work starts, not where it ends. The corpus is ~3k
identifiers of ordinary graphics vocabulary, so a single common word landing on
a semantically plausible field is exactly the coincidence to expect: `Bounce`
occurs in one UI RGB-shift shader and lands on a physics collision modifier, and
that is vocabulary, not evidence. `--context` prints the constant buffer and the
shader paths each hit came from, which is what separates the two - a name is
attested when the block it lives in corresponds to the class that owns the
field.

Output goes to `hash-guesser-out/`, gitignored, because a candidate must never
be mistaken for a crack. Needs the `zstandard` package and a League install.

Usage:
    python3 scripts/shader_reflect.py
    python3 scripts/shader_reflect.py --context          # per-hit provenance
    python3 scripts/shader_reflect.py --names ReflectionSkyTint --context
"""

import argparse
import collections
import glob
import gzip
import hashlib
import json
import os
import pathlib
import re
import struct
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import names as names_mod  # noqa: E402
from hashtool import fnv1a_32, load_tables  # noqa: E402
from guesser_families import (  # noqa: E402
    load_db, field_name_map, unresolved_sets)

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Windows retail is dx11 only; the .glsl/.metal/.dx9 variants in the `.game`
# path table ship on other platforms. Checked across all 455 wads of a 16.15
# install: compiled-shader paths occur in these two and nowhere else.
WADS = ("ShaderCache.dx11.wad.client", "Bootstrap.windows.wad.client")

DXBC = b"DXBC"
ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"

# The effect builder appends these to a resource binding's name.
DECOR = ("__TX", "__SMP", "_SharedTexture", "_SharedSampler", "_SharedDataBuffer")

KINDS = ("cbuffer", "struct", "member", "binding")


# --------------------------------------------------------------------------
# WAD v3.x - enough to enumerate and pull chunks
# --------------------------------------------------------------------------

class Wad:
    def __init__(self, path):
        import zstandard
        self._zstd = zstandard
        self.f = open(path, "rb")
        magic, major, minor = struct.unpack("<2sBB", self.f.read(4))
        if magic != b"RW" or major != 3:
            raise ValueError(f"unsupported wad {magic!r} v{major}.{minor}")
        self.f.read(256)                                    # PKCS#1 signature
        self.f.read(8)                                      # data checksum
        count, = struct.unpack("<I", self.f.read(4))
        self.chunks = []
        for _ in range(count):
            buf = self.f.read(32)
            if minor >= 4:
                (ph, off, csize, usize, tf, hi, lo, mi, _ck) = \
                    struct.unpack("<QIIIBBBBQ", buf)
            else:
                (ph, off, csize, usize, tf, _d, _sf, _ck) = \
                    struct.unpack("<QIiiBBHQ", buf)
            self.chunks.append((ph, off, csize, usize, tf & 0xF))

    def data(self, chunk):
        _ph, off, csize, usize, comp = chunk
        self.f.seek(off)
        raw = self.f.read(csize)
        if comp == 0:
            return raw
        if comp == 1:
            return gzip.decompress(raw)[:usize]
        if comp == 3:
            return self._zstd.ZstdDecompressor().decompress(
                raw, max_output_size=usize)
        if comp == 4:
            # A raw head, then one or more zstd frames.
            i = raw.find(ZSTD_MAGIC)
            if i < 0:
                raise ValueError("zstd-multi with no frame magic")
            rdr = self._zstd.ZstdDecompressor().stream_reader(
                raw[i:], read_across_frames=True)
            return raw[:i] + rdr.read(usize - i)
        raise ValueError(f"unsupported wad compression {comp}")


# --------------------------------------------------------------------------
# DXBC container + RDEF reflection
# --------------------------------------------------------------------------

def containers(blob):
    """A shader-cache chunk bundles every permutation of one shader stage as a
    record sequence: `u32 size` (the container's own totalSize plus one), the
    DXBC container, then a trailer byte. Falls back to a scan for the wads that
    mix shaders in with everything else."""
    if len(blob) > 8 and blob[4:8] == DXBC:
        pos, n = 0, len(blob)
        while pos + 4 <= n:
            size, = struct.unpack_from("<I", blob, pos)
            pos += 4
            if size < 32 or pos + size > n or blob[pos:pos + 4] != DXBC:
                return
            total, = struct.unpack_from("<I", blob, pos + 24)
            if total > size:
                return
            yield blob[pos:pos + total]
            pos += size
        return
    pos = 0
    while True:
        pos = blob.find(DXBC, pos)
        if pos < 0:
            return
        if pos + 32 <= len(blob):
            total, = struct.unpack_from("<I", blob, pos + 24)
            if 32 < total <= len(blob) - pos:
                yield blob[pos:pos + total]
                pos += total
                continue
        pos += 4


def dxbc_chunks(cont):
    count, = struct.unpack_from("<I", cont, 28)
    out = {}
    for o in struct.unpack_from(f"<{count}I", cont, 32):
        if o + 8 > len(cont):
            continue
        size, = struct.unpack_from("<I", cont, o + 4)
        out[cont[o:o + 4]] = cont[o + 8:o + 8 + size]
    return out


def _cstr(buf, off):
    if off <= 0 or off >= len(buf):
        return None
    end = buf.find(b"\0", off)
    if end < 0:
        return None
    try:
        return buf[off:end].decode("ascii")
    except UnicodeDecodeError:
        return None


def parse_rdef(rd):
    """RDEF payload -> {cbuffers, bindings}. Offsets are relative to its start."""
    cb_count, cb_off, rb_count, rb_off = struct.unpack_from("<IIII", rd, 0)
    major = rd[17]
    sm5 = major >= 5
    seen = {}

    def parse_type(off, depth=0):
        """Type descriptor -> name, class, members.

        Shader model 5 appends five u32 to the 16-byte descriptor, the last of
        which points at the type's own name. A struct's declared name survives
        nowhere else, so an SM4-shaped parser loses it silently."""
        if off in seen:
            return seen[off]
        if off <= 0 or off + 16 > len(rd):
            return None
        var_class, var_type, rows, cols, elements, members = \
            struct.unpack_from("<6H", rd, off)
        member_off, = struct.unpack_from("<I", rd, off + 12)
        name = None
        if sm5 and off + 36 <= len(rd):
            name_off, = struct.unpack_from("<I", rd, off + 32)
            name = _cstr(rd, name_off)
        e = {"name": name, "class": var_class, "type": var_type, "rows": rows,
             "cols": cols, "elements": elements, "members": []}
        seen[off] = e
        if members and member_off and depth < 8:
            for i in range(members):
                mo = member_off + i * 12
                if mo + 12 > len(rd):
                    break
                mn, mt, moff = struct.unpack_from("<III", rd, mo)
                e["members"].append({"name": _cstr(rd, mn), "offset": moff,
                                     "type": parse_type(mt, depth + 1)})
        return e

    stride = 40 if sm5 else 24
    cbuffers = []
    for i in range(cb_count):
        o = cb_off + i * 24
        if o + 24 > len(rd):
            break
        name_off, var_count, var_off, size, _f, _t = \
            struct.unpack_from("<6I", rd, o)
        cb = {"name": _cstr(rd, name_off), "size": size, "vars": []}
        for j in range(var_count):
            vo = var_off + j * stride
            if vo + 24 > len(rd):
                break
            vn, start, vsize, _vf, toff, _dv = struct.unpack_from("<6I", rd, vo)
            cb["vars"].append({"name": _cstr(rd, vn), "offset": start,
                               "size": vsize, "type": parse_type(toff)})
        cbuffers.append(cb)

    bindings = []
    for i in range(rb_count):
        o = rb_off + i * 32
        if o + 32 > len(rd):
            break
        nm, rtype, _ret, _dim, _ns, point, _cnt, _fl = \
            struct.unpack_from("<8I", rd, o)
        bindings.append({"name": _cstr(rd, nm), "type": rtype, "slot": point})

    return {"cbuffers": cbuffers, "bindings": bindings}


# --------------------------------------------------------------------------
# Harvest
# --------------------------------------------------------------------------

def harvest(final_dir, stats):
    """Every shader wad -> name -> {kinds, cbuffers, shader path hashes}."""
    idx = collections.defaultdict(
        lambda: {"kind": collections.Counter(),
                 "cb": collections.Counter(), "sh": set()})
    seen_rdef = set()

    def note(name, kind, cb, ph):
        if not name:
            return
        e = idx[name]
        e["kind"][kind] += 1
        if cb:
            e["cb"][cb] += 1
        e["sh"].add(ph)

    def walk(t, cb, ph, depth=0):
        if not t or depth > 6:
            return
        if t.get("name"):
            note(t["name"], "struct", cb, ph)
        for m in t["members"]:
            note(m.get("name"), "member", cb, ph)
            walk(m.get("type"), cb, ph, depth + 1)

    for wname in WADS:
        path = os.path.join(final_dir, wname)
        if not os.path.exists(path):
            print(f"[warn] missing {path}", file=sys.stderr)
            continue
        w = Wad(path)
        stats[f"chunks.{wname}"] = len(w.chunks)
        for c in w.chunks:
            try:
                blob = w.data(c)
            except Exception:
                stats["chunk_error"] += 1
                continue
            ph = f"{c[0]:016x}"
            for cont in containers(blob):
                stats["dxbc"] += 1
                try:
                    ch = dxbc_chunks(cont)
                except Exception:
                    stats["container_error"] += 1
                    continue
                rd = ch.get(b"RDEF")
                if rd is None:
                    stats["no_rdef"] += 1
                    continue
                key = hashlib.sha1(rd).digest()
                if key in seen_rdef:
                    stats["rdef_duplicate"] += 1
                    continue
                seen_rdef.add(key)
                try:
                    r = parse_rdef(rd)
                except Exception:
                    stats["rdef_error"] += 1
                    continue
                stats["rdef_unique"] += 1
                for cb in r["cbuffers"]:
                    note(cb["name"], "cbuffer", None, ph)
                    for v in cb["vars"]:
                        note(v["name"], "member", cb["name"], ph)
                        walk(v.get("type"), cb["name"], ph)
                for b in r["bindings"]:
                    note(b["name"], "binding", None, ph)
        print(f"[wad] {wname}: {len(w.chunks)} chunks", file=sys.stderr)
    return idx


# --------------------------------------------------------------------------
# Probe tiers
# --------------------------------------------------------------------------

def undecorate(name):
    for s in DECOR:
        if name.endswith(s) and len(name) > len(s):
            return name[:-len(s)]
    return None


def deunderscore(name):
    """`SUN_PENUMBRA_SATURATION` -> `SunPenumbraSaturation`. A reconstruction:
    the separator is hashed, so the shipped identifier is not what matches."""
    parts = [p for p in name.split("_") if p]
    if len(parts) < 2:
        return None
    return "".join(p[0].upper() + (p[1:].lower() if p.isupper() else p[1:])
                   for p in parts)


def build_tiers(idx):
    """Ordered tiers of `probe -> source identifier`, each disjoint from those
    before it, so a probe is priced once and at its strongest derivation."""
    tiers = collections.OrderedDict(
        (t, {}) for t in ("literal", "undecorated", "deunderscored"))
    for n in idx:
        tiers["literal"].setdefault(n, n)
    taken = {k.lower() for k in tiers["literal"]}
    for n in sorted(idx):
        u = undecorate(n)
        if u and u.lower() not in taken:
            tiers["undecorated"].setdefault(u, n)
    taken |= {k.lower() for k in tiers["undecorated"]}
    for n in sorted(idx):
        for form in (n, undecorate(n)):
            d = deunderscore(form) if form else None
            if d and d.lower() not in taken:
                tiers["deunderscored"].setdefault(d, n)
    return tiers


# --------------------------------------------------------------------------

def load_known(hashes_dir):
    merged = {}
    for t in ("bintypes", "binfields"):
        ovr, m = load_tables(hashes_dir, t)
        for src in (ovr, m):
            for h, n in src.items():
                merged.setdefault(int(h, 16), n)
    return merged


def resolve_paths(wanted, game_glob):
    out = {}
    files = sorted(glob.glob(game_glob))
    if not files:
        return out
    for f in files:
        with open(f, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                sp = line.find(" ")
                if sp > 0 and line[:sp] in wanted:
                    out[line[:sp]] = line[sp + 1:].strip()
    return out


def main():
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("--game", metavar="DIR",
                   default="C:/Riot Games/League of Legends/Game/DATA/FINAL",
                   help="the install's DATA/FINAL (default: %(default)s)")
    p.add_argument("--db", default="db/meta.db.json", metavar="FILE")
    p.add_argument("--hashes", default="hashes", metavar="DIR")
    p.add_argument("--paths", metavar="GLOB",
                   default="D:/lol/Data/hashes/lol/hashes.game.txt*",
                   help="CDragon's `.game` table, for --context shader paths")
    p.add_argument("--names", nargs="*", metavar="NAME",
                   help="report only these reflected identifiers, hit or not")
    p.add_argument("--context", action="store_true",
                   help="per-hit constant buffer and shader paths - the thing "
                        "that separates attestation from shared vocabulary")
    p.add_argument("-o", "--out", default="hash-guesser-out/shader.reflect.tsv",
                   metavar="FILE", help="the probe set (default: %(default)s)")
    args = p.parse_args()

    stats = collections.Counter()
    idx = harvest(args.game, stats)
    for k in sorted(stats):
        print(f"       {k} = {stats[k]}", file=sys.stderr)
    print(f"[in] {len(idx)} distinct reflected identifier(s)", file=sys.stderr)
    kinds = collections.Counter(max(e["kind"], key=e["kind"].get)
                                for e in idx.values())
    print("     " + "  ".join(f"{k}={v}" for k, v in kinds.most_common()),
          file=sys.stderr)

    _db, classes, nameof, _patch = load_db(ROOT / args.db)
    fnames = field_name_map(classes)
    types, fields = unresolved_sets(classes, fnames)
    known = load_known(str(ROOT / args.hashes))
    t_states = len(types) + len(fields)
    print(f"[targets] {len(types)} unresolved classes + {len(fields)} "
          f"unresolved fields = {t_states} states", file=sys.stderr)

    tiers = build_tiers(idx)
    rows, hits = [], []
    for tier, probes in tiers.items():
        uniq = {}
        for probe, src in probes.items():
            uniq.setdefault(probe.lower(), (probe, src))
        exp = len(uniq) * t_states / 2 ** 32
        tally = collections.Counter()
        for _, (probe, src) in sorted(uniq.items()):
            h = fnv1a_32(probe)
            if h in known:
                verdict = "known"
            elif h in types:
                verdict = "type"
            elif h in fields:
                verdict = "field"
            else:
                verdict = "miss"
            tally[verdict] += 1
            rows.append((tier, probe, src, f"{h:08x}", verdict))
            if verdict in ("type", "field"):
                hits.append((tier, probe, src, h, verdict))
        print(f"\n=== tier {tier}: {len(uniq)} probes, {exp:.3f} expected "
              f"chance hits ===", file=sys.stderr)
        print("     " + ", ".join(f"{v} {k}" for k, v in tally.most_common()),
              file=sys.stderr)

    os.makedirs(os.path.dirname(args.out) or ".", exist_ok=True)
    with open(args.out, "w", encoding="utf-8", newline="\n") as f:
        print("tier\tprobe\tsource\thash\tverdict", file=f)
        for r in rows:
            print("\t".join(r), file=f)
    print(f"\n[out] {len(rows)} probe(s) -> {args.out}", file=sys.stderr)

    if args.names:
        wanted = set(args.names)
        hits = [h for h in hits if h[1] in wanted or h[2] in wanted]
        for n in args.names:
            if n not in idx:
                print(f"[warn] {n} is not in the reflection corpus",
                      file=sys.stderr)

    paths = {}
    if args.context:
        need = set()
        for _t, _p, src, _h, _v in hits:
            need |= idx[src]["sh"]
        paths = resolve_paths(need, args.paths)

    print(f"\n{len(hits)} hit(s) on an unresolved hash:")
    for tier, probe, src, h, verdict in sorted(hits):
        key = f"0x{h:x}"
        if verdict == "type":
            c = classes[key]
            bases = sorted({b for r in c["revisions"]
                            for b in r.get("bases", [])})
            ctx = ("base " + ", ".join(nameof(b) for b in bases)
                   if bases else "no base")
        else:
            owners = sorted(nameof(ch) for ch, c in classes.items()
                            if key in c.get("properties", {}) and c.get("name"))
            ctx = "on " + ", ".join(owners[:4]) if owners else \
                "unnamed owner(s) only"
        via = "" if probe == src else f" <- {src}"
        flag = "" if names_mod.is_valid_name(probe) else \
            f"  [needs recasing: {names_mod.why_invalid(probe)[:40]}]"
        print(f"  {verdict.upper():5} {h:08x} {probe}{via}  "
              f"[{tier}] ({ctx}){flag}")
        if args.context:
            e = idx[src]
            if e["cb"]:
                print("        cbuffer " + ", ".join(
                    f"{c} x{k}" for c, k in e["cb"].most_common(3)))
            print(f"        in {len(e['sh'])} shader(s):")
            for ph in sorted(e["sh"])[:5]:
                print(f"          {paths.get(ph, '(path unresolved) ' + ph)}")
            if len(e["sh"]) > 5:
                print(f"          ... {len(e['sh']) - 5} more")
    print("\n[note] a hash match is a candidate, not a crack. Read --context: "
          "a common word from an unrelated shader landing on a plausible field "
          "is the coincidence this corpus produces most.")
    return 0


if __name__ == "__main__":
    sys.exit(main())

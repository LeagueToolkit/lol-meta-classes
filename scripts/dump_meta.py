#!/bin/env python
import json
import sys
import os

def fnv1a(s):
    h = 0x811c9dc5
    for b in s.encode('ascii').lower():
        h = ((h ^ b) * 0x01000193) % 0x100000000
    return h

def readhashes(filename):
    with open(filename, "r") as inf:
        lines = [ x.rstrip().split(' ') for x in inf.readlines() ]
        return { hex(int(h, 16)) : v for h, v in lines }

def read_resolved_hashes(hashes_dir, table):
    """Vendored upstream mirror (hashes.<table>.txt) overlaid with local cracks
    (overrides/<table>.txt); an override wins on collision. Matches
    db_import.read_resolved_hashes - the blend is joined at use, not on disk."""
    names = readhashes(os.path.join(hashes_dir, f"hashes.{table}.txt"))
    override_path = os.path.join(hashes_dir, "overrides", f"{table}.txt")
    if os.path.exists(override_path):
        names.update(readhashes(override_path))
    return names

def read_meta(filename):
    def convert_type(t, is_old):
        if t == None: return None
        if is_old:
            if t >= 18 and t < 0x80: t = (t - 18) | 0x80
            if t >= 0x81: t += 1
        if t == 1: return "Bool"
        if t == 2: return "I8"
        if t == 3: return "U8"
        if t == 4: return "I16"
        if t == 5: return "U16"
        if t == 6: return "I32"
        if t == 7: return "U32"
        if t == 8: return "I64"
        if t == 9: return "U64"
        if t == 10: return "F32"
        if t == 11: return "Vec2"
        if t == 12: return "Vec3"
        if t == 13: return "Vec4"
        if t == 14: return "Mtx44"
        if t == 15: return "Color"
        if t == 16: return "String"
        if t == 17: return "Hash"
        if t == 18: return "File"
        if t == 0x80 | 0: return "List"
        if t == 0x80 | 1: return "List2"
        if t == 0x80 | 2: return "Pointer"
        if t == 0x80 | 3: return "Embed"
        if t == 0x80 | 4: return "Link"
        if t == 0x80 | 5: return "Option"
        if t == 0x80 | 6: return "Map"
        if t == 0x80 | 7: return "Flag"
        raise ValueError(f"Unknown old type: {hex(t)}")
    def convert_hex(h):
        if not h: return None
        return hex(h)
    def convert_digit(h):
        if not h: return None
        return int(h)
    def convert_container_storage(s, f):
        if f: return "Fixed"
        return None
    def convert_map_storage(s):
        if s == 0: return "StdMap"
        if s == 1: return "StdUnorderedMap"
        return None
    with open(filename) as inf:
        meta = json.load(inf)
        # No upgrade necessary
        if isinstance(meta["classes"], dict):
            return meta
        # Very old meta, type indexes need fixup
        is_old = tuple(int(x) for x in meta["version"].split('.')) < (10, 8)
        # Convert to new json format somehwat
        return {
            "classes": {
                hex(klass["hash"]) : {
                    "alignment": klass["alignment"],
                    "base": convert_hex(klass["parentClass"]),
                    "defaults": None,
                    "fn": {
                        "constructor": convert_hex(klass["constructor"]),
                        "destructor": convert_hex(klass["destructor"]),
                        "inplace_constructor": convert_hex(klass["inplaceconstructor"]),
                        "inplace_destructor": convert_hex(klass["inplacedestructor"]),
                        "register": convert_hex(klass["initfunction"]),
                        "upcast_secondary": convert_hex(klass["upcastSecondary"]),
                    },
                    "is": {
                        "interface": klass["isInterface"],
                        "property_base": klass["isPropertyBase"],
                        "secondary_base": klass["isSecondaryBase"],
                        "unk5": False if not "isUnk5" in klass else klass["isUnk5"],
                        "value": klass["isValue"],
                    },
                    "properties": {
                        hex(prop["hash"]): {
                            "bitmask": prop["bitmask"],
                            "container": None if not prop["containerI"] else {
                                "fixed_size": convert_digit(prop["containerI"]["fixedSize"]),
                                "storage": convert_map_storage(prop["containerI"]["fixedSize"]),
                                "value_size": prop["containerI"]["elemSize"],
                                "value_type": convert_type(prop["containerI"]["type"], is_old),
                                "vtable": convert_hex(prop["containerI"]["vtable"]),
                            },
                            "map": None if not prop["mapI"] else {
                                "key_type": convert_type(prop["mapI"]["key"], is_old),
                                "storage": convert_map_storage(prop["mapI"]["storage"]),
                                "value_type": convert_type(prop["mapI"]["value"], is_old),
                                "vtable": convert_hex(prop["mapI"]["vtable"]),
                            },
                            "offset": prop["offset"],
                            "other_class": convert_hex(prop["otherClass"]),
                            "value_type": convert_type(prop["type"], is_old),
                        } for prop in klass["properties"]
                    },
                    "secondary_bases": {
                        convert_hex(bhash): boffset for bhash, boffset in klass["secondaryBases"]
                    },
                    "secondary_children": {
                        convert_hex(chash): coffset for chash, coffset in klass["secondaryChildren"]
                    },
                    "size": klass["classSize"]
                } for klass in meta["classes"]
            }
        }

def h2type(h):
    if h in h_types:
        return h_types[h]
    unktypes.add(h)
    return "t_" + h

def h2field(h):
    if h in h_fields:
        return h_fields[h]
    unkfields.add(h)
    return "m_" + h

def n2ype(t, k):
    if t == "Bool": return "bool"
    if t == "I8": return "int8_t"
    if t == "U8": return "uint8_t"
    if t == "I16": return "int16_t"
    if t == "U16": return "uint16_t"
    if t == "I32": return "int32_t"
    if t == "U32": return "uint32_t"
    if t == "I64": return "int64_t"
    if t == "U64": return "uint64_t"
    if t == "F32": return "float_t"
    if t == "Vec2": return "point2D_t"
    if t == "Vec3": return "point3D_t"
    if t == "Vec4": return "point4D_t"
    if t == "Mtx44": return "matrix44_t"
    if t == "Color": return "color_t"
    if t == "String": return "string_t"
    if t == "Hash": return "hash_t"
    if t == "File": return "file_t"
    if t == "Flag": return "bool"
    if not k: raise Exception(f"other class is 0 for type: {t}")
    if t == "Pointer": return f"unique_ptr_t<{h2type(k)}>"
    if t == "Embed": return h2type(k)
    if t == "Link": return f"link_ptr_t<{h2type(k)}>"
    raise Exception(f"Nested type name too complex: {t}!")

def get_type(field):
    t = field["value_type"]
    if t == "List" or t == "List2":
        vt = n2ype(field["container"]["value_type"], field["other_class"])
        sz = field["container"]["fixed_size"]
        if sz: return "array_t<{}, {}>".format(vt, sz)
        return "vector_t<{}>".format(vt)
    if t == "Option":
        vt = n2ype(field["container"]["value_type"], field["other_class"])
        return "optional_t<{}>".format(vt)
    if t == "Map":
        kt = n2ype(field["map"]["key_type"], None)
        vt = n2ype(field["map"]["value_type"], field["other_class"])
        return f"map_t<{kt}, {vt}>"
    return n2ype(t, field["other_class"])

def find_field(klasses, khash, k):
    bases = [klasses[khash]]
    while len(bases):
        b = bases.pop()
        if k in b["properties"]:
            return b
        bb = b["base"]
        if bb:
            bases.append(klasses[bb])
        for bb in b["secondary_bases"]:
            bases.append(klasses[bb])
    return None

def dump_default_impl_value(klasses, value_type, khash, default, outf, indent = 1):
    if default == None:
        outf.write("nullptr")
        return
    match value_type:
        case "Bool" | "I8" | "U8" | "I16" | "U16" | "I32" | "U32" | "I64" | "U64" | "F32" | "String" | "Flag":
            outf.write(json.dumps(default))
        case "Vec2" | "Vec3" | "Vec4" | "Mtx44" | "Color":
            outf.write("{%s}" % (','.join(json.dumps(x) for x in default)))
        case "Link" | "Hash" | "File":
            outf.write(default)
        case "Embed" | "Pointer":
            if len(default) == 0:
                return outf.write(h2type(khash) + "{}")
            outf.write(h2type(khash) + " {\n")
            for k, v in default.items():
                outf.write('    ' * (indent + 1) + '.' + h2field(k) + " = ")
                found = find_field(klasses, khash, k)
                dump_default_impl(klasses, found["properties"][k], v, outf, indent + 1)
                outf.write(',\n')
            outf.write('    ' * indent + '}')
        case _:
            raise ValueError("Unhandled: " + value_type)


def dump_default_impl(klasses, field, default, outf, indent = 1):
    match field["value_type"]:
        case "Option":
            dump_default_impl_value(klasses, field["container"]["value_type"], field["other_class"], default, outf, indent)
        case "List" | "List2":
            if len(default) == 0:
                return outf.write("{}")
            outf.write("{\n")
            for item in default:
                o = outf.write('    ' * (indent + 1))
                o = dump_default_impl_value(klasses, field["container"]["value_type"], field["other_class"], item, outf, indent + 1)
                o = outf.write('\n')
            outf.write('    ' * indent + '}')
        case "Map":
            if len(default) == 0:
                return outf.write("{}")
            outf.write("{\n")
            for k, v in default:
                o = outf.write('    ' * (indent + 1) + '{')
                o = dump_default_impl_value(klasses, field["container"]["key_type"], None, k, outf, indent + 1)
                o = outf.write(',')
                o = dump_default_impl_value(klasses, field["container"]["value_type"], field["other_class"], v, outf, indent + 1)
                o = outf.write('},\n')
            outf.write('    ' * indent + '}')
        case _:
            dump_default_impl_value(klasses, field["value_type"], field["other_class"], default, outf, indent)

def dump_defaults(klasses, klass, fhash, outf):
    if klass["defaults"] and fhash in klass["defaults"]:
        dump_default_impl(klasses, klass["properties"][fhash], klass["defaults"][fhash], outf)
    else:
        outf.write("{}")

def dump_klass(klasses, khash, klass, outf):
    virtual = [ h2type(x) for x in klass["secondary_bases"].keys() ]
    prop = [ "PropertyBase" ] if klass["is"].get("property_base") else []
    normal = [ h2type(klass["base"]) ] if klass["base"] else prop
    bases = normal + virtual
    inheritance = (": " + ", ".join(bases)) if bases else ""
    o = outf.write(f"struct {h2type(khash)}{inheritance} {{\n")
    for fhash, field in sorted(klass["properties"].items(), key=lambda kvp: kvp[1]["offset"]):
        o = outf.write(f"    {get_type(field)} {h2field(fhash)} = ")
        o = dump_defaults(klasses, klass, fhash, outf)
        o = outf.write(";\n")
    o = outf.write("};\n")

def find_klass(klasses, h):
    if not h:
        return None
    for klass in meta_klasses["classes"]:
        if klass["hash"] == h:
            return klass
    return None

def build_deps(klasses, root_hashes, skip = []):
    q = []
    deps = set()
    done = set()
    for kh, k in klasses.items():
        if kh in root_hashes:
            q.append(kh)
            continue
        for fh in k["properties"].keys():
            if fh in root_hashes:
                q.append(kh)
                break
    done.add(hex(0))
    done.add(None)
    for x in skip:
        done.add(x)
    while len(q) > 0:
        h = q.pop()
        if h in done:
            continue
        done.add(h)
        deps.add(h)
        k = klasses[h]
        p = k["base"]
        if not p in done:
            q.append(p)
        for f in k["properties"].values():
            o = f["other_class"]
            if not (o in done):
                q.append(o)
        for h2, k2 in klasses.items():
            if h2 in done:
                continue
            if h2 == p:
                q.append(h2)
                continue
    return deps

def dump(klasses, outf, filterf = None):
    for khash, klass in klasses.items():
        if not filterf or filterf(khash, klass):
            dump_klass(klasses, khash, klass, outf)

if __name__ == '__main__':
    h_fields = read_resolved_hashes("hashes", "binfields")
    h_types = read_resolved_hashes("hashes", "bintypes")
    meta_klasses = read_meta(sys.argv[1])
    unktypes = set()
    unkfields = set()

    if len(sys.argv) > 2:
        root_hashes = { hex(fnv1a(x)) if not x.startswith('0x') else hex(int(x, 16)) for x in sys.argv[2:] }
        deps = build_deps(meta_klasses["classes"], root_hashes, [ ])
        dump(meta_klasses["classes"], sys.stdout, lambda kh, k: kh in deps)
    else:
        dump(meta_klasses["classes"], sys.stdout, None)
//! Pins the serialized shape of a dump.
//!
//! The deserialize tests prove old files still *read*. This proves the keys we
//! *write* have not moved: a stray `#[serde(rename)]` would otherwise change
//! every future dump silently, with nothing failing until a downstream consumer
//! broke against files that cannot be regenerated.
//!
//! When a change here is intentional, bump `FORMAT_VERSION` and update
//! `tests/golden/dump.json` in the same commit.

use lol_meta_schema::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn golden_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/dump.json")
}

/// One of everything: both nullable branches of a class, a property with a
/// container, one with a map, one with a hash helper, and a plain scalar.
fn specimen() -> MetaDump {
    let container_property = PropertyDump {
        other_class: Some("0xcafebabe".to_string()),
        offset: 16,
        bitmask: 0,
        value_type: BinType::List,
        container: Some(ContainerDump {
            vtable: "0x2512c60".to_string(),
            value_type: BinType::U32,
            value_size: 4,
            fixed_size: None,
            storage: Some(ContainerStorage::UnknownVector),
        }),
        map: None,
        hasher: None,
        unkptr: "0x0".to_string(),
    };

    let map_property = PropertyDump {
        other_class: None,
        offset: 32,
        bitmask: 0,
        value_type: BinType::Map,
        container: None,
        map: Some(MapDump {
            vtable: "0x2512d00".to_string(),
            key_type: BinType::Hash,
            value_type: BinType::Embed,
            storage: MapStorage::UnknownMap,
        }),
        hasher: None,
        unkptr: "0x0".to_string(),
    };

    let hash_property = PropertyDump {
        other_class: None,
        offset: 40,
        bitmask: 0,
        value_type: BinType::Hash,
        container: None,
        map: None,
        hasher: Some("0x2512e40".to_string()),
        unkptr: "0x0".to_string(),
    };

    let flag_property = PropertyDump {
        other_class: None,
        offset: 8,
        bitmask: 0x4,
        value_type: BinType::Flag,
        container: None,
        map: None,
        hasher: None,
        unkptr: "0x0".to_string(),
    };

    let concrete = ClassDump {
        base: Some("0x91873e8a".to_string()),
        secondary_bases: BTreeMap::from([("0xb0607142".to_string(), 8u32)]),
        secondary_children: BTreeMap::new(),
        size: 48,
        alignment: 8,
        flags: ClassFlags {
            interface: false,
            value: false,
            secondary_base: false,
            finalized: false,
        },
        functions: ClassFunctions {
            upcast_secondary: None,
            constructor: Some("0x2e3e0".to_string()),
            destructor: Some("0x2e410".to_string()),
            inplace_constructor: Some("0x2e430".to_string()),
            inplace_destructor: Some("0x2e460".to_string()),
            register: None,
        },
        properties: BTreeMap::from([
            ("0x4bc37f00".to_string(), flag_property),
            ("0x8e2676a9".to_string(), container_property),
            ("0x9a1b2c3d".to_string(), map_property),
            ("0xc0ffee11".to_string(), hash_property),
        ]),
        defaults: Some(BTreeMap::from([(
            "0x4bc37f00".to_string(),
            serde_json::Value::Bool(false),
        )])),
    };

    // An interface: no constructor, and `defaults` is null rather than empty.
    let interface = ClassDump {
        base: None,
        secondary_bases: BTreeMap::new(),
        secondary_children: BTreeMap::from([("0x1003c990".to_string(), 0u32)]),
        size: 0,
        alignment: 1,
        flags: ClassFlags {
            interface: true,
            value: false,
            secondary_base: true,
            finalized: true,
        },
        functions: ClassFunctions {
            upcast_secondary: Some("0x1234".to_string()),
            constructor: None,
            destructor: None,
            inplace_constructor: None,
            inplace_destructor: None,
            register: Some("0x5678".to_string()),
        },
        properties: BTreeMap::new(),
        defaults: None,
    };

    MetaDump {
        format_version: FORMAT_VERSION,
        version: "16.14.7949266".to_string(),
        hashers: BTreeMap::from([(
            "0x2512e40".to_string(),
            HasherDump {
                storage_width: 4,
                hash_function: HashFunction {
                    algorithm: HashAlgorithm::Fnv1a32,
                    lowercased: true,
                },
            },
        )]),
        classes: BTreeMap::from([
            ("0xfa33b8e8".to_string(), concrete),
            ("0x635d04b7".to_string(), interface),
        ]),
    }
}

fn serialize(dump: &MetaDump) -> String {
    let mut out = serde_json::to_string_pretty(dump).expect("serializing the specimen");
    out.push('\n');
    out
}

#[test]
fn serialized_shape_matches_the_golden_file() {
    let actual = serialize(&specimen());
    let path = golden_path();

    // UPDATE_GOLDEN=1 rewrites the fixture; review the diff before committing.
    if std::env::var("UPDATE_GOLDEN").as_deref() == Ok("1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &actual).unwrap();
        eprintln!("rewrote {}", path.display());
        return;
    }

    let expected = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "reading {}: {e}\nRun with UPDATE_GOLDEN=1 to create it.",
            path.display()
        )
    });

    assert_eq!(
        expected.replace("\r\n", "\n"),
        actual.replace("\r\n", "\n"),
        "serialized dump shape changed.\nIf intentional: bump FORMAT_VERSION and \
         re-run with UPDATE_GOLDEN=1."
    );
}

#[test]
fn the_specimen_round_trips() {
    let dump = specimen();
    let parsed: MetaDump = serde_json::from_str(&serialize(&dump)).expect("round trip");

    assert_eq!(parsed.format_version, FORMAT_VERSION);
    assert_eq!(parsed.version, dump.version);
    assert_eq!(parsed.classes.len(), dump.classes.len());

    // The two branches that are easy to get wrong: an interface's null defaults
    // must stay null, not become an empty map.
    assert!(parsed.classes["0x635d04b7"].defaults.is_none());
    assert!(parsed.classes["0xfa33b8e8"].defaults.is_some());

    // A dangling hasher key is the silent failure mode of interning.
    let hash_property = &parsed.classes["0xfa33b8e8"].properties["0xc0ffee11"];
    let hasher = parsed
        .hasher(hash_property)
        .expect("the interned helper must resolve after a round trip");
    assert_eq!(hasher.storage_width, 4);
}

#[test]
fn new_dumps_carry_the_current_format_version() {
    assert_ne!(
        FORMAT_VERSION, FORMAT_VERSION_UNVERSIONED,
        "the current version must be distinguishable from an absent field"
    );
}

//! Schema types for League of Legends metaclass dumps.
//!
//! This crate provides typed structures for deserializing the JSON dumps
//! produced by the `dumper` tool. It allows consumers to work with strongly-typed
//! data instead of raw `serde_json::Value`.
//!
//! # Example
//!
//! ```no_run
//! use lol_meta_schema::MetaDump;
//! use std::fs::File;
//!
//! let file = File::open("dumps/14.24.6442327.json").unwrap();
//! let dump: MetaDump = serde_json::from_reader(file).unwrap();
//!
//! println!("Version: {}", dump.version);
//! println!("Classes: {}", dump.classes.len());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema version written into new dumps.
///
/// Bump on any change to the shape or meaning of the fields below, and handle
/// the older value on read. Dumps are historical records of what a given build
/// contained and are never regenerated, so compatibility is a read-side concern:
/// this crate tolerates every version ever written, while the dumper only ever
/// emits the current one.
///
/// History:
///   1 - the field was introduced; shape otherwise as dumped up to 16.14.
///   2 - `ClassFlags::unk5` renamed to `finalized`.
///   3 - hash helpers added: a `MetaDump::hashers` table, and `PropertyDump::hasher`
///       naming the entry behind a `Hash` property.
pub const FORMAT_VERSION: u32 = 3;

/// Version of dumps written before the field existed.
pub const FORMAT_VERSION_UNVERSIONED: u32 = 0;

/// Root structure for a metaclass dump file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaDump {
    /// Schema version of this file. Absent in every dump written before the
    /// field was introduced, which reads back as [`FORMAT_VERSION_UNVERSIONED`].
    #[serde(rename = "formatVersion", default)]
    pub format_version: u32,
    /// Game version string (e.g., "14.24.6442327").
    pub version: String,
    /// Hash helpers referenced by [`PropertyDump::hasher`], keyed by vtable
    /// offset (hex string). Interned: an image has a handful of helpers behind
    /// thousands of `Hash` properties. Empty before format version 3.
    #[serde(default)]
    pub hashers: BTreeMap<String, HasherDump>,
    /// Map of class hash (hex string) to class definition.
    pub classes: BTreeMap<String, ClassDump>,
}

impl MetaDump {
    /// Resolves a property's hash helper.
    ///
    /// `None` if the property carries no helper, including every dump written
    /// before format version 3.
    pub fn hasher(&self, property: &PropertyDump) -> Option<&HasherDump> {
        self.hashers.get(property.hasher.as_ref()?)
    }
}

/// A metaclass definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassDump {
    /// Base class hash (hex string), if any.
    pub base: Option<String>,
    /// Secondary base classes with their offsets.
    pub secondary_bases: BTreeMap<String, u32>,
    /// Secondary child classes with their offsets.
    pub secondary_children: BTreeMap<String, u32>,
    /// Size of the class in bytes.
    pub size: usize,
    /// Alignment requirement.
    pub alignment: usize,
    /// Class flags.
    #[serde(rename = "is")]
    pub flags: ClassFlags,
    /// Class function pointers.
    #[serde(rename = "fn")]
    pub functions: ClassFunctions,
    /// Map of property hash (hex string) to property definition.
    pub properties: BTreeMap<String, PropertyDump>,
    /// Default values for properties (null for interfaces).
    pub defaults: Option<BTreeMap<String, serde_json::Value>>,
}

/// Flags describing class characteristics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassFlags {
    /// True if this is an interface (no constructor).
    pub interface: bool,
    /// True if this is a value type.
    pub value: bool,
    /// True if this is a secondary base class.
    pub secondary_base: bool,
    /// An "already processed" marker - the metaclass registry sets it when it finalises an entry.
    ///
    /// Written as `unk5` before format version 2, which is what every dump up to
    /// 16.14 carries
    #[serde(alias = "unk5")]
    pub finalized: bool,
}

/// Function pointers for class operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassFunctions {
    /// Upcast to secondary base function.
    pub upcast_secondary: Option<String>,
    /// Constructor function.
    pub constructor: Option<String>,
    /// Destructor function.
    pub destructor: Option<String>,
    /// In-place constructor function.
    pub inplace_constructor: Option<String>,
    /// In-place destructor function.
    pub inplace_destructor: Option<String>,
    /// Register function.
    pub register: Option<String>,
}

/// A property definition within a class.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDump {
    /// Other class hash for Link/Pointer/Embed types.
    pub other_class: Option<String>,
    /// Byte offset within the class.
    pub offset: u32,
    /// Bitmask for Flag types.
    pub bitmask: u8,
    /// The property's value type.
    pub value_type: BinType,
    /// Container info for List/Option types.
    pub container: Option<ContainerDump>,
    /// Map info for Map types.
    pub map: Option<MapDump>,
    /// Key into [`MetaDump::hashers`] for `Hash` types. Absent before format
    /// version 3, null where no helper is installed (pre-16.1).
    #[serde(default)]
    pub hasher: Option<String>,
    /// Unknown pointer (always "0x0").
    pub unkptr: String,
}

/// Container type information (for List, List2, Option).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDump {
    /// Vtable offset (hex string).
    pub vtable: String,
    /// Type of values in the container.
    pub value_type: BinType,
    /// Size of each value in bytes.
    pub value_size: usize,
    /// Fixed size for fixed-length arrays, if applicable.
    pub fixed_size: Option<usize>,
    /// Storage type, if known.
    pub storage: Option<ContainerStorage>,
}

/// A hash helper, interned in [`MetaDump::hashers`] under its vtable offset.
///
/// A `Hash` tag names neither a hash function nor a width; both come from this
/// helper, and the reader asks it how many bytes to consume. Anything writing
/// a `Hash` value must consult it: two `Hash` properties can disagree on both
/// counts.
///
/// The vtable offset is the key because it is the helper's identity: it
/// separates helpers that agree on every field below.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasherDump {
    /// Bytes the helper stores, from its `get_size`. 4 everywhere so far;
    /// the reader honours 8 since 16.14 but nothing returns it yet.
    pub storage_width: usize,
    /// The hash computed for a string, measured by calling the helper.
    pub hash_function: HashFunction,
}

/// A hash function identified by probing a helper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HashFunction {
    /// The algorithm, or [`HashAlgorithm::Unknown`] if nothing matched.
    pub algorithm: HashAlgorithm,
    /// Whether the helper lowercases before hashing. Meaningless when the
    /// algorithm is unknown.
    pub lowercased: bool,
}

/// Hash algorithms a helper has been observed to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// The classic bin hash: FNV-1a, 32-bit.
    Fnv1a32,
    /// XXH64, the same hash as a WAD chunk path.
    Xxh64,
    /// XXH3, 64-bit output.
    Xxh3,
    /// Probed, but nothing matched.
    Unknown,
}

/// Map type information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDump {
    /// Vtable offset (hex string).
    pub vtable: String,
    /// Type of keys in the map.
    pub key_type: BinType,
    /// Type of values in the map.
    pub value_type: BinType,
    /// Storage type.
    pub storage: MapStorage,
}

/// Binary property types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BinType {
    None,
    Bool,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    Vec2,
    Vec3,
    Vec4,
    Mtx44,
    Color,
    String,
    Hash,
    File,
    List,
    List2,
    Pointer,
    Embed,
    Link,
    Option,
    Map,
    Flag,
}

/// Container storage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerStorage {
    UnknownVector,
    Option,
    Fixed,
    StdVector,
    RitoVector,
}

/// Map storage types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapStorage {
    UnknownMap,
    StdMap,
    StdUnorderedMap,
    RitoVectorMap,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal() {
        let json = r#"{
            "version": "14.24.6442327",
            "classes": {}
        }"#;
        let dump: MetaDump = serde_json::from_str(json).unwrap();
        assert_eq!(dump.version, "14.24.6442327");
        assert!(dump.classes.is_empty());
    }

    #[test]
    fn the_pre_v2_unk5_key_still_reads() {
        // Every dump up to 16.14 spells this `unk5`. Those files cannot be
        // regenerated, so dropping the alias would orphan the whole corpus.
        let flags: ClassFlags = serde_json::from_str(
            r#"{"interface":false,"value":false,"secondary_base":false,"unk5":true}"#,
        )
        .expect("the old key must still deserialize");
        assert!(flags.finalized);
    }

    #[test]
    fn the_current_finalized_key_reads() {
        let flags: ClassFlags = serde_json::from_str(
            r#"{"interface":false,"value":false,"secondary_base":false,"finalized":true}"#,
        )
        .unwrap();
        assert!(flags.finalized);
    }

    /// A pre-v3 dump has neither the table nor the key.
    #[test]
    fn a_dump_without_hashers_still_reads() {
        let dump: MetaDump = serde_json::from_str(
            r#"{"version":"14.24.6442327","classes":{}}"#,
        )
        .unwrap();
        assert!(dump.hashers.is_empty());
    }

    #[test]
    fn a_property_resolves_its_hasher() {
        let dump: MetaDump = serde_json::from_str(
            r#"{
                "formatVersion": 3,
                "version": "16.17.8068374",
                "hashers": {
                    "0x25d0ff0": {
                        "storage_width": 4,
                        "hash_function": {"algorithm": "Fnv1a32", "lowercased": true}
                    }
                },
                "classes": {}
            }"#,
        )
        .unwrap();

        let property: PropertyDump = serde_json::from_str(
            r#"{
                "other_class": null, "offset": 40, "bitmask": 0,
                "value_type": "Hash", "container": null, "map": null,
                "hasher": "0x25d0ff0", "unkptr": "0x0"
            }"#,
        )
        .unwrap();

        let hasher = dump.hasher(&property).expect("the key must resolve");
        assert_eq!(hasher.storage_width, 4);
        assert_eq!(hasher.hash_function.algorithm, HashAlgorithm::Fnv1a32);

        // No key resolves to nothing, not to an arbitrary entry.
        let no_hasher = PropertyDump {
            hasher: None,
            ..property
        };
        assert!(dump.hasher(&no_hasher).is_none());
    }

    #[test]
    fn flags_are_written_under_the_new_name() {
        let flags = ClassFlags {
            interface: false,
            value: false,
            secondary_base: false,
            finalized: true,
        };
        let json = serde_json::to_string(&flags).unwrap();
        assert!(json.contains("\"finalized\""), "{json}");
        assert!(!json.contains("unk5"), "the alias must not be written: {json}");
    }
}

use core::ffi::c_void;
use core::fmt::LowerHex;
use std::collections::BTreeMap;

use lol_meta_schema::{
    BinType as SchemaBinType, ClassDump, ClassFlags, ClassFunctions, ContainerDump,
    ContainerStorage as SchemaContainerStorage, MapDump, MapStorage as SchemaMapStorage, MetaDump,
    PropertyDump,
};
use serde_json::Value;

use crate::meta::*;

fn dump_hex<T: Copy + LowerHex>(value: T) -> String {
    format!("0x{:x}", value)
}

// Convert internal BinType to schema BinType
fn convert_bin_type(t: BinType) -> SchemaBinType {
    match t {
        BinType::None => SchemaBinType::None,
        BinType::Bool => SchemaBinType::Bool,
        BinType::I8 => SchemaBinType::I8,
        BinType::U8 => SchemaBinType::U8,
        BinType::I16 => SchemaBinType::I16,
        BinType::U16 => SchemaBinType::U16,
        BinType::I32 => SchemaBinType::I32,
        BinType::U32 => SchemaBinType::U32,
        BinType::I64 => SchemaBinType::I64,
        BinType::U64 => SchemaBinType::U64,
        BinType::F32 => SchemaBinType::F32,
        BinType::Vec2 => SchemaBinType::Vec2,
        BinType::Vec3 => SchemaBinType::Vec3,
        BinType::Vec4 => SchemaBinType::Vec4,
        BinType::Mtx44 => SchemaBinType::Mtx44,
        BinType::Color => SchemaBinType::Color,
        BinType::String => SchemaBinType::String,
        BinType::Hash => SchemaBinType::Hash,
        BinType::File => SchemaBinType::File,
        BinType::List => SchemaBinType::List,
        BinType::List2 => SchemaBinType::List2,
        BinType::Pointer => SchemaBinType::Pointer,
        BinType::Embed => SchemaBinType::Embed,
        BinType::Link => SchemaBinType::Link,
        BinType::Option => SchemaBinType::Option,
        BinType::Map => SchemaBinType::Map,
        BinType::Flag => SchemaBinType::Flag,
        // Reached when a record is read at the wrong offset, so the raw
        // discriminant is the useful part of the message.
        _ => panic!("Unknown BinType {:#04x}", t as u8),
    }
}

fn convert_container_storage(s: ContainerStorage) -> SchemaContainerStorage {
    match s {
        ContainerStorage::UnknownVector => SchemaContainerStorage::UnknownVector,
        ContainerStorage::Option => SchemaContainerStorage::Option,
        ContainerStorage::Fixed => SchemaContainerStorage::Fixed,
        ContainerStorage::StdVector => SchemaContainerStorage::StdVector,
        ContainerStorage::RitoVector => SchemaContainerStorage::RitoVector,
    }
}

fn convert_map_storage(s: MapStorage) -> SchemaMapStorage {
    match s {
        MapStorage::UnknownMap => SchemaMapStorage::UnknownMap,
        MapStorage::StdMap => SchemaMapStorage::StdMap,
        MapStorage::StdUnorderedMap => SchemaMapStorage::StdUnorderedMap,
        MapStorage::RitoVectorMap => SchemaMapStorage::RitoVectorMap,
    }
}

// ============================================================================
// Instance dumping (for default values) - these return serde_json::Value
// ============================================================================

fn dump_instance_bool(instance: usize) -> Value {
    let result = unsafe { *(instance as *const c_void as *const u8) };
    (result != 0).into()
}

fn dump_instance_num<T: Copy + Into<Value>>(instance: usize) -> Value {
    let result = unsafe { *(instance as *const c_void as *const T) };
    result.into()
}

fn dump_instance_vec<T: Copy + Into<Value>, const X: usize>(instance: usize) -> Value {
    let result = unsafe { *(instance as *const c_void as *const [T; X]) };
    result.to_vec().into()
}

fn dump_instance_mtx44(instance: usize) -> Value {
    let result = unsafe { *(instance as *const c_void as *const [[f32; 4]; 4]) };
    let mut results = Vec::<Value>::new();
    for item in result.iter() {
        results.push(item.to_vec().into());
    }
    results.into()
}

fn dump_instance_string(instance: usize) -> Value {
    let result = unsafe { &*(instance as *const c_void as *const AString) };
    let s = result.str();
    if s.len() > 10_000 {
        eprintln!(
            "WARNING: string property has length {}, likely garbage. Truncating.",
            s.len()
        );
        return Value::String(String::new());
    }
    s.into()
}

fn dump_instance_hash(instance: usize) -> Value {
    let result = unsafe { *(instance as *const c_void as *const u32) };
    dump_hex(result).into()
}

fn dump_instance_link(instance: usize, _class: &Class) -> Value {
    dump_instance_hash(instance)
}

fn dump_instance_path(instance: usize) -> Value {
    let result = unsafe { *(instance as *const c_void as *const u64) };
    dump_hex(result).into()
}

fn dump_instance_embed(instance: usize, class: &Class) -> Value {
    let mut results = serde_json::Map::new();
    dump_instance_properties(class, instance, &mut results);
    results.into()
}

fn dump_instance_pointer(instance: usize, class: &Class) -> Value {
    let instance = unsafe { *(instance as *const usize) };
    if instance != 0 {
        return dump_instance_embed(instance, class);
    }
    Value::Null
}

fn dump_instance_list(instance: usize, container: &ContainerI, class: Option<&Class>) -> Value {
    let size = container.get_size(instance);
    let mut result = Vec::<Value>::new();
    for index in 0..size {
        let item_instance = container.get_const(instance, index);
        let item = dump_instance_nestable(item_instance, container.value_type, class);
        result.push(item);
    }
    result.into()
}

fn dump_instance_map(instance: usize, map: &MapI, _class: Option<&Class>) -> Value {
    let size = map.get_size(instance);
    if size != 0 {
        eprintln!(
            "WARNING: map size {} is non-zero on default instance, likely uninitialized memory. Skipping.",
            size
        );
    }
    serde_json::Map::new().into()
}

fn dump_instance_flag(instance: usize, bitmask: u8) -> Value {
    let result = unsafe { *(instance as *const c_void as *const u8) };
    ((result & bitmask) != 0).into()
}

fn dump_instance_option(instance: usize, container: &ContainerI, class: Option<&Class>) -> Value {
    match container.get_size(instance) {
        0 => Value::Null,
        _ => dump_instance_nestable(instance, container.value_type, class),
    }
}

fn dump_instance_nestable(instance: usize, item_type: BinType, class: Option<&Class>) -> Value {
    match item_type {
        BinType::None => panic!("Trying to print None!"),
        BinType::Bool => dump_instance_bool(instance),
        BinType::I8 => dump_instance_num::<i8>(instance),
        BinType::U8 => dump_instance_num::<u8>(instance),
        BinType::I16 => dump_instance_num::<i16>(instance),
        BinType::U16 => dump_instance_num::<u16>(instance),
        BinType::I32 => dump_instance_num::<i32>(instance),
        BinType::U32 => dump_instance_num::<u32>(instance),
        BinType::I64 => dump_instance_num::<i64>(instance),
        BinType::U64 => dump_instance_num::<u64>(instance),
        BinType::F32 => dump_instance_num::<f32>(instance),
        BinType::Vec2 => dump_instance_vec::<f32, 2>(instance),
        BinType::Vec3 => dump_instance_vec::<f32, 3>(instance),
        BinType::Vec4 => dump_instance_vec::<f32, 4>(instance),
        BinType::Mtx44 => dump_instance_mtx44(instance),
        BinType::Color => dump_instance_vec::<u8, 4>(instance),
        BinType::String => dump_instance_string(instance),
        BinType::Hash => dump_instance_hash(instance),
        BinType::Link => dump_instance_link(instance, class.expect("Link needs class!")),
        BinType::File => dump_instance_path(instance),
        BinType::List => panic!("List is not nestable!"),
        BinType::List2 => panic!("List is not nestable!"),
        BinType::Map => panic!("Map is not nestable!"),
        BinType::Pointer => dump_instance_pointer(instance, class.expect("Pointer needs class!")),
        BinType::Embed => dump_instance_embed(instance, class.expect("Embed needs class!")),
        BinType::Option => panic!("Option is not nestable!"),
        BinType::Flag => panic!("Flag is not nestable!"),
    }
}

/// Describe a property for a diagnostic message. A broken invariant here almost
/// always means the record was read at the wrong offset or stride, so the raw
/// field values are what identify the misread.
fn property_context(property: &Property) -> String {
    format!(
        "class {} property {} type {:?} offset {} bitmask {:#x} container {} map {}",
        dump_hex(crate::diag::current_class()),
        dump_hex(property.hash),
        property.value_type,
        property.offset,
        property.bitmask,
        if property.container().is_some() { "set" } else { "NULL" },
        if property.map().is_some() { "set" } else { "NULL" },
    ) + &format!(" union_slot {:#x}", property.union_raw())
}

/// Resolve a pointer a property's type says must be present.
///
/// Panics by default; under `DUMPER_TOLERATE_ANOMALIES` it records the
/// inconsistency and lets the walk continue so the full extent is visible.
fn require<'a, T>(value: Option<&'a T>, what: &str, property: &Property) -> Option<&'a T> {
    if value.is_none() {
        let message = format!("{what}: {}", property_context(property));
        if crate::diag::tolerate_anomalies() {
            crate::diag::record_anomaly(&message);
        } else {
            panic!("{message}");
        }
    }
    value
}

fn dump_instance_property(instance: usize, property: &Property) -> Value {
    let instance = instance + property.offset as usize;
    match property.value_type {
        BinType::List | BinType::List2 => {
            match require(property.container(), "List needs container", property) {
                Some(container) => dump_instance_list(instance, container, property.other_class),
                None => Value::Null,
            }
        }
        BinType::Map => match require(property.map(), "Map needs map", property) {
            Some(map) => dump_instance_map(instance, map, property.other_class),
            None => Value::Null,
        },
        BinType::Option => {
            match require(property.container(), "Option needs container", property) {
                Some(container) => dump_instance_option(instance, container, property.other_class),
                None => Value::Null,
            }
        }
        BinType::Flag => dump_instance_flag(instance, property.bitmask),
        _ => dump_instance_nestable(instance, property.value_type, property.other_class),
    }
}

fn dump_instance_properties(
    class: &Class,
    instance: usize,
    results: &mut serde_json::Map<String, Value>,
) {
    if let Some(class) = class.base_class {
        dump_instance_properties(class, instance, results);
    }
    for &BaseOff(class, offset) in class.secondary_bases.slice() {
        dump_instance_properties(class, instance + offset as usize, results);
    }
    for property in class.iter_properties() {
        let key = dump_hex(property.hash);
        let value = dump_instance_property(instance, property);
        results.insert(key, value);
    }
}

// ============================================================================
// Schema type construction
// ============================================================================

fn dump_property_container(base: usize, container: &ContainerI, source: BinType) -> ContainerDump {
    ContainerDump {
        vtable: dump_hex(container.vtable as *const _ as usize - base),
        value_type: convert_bin_type(container.value_type),
        value_size: container.value_size,
        fixed_size: container.get_fixed_size(),
        storage: (source != BinType::Option)
            .then(|| convert_container_storage(container.get_storage())),
    }
}

fn dump_property_map(base: usize, map: &MapI) -> MapDump {
    MapDump {
        vtable: dump_hex(map.vtable as *const _ as usize - base),
        key_type: convert_bin_type(map.key_type),
        value_type: convert_bin_type(map.value_type),
        storage: convert_map_storage(map.get_storage()),
    }
}

fn dump_property(base: usize, property: &Property) -> PropertyDump {
    PropertyDump {
        other_class: property.other_class.map(|c| dump_hex(c.hash)),
        offset: property.offset,
        bitmask: property.bitmask,
        value_type: convert_bin_type(property.value_type),
        container: property
            .container()
            .map(|c| dump_property_container(base, c, property.value_type)),
        map: property.map().map(|m| dump_property_map(base, m)),
        unkptr: dump_hex(0usize),
    }
}

fn dump_property_list<'a>(
    base: usize,
    properties: impl Iterator<Item = &'a Property>,
) -> BTreeMap<String, PropertyDump> {
    let mut results = BTreeMap::new();
    for property in properties {
        let key = dump_hex(property.hash);
        let value = dump_property(base, property);
        results.insert(key, value);
    }
    results
}

fn dump_class_functions(base: usize, class: &Class) -> ClassFunctions {
    ClassFunctions {
        upcast_secondary: class
            .upcast_secondary_fn
            .map(|c| dump_hex(c as usize - base)),
        constructor: class.constructor_fn.map(|c| dump_hex(c as usize - base)),
        destructor: class.destructor_fn.map(|c| dump_hex(c as usize - base)),
        inplace_constructor: class
            .inplace_constructor_fn
            .map(|c| dump_hex(c as usize - base)),
        inplace_destructor: class
            .inplace_destructor_fn
            .map(|c| dump_hex(c as usize - base)),
        register: class.register_fn.map(|c| dump_hex(c as usize - base)),
    }
}

fn dump_class_flags(class: &Class) -> ClassFlags {
    ClassFlags {
        interface: class.constructor_fn.is_none(),
        value: class.is_value,
        secondary_base: class.is_secondary_base,
        unk5: class.is_unk5,
    }
}

/// Raw view of [`BaseOff`], whose first field is a non-nullable reference.
#[repr(C)]
#[derive(Clone, Copy)]
struct RawBaseOff {
    class: *const Class,
    offset: u32,
}

fn dump_class_secondary(class_offset_pairs: &[BaseOff], what: &str) -> BTreeMap<String, u32> {
    let mut results = BTreeMap::new();

    // Same reasoning as the class vector: read the pointer before trusting it,
    // so a null reports itself instead of faulting at `Class+0x08`.
    let raw: &[RawBaseOff] = unsafe {
        std::slice::from_raw_parts(class_offset_pairs.as_ptr().cast(), class_offset_pairs.len())
    };

    for (index, entry) in raw.iter().enumerate() {
        if entry.class.is_null() {
            let message = format!(
                "null class pointer in {what}[{index}] of class {}",
                dump_hex(crate::diag::current_class())
            );
            if crate::diag::tolerate_anomalies() {
                crate::diag::record_anomaly(&message);
                continue;
            }
            panic!("{message}");
        }
        let class = unsafe { &*entry.class };
        results.insert(dump_hex(class.hash), entry.offset);
    }
    results
}

fn is_empty(class: &Class) -> bool {
    // FIXME: 16.1+ these depend on game flow being started
    let blacklist: &[u32] = &[
        0xfea4e3fe,
        0xe501834f,
    ];
    if blacklist.contains(&class.hash) {
        return true;
    }
    class.properties.size() == 0 && class.base_class.iter().all(|class| is_empty(class))
}

pub fn dump_class_defaults(class: &Class) -> Option<BTreeMap<String, Value>> {
    use crate::diag::{set_phase, Phase};

    if class.constructor_fn.is_some() {
        let mut results = serde_json::Map::new();
        if !is_empty(class) {
            // Each of these runs shipped game code and can fault. Narrowing the
            // phase means a crash report says which of the three it was.
            set_phase(Phase::ClassConstructor);
            let instance = class.create_instance();
            set_phase(Phase::ClassDefaults);
            dump_instance_properties(class, instance, &mut results);
            set_phase(Phase::ClassDestructor);
            class.destroy_instance(instance);
        }
        Some(results.into_iter().collect())
    } else {
        None
    }
}

pub fn dump_class(base: usize, class: &Class) -> ClassDump {
    ClassDump {
        base: class.base_class.map(|c| dump_hex(c.hash)),
        secondary_bases: dump_class_secondary(class.secondary_bases.slice(), "secondary_bases"),
        secondary_children: dump_class_secondary(
            class.secondary_children.slice(),
            "secondary_children",
        ),
        size: class.class_size,
        alignment: class.alignment,
        flags: dump_class_flags(class),
        functions: dump_class_functions(base, class),
        properties: dump_property_list(base, class.iter_properties()),
        defaults: dump_class_defaults(class),
    }
}

pub fn dump_class_list(base: usize, classes: &[&Class]) -> BTreeMap<String, ClassDump> {
    let mut results = BTreeMap::new();
    let slots: &[*const Class] =
        unsafe { std::slice::from_raw_parts(classes.as_ptr().cast(), classes.len()) };

    for (index, &ptr) in slots.iter().enumerate() {
        crate::diag::set_current_slot(index, ptr as usize);

        if ptr.is_null() {
            let message = format!("null class pointer at vector index {index}");
            if crate::diag::tolerate_anomalies() {
                crate::diag::record_anomaly(&message);
                continue;
            }
            panic!("{message}");
        }
        let class: &Class = unsafe { &*ptr };

        // Published before any work on this class so a fault handler can name
        // the class that killed us.
        crate::diag::set_current_class(class.hash);
        crate::diag::set_phase(crate::diag::Phase::ClassMetadata);
        if crate::diag::dump_class_filter() == Some(class.hash) {
            let (data, count) = class.properties.raw_parts();
            crate::diag::hexdump("class", class as *const _ as usize, 0x88);
            // Two records' worth at the larger stride, so both candidate
            // layouts are visible in one dump.
            crate::diag::hexdump("properties", data, (count * 56).min(512));
        }
        let key = dump_hex(class.hash);
        let value = dump_class(base, class);
        results.insert(key, value);
    }
    crate::diag::set_current_class(0);
    results
}

pub fn dump_meta(base: usize, classes: &[&Class], version: String) -> MetaDump {
    MetaDump {
        version,
        classes: dump_class_list(base, classes),
    }
}

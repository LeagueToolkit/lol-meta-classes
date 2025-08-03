use core::ffi::c_void;
use core::fmt::LowerHex;

use serde_json::json;
use serde_json::{Map, Value};

use crate::meta::*;

fn dump_hex<T: Copy + LowerHex>(value: T) -> String {
    format!("0x{:x}", value)
}

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
    result.str().into()
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
    let mut results = Map::new();
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
    assert_eq!(map.get_size(instance), 0, "Map is not empty");
    Map::new().into()
}

fn dump_instance_flag(instance: usize, bitmask: u8) -> Value {
    let result = unsafe { *(instance as *const c_void as *const u8) };
    ((result & (1 << bitmask)) != 0).into()
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

fn dump_instance_property(instance: usize, property: &Property) -> Value {
    let instance = instance + property.offset as usize;
    match property.value_type {
        BinType::List | BinType::List2 => dump_instance_list(
            instance,
            property.container.expect("List needs container"),
            property.other_class,
        ),
        BinType::Map => dump_instance_map(
            instance,
            property.map.expect("Map needs map"),
            property.other_class,
        ),
        BinType::Option => dump_instance_option(
            instance,
            property.container.expect("Option needs container"),
            property.other_class,
        ),
        BinType::Flag => dump_instance_flag(instance, property.bitmask),
        _ => dump_instance_nestable(instance, property.value_type, property.other_class),
    }
}

fn dump_instance_properties(class: &Class, instance: usize, results: &mut Map<String, Value>) {
    if let Some(class) = class.base_class {
        dump_instance_properties(class, instance, results);
    }
    for &BaseOff(class, offset) in class.secondary_bases.slice() {
        dump_instance_properties(class, instance + offset as usize, results);
    }
    for property in class.properties.slice() {
        let key = dump_hex(property.hash);
        let value = dump_instance_property(instance, property);
        results.insert(key, value);
    }
}

fn dump_property_container(base: usize, container: &ContainerI, source: BinType) -> Value {
    json!({
        "vtable": dump_hex(container.vtable as *const _ as usize - base),
        "value_type": container.value_type,
        "value_size": container.value_size,
        "fixed_size": container.get_fixed_size(),
        "storage": (source != BinType::Option).then(|| container.get_storage()),
    })
}

fn dump_property_map(base: usize, map: &MapI) -> Value {
    json!({
        "vtable": dump_hex(map.vtable as *const _ as usize - base),
        "key_type": map.key_type,
        "value_type": map.value_type,
        "storage": map.get_storage(),
    })
}

fn dump_property(base: usize, property: &Property) -> Value {
    json!({
        "other_class": property.other_class.map(|c| dump_hex(c.hash)),
        "offset": property.offset,
        "bitmask": property.bitmask,
        "value_type": property.value_type,
        "container": property.container.map(|c| dump_property_container(base, c, property.value_type)),
        "map": property.map.map(|m| dump_property_map(base, m)),
        "unkptr": dump_hex(0usize),
    })
}

fn dump_property_list(base: usize, properites: &[Property]) -> Value {
    let mut results = Map::new();
    for property in properites {
        let key = dump_hex(property.hash);
        let value = dump_property(base, property);
        results.insert(key, value);
    }
    results.into()
}

fn dump_class_functions(base: usize, class: &Class) -> Value {
    json!({
        "upcast_secondary": class.upcast_secondary_fn.map(|c| dump_hex(c as usize - base)),
        "constructor": class.constructor_fn.map(|c| dump_hex(c as usize - base)),
        "destructor": class.destructor_fn.map(|c| dump_hex(c as usize - base)),
        "inplace_constructor": class.inplace_constructor_fn.map(|c| dump_hex(c as usize - base)),
        "inplace_destructor": class.inplace_destructor_fn.map(|c| dump_hex(c as usize - base)),
        "register": class.register_fn.map(|c| dump_hex(c as usize - base)),
    })
}

fn dump_class_flags(class: &Class) -> Value {
    json!({
        // FIXME: gone with 12.10, what do we print here?
        // "property_base": class.is_property_base,
        "interface": class.constructor_fn.is_none(),
        "value": class.is_value,
        "secondary_base": class.is_secondary_base,
        "unk5": class.is_unk5,
    })
}

fn dump_class_secondary(class_offset_pairs: &[BaseOff]) -> Value {
    let mut results = Map::new();
    for &BaseOff(class, offset) in class_offset_pairs {
        let key = dump_hex(class.hash);
        let value = offset.into();
        results.insert(key, value);
    }
    results.into()
}

fn is_empty(class: &Class) -> bool {
    class.properties.size() == 0 && class.base_class.iter().all(|class| is_empty(class))
}

pub fn dump_class_defaults(class: &Class) -> Value {
    if class.constructor_fn.is_some() {
        let mut results = Map::new();
        if !is_empty(class) {
            let instance = class.create_instance();
            dump_instance_properties(class, instance, &mut results);
            class.destroy_instance(instance);
        }
        results.into()
    } else {
        Value::Null
    }
}

pub fn dump_class(base: usize, class: &Class) -> Value {
    json!({
        "base": class.base_class.map(|c| dump_hex(c.hash)),
        "secondary_bases": dump_class_secondary(class.secondary_bases.slice()),
        "secondary_children": dump_class_secondary(class.secondary_children.slice()),
        "size": class.class_size,
        "alignment": class.alignment,
        "is": dump_class_flags(class),
        "fn": dump_class_functions(base, class),
        "properties": dump_property_list(base, class.properties.slice()),
        "defaults": dump_class_defaults(class),
    })
}

pub fn dump_class_list(base: usize, classes: &[&Class]) -> Value {
    let mut results = Map::new();
    for &class in classes {
        let key = dump_hex(class.hash);
        let value = dump_class(base, class);
        results.insert(key, value);
    }
    results.into()
}

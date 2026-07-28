use serde::Serialize;
use std::sync::atomic::{AtomicUsize, Ordering};

#[repr(C)]
pub struct StdVector<T> {
    beg: *const T,
    end: *const T,
    cap: *const T,
}

impl<'a, T> StdVector<T> {
    pub fn size(&self) -> usize {
        unsafe { self.end.offset_from(self.beg) as usize }
    }

    pub fn slice(&self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts(self.beg, self.size()) }
    }
}

#[repr(C)]
pub struct RiotVector<T> {
    data: *const T,
    size: u32,
    capacity: u32,
}

impl<'a, T> RiotVector<T> {
    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn slice(&self) -> &'a [T] {
        (self.size != 0)
            .then(|| unsafe { std::slice::from_raw_parts(self.data, self.size()) })
            .unwrap_or_default()
    }

    /// Walk the elements using an explicit byte stride rather than
    /// `size_of::<T>()`, for records whose in-image size differs from the Rust
    /// struct describing them. See [`property_stride`].
    pub fn iter_strided(&self, stride: usize) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        let base = self.data as usize;
        let count = self.size();
        (0..count).map(move |i| unsafe { &*((base + i * stride) as *const T) })
    }
}

#[repr(C)]
pub struct AString {
    data: RiotVector<u8>,
}

impl AString {
    pub fn str(&self) -> &str {
        (self.data.size() != 0)
            .then(|| unsafe { std::str::from_utf8_unchecked(self.data.slice()) })
            .unwrap_or_default()
    }
}

#[repr(u8)]
#[non_exhaustive]
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BinType {
    None = 0,
    Bool = 1,
    I8 = 2,
    U8 = 3,
    I16 = 4,
    U16 = 5,
    I32 = 6,
    U32 = 7,
    I64 = 8,
    U64 = 9,
    F32 = 10,
    Vec2 = 11,
    Vec3 = 12,
    Vec4 = 13,
    Mtx44 = 14,
    Color = 15,
    String = 16,
    Hash = 17,
    File = 18,
    List = 0x80 | 0,
    List2 = 0x80 | 1,
    Pointer = 0x80 | 2,
    Embed = 0x80 | 3,
    Link = 0x80 | 4,
    Option = 0x80 | 5,
    Map = 0x80 | 6,
    Flag = 0x80 | 7,
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(C)]
pub enum ContainerStorage {
    UnknownVector,
    Option,
    Fixed,
    StdVector,
    RitoVector,
}

#[repr(C)]
pub struct ContainerIVtable {
    pub destructor: extern "C" fn(this: &ContainerI),
    pub deleter: extern "C" fn(this: &ContainerI),
    pub get_size: extern "C" fn(this: &ContainerI, instance: usize) -> usize,
    pub set_size: extern "C" fn(this: &ContainerI, instance: usize, size: usize),
    pub get_mut: extern "C" fn(this: &ContainerI, instance: usize, index: usize) -> usize,
    pub get_const: extern "C" fn(this: &ContainerI, instance: usize, index: usize) -> usize,
    pub clear: extern "C" fn(this: &ContainerI, instance: usize),
    pub push: extern "C" fn(this: &ContainerI, instance: usize, value: usize) -> usize,
    pub pop: extern "C" fn(this: &ContainerI, instance: usize),
    pub get_fixed_size: extern "C" fn(this: &ContainerI) -> i32,
}

#[repr(C)]
pub struct ContainerI {
    pub vtable: &'static ContainerIVtable,
    pub value_type: BinType,
    pub value_size: usize,
}

impl ContainerI {
    pub fn get_size(&self, instance: usize) -> usize {
        self.get_fixed_size()
            .unwrap_or_else(|| (self.vtable.get_size)(self, instance))
    }

    pub fn get_fixed_size(&self) -> Option<usize> {
        let result = (self.vtable.get_fixed_size)(self);
        (result >= 0).then(|| result as usize)
    }

    pub fn get_const(&self, instance: usize, index: usize) -> usize {
        (self.vtable.get_const)(self, instance, index)
    }

    pub fn get_storage(&self) -> ContainerStorage {
        if self.get_fixed_size().is_some() {
            ContainerStorage::Fixed
        } else {
            // FIXME: x86_64
            // let hax: [u32; 4] = [self.value_size, self.value_size * 2, 0, 0];
            // let result = self.get_size(&hax as *const _ as _);
            // if result == (self.value_size * 2) as usize {
            //     ContainerStorage::RitoVector
            // } else if result == 1 {
            //     ContainerStorage::StdVector
            // } else {
            //     ContainerStorage::UnknownVector
            // }
            ContainerStorage::UnknownVector
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(C)]
pub enum MapStorage {
    UnknownMap,
    StdMap,
    StdUnorderedMap,
    RitoVectorMap,
}

#[repr(C)]
pub struct MapConstIterIVtable {
    pub destructor: extern "C" fn(this: &mut MapConstIterI),
    pub deleter: extern "C" fn(this: &MapConstIterI),
    pub has_next: extern "C" fn(this: &MapConstIterI) -> bool,
    pub next: extern "C" fn(this: &mut MapConstIterI) -> usize,
    pub get_key: extern "C" fn(this: &MapConstIterI) -> usize,
    pub get_value: extern "C" fn(this: &MapConstIterI) -> usize,
}

#[repr(C)]
pub struct MapConstIterI {
    pub vtable: &'static MapConstIterIVtable,
}

#[repr(C)]
pub struct MapConstIter<'a> {
    pub ptr: &'a mut MapConstIterI,
}

impl<'a> Drop for MapConstIter<'a> {
    fn drop(&mut self) {
        (self.ptr.vtable.deleter)(self.ptr);
    }
}

impl<'a> Iterator for MapConstIter<'a> {
    type Item = (usize, usize);
    fn next(&mut self) -> Option<Self::Item> {
        if (self.ptr.vtable.has_next)(self.ptr) && (self.ptr.vtable.next)(self.ptr) != 0 {
            let key = (self.ptr.vtable.get_key)(self.ptr);
            let value = (self.ptr.vtable.get_value)(self.ptr);
            Some((key, value))
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct MapIVtable {
    pub destructor: extern "C" fn(this: &MapI),
    pub deleter: extern "C" fn(this: &MapI),
    pub get_size: extern "C" fn(this: &MapI, instance: usize) -> usize,
    pub reserve_size: extern "C" fn(this: &MapI, instance: usize, size: usize),
    pub finalize: extern "C" fn(this: &MapI, instance: usize),
    pub find: extern "C" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub clear: extern "C" fn(this: &MapI, instance: usize),
    pub create: extern "C" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub inplace_ctor: extern "C" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub inplace_dtor: extern "C" fn(this: &MapI, instance: usize, key: usize),
    pub erase: extern "C" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub iter_mut: extern "C" fn(this: &MapI, instance: usize) -> usize,
    pub iter_const: extern "C" fn(this: &MapI, instance: usize) -> &mut MapConstIterI,
}

#[repr(C)]
pub struct MapI {
    pub vtable: &'static MapIVtable,
    pub key_type: BinType,
    pub value_type: BinType,
}

impl MapI {
    pub fn get_size(&self, instance: usize) -> usize {
        (self.vtable.get_size)(self, instance)
    }

    pub fn iter_const(&self, instance: usize) -> MapConstIter {
        MapConstIter {
            ptr: (self.vtable.iter_const)(self, instance),
        }
    }

    pub fn get_storage(&self) -> MapStorage {
        // FIXME: x86_64
        // let hax: [usize; 8] = [0, 0x78000000, 1, 0, 0, 0, 0, 0];
        // let result = self.get_size(&hax as *const _ as _) as isize;
        // match result {
        //     0x78000000 => MapStorage::StdMap,
        //     0x7000.. => MapStorage::RitoVectorMap, // TODO: is this StdVector<Pair> or RitoVector<Pair> ???
        //     1 => MapStorage::StdUnorderedMap,
        //     _ => MapStorage::UnknownMap,
        // }
        MapStorage::UnknownMap
    }
}

#[repr(C)]
pub struct HashedI {
    pub vtable: &'static HashedIVtable,
}

#[repr(C)]
pub struct HashedIVtable {
    pub destructor: extern "C" fn(this: &HashedI),
    pub deleter: extern "C" fn(this: &HashedI),
    pub get_size: extern "C" fn(this: &HashedI, instance: usize) -> usize,
    pub from_string: extern "C" fn(this: &HashedI, instance: usize, str: *const AString) -> usize,
    pub from_hash: extern "C" fn(this: &HashedI, instance: usize, hash: u64) -> usize,
    pub to_hash: extern "C" fn(this: &HashedI, instance: usize) -> u64,
}

#[repr(C)]
pub struct Property {
    pub other_class: Option<&'static Class>,
    pub hash: u32,
    pub offset: u32,
    pub bitmask: u8,
    pub value_type: BinType,
    pub container: Option<&'static ContainerI>,
    pub map: Option<&'static MapI>,
    // added in 13.13
    pub hashed: Option<&'static HashedI>,
    // added in 16.1
    pub unkptr2: usize,
}

// ---------------------------------------------------------------------------
// Property stride
//
// 16.14 dropped one of `Property`'s trailing 8-byte fields, taking the record
// from 56 to 48 bytes. Everything up to and including `map` kept its offset -
// confirmed both by disassembly (the game's own finalize pass walks properties
// with a 48-byte stride) and by the 16.14 crash, whose faulting address decoded
// to exactly `property[1] + 8`, i.e. one stride's worth of overshoot.
//
// The `Property` struct below still describes the 56-byte form. The two fields
// that may or may not be present, `hashed` and `unkptr2`, are both unread by
// the dumper, so only the stride has to vary.
// ---------------------------------------------------------------------------

/// `Property` size up to and including 16.13.
const PROPERTY_STRIDE_LEGACY: usize = 56;

/// `Property` size from 16.14 onward.
const PROPERTY_STRIDE_16_14: usize = 48;

/// First version using the shorter record, as (major, minor).
const PROPERTY_SHRANK_AT: (u32, u32) = (16, 14);

static PROPERTY_STRIDE: AtomicUsize = AtomicUsize::new(PROPERTY_STRIDE_LEGACY);

fn parse_major_minor(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

/// Pick the `Property` stride for a detected version string, which may be
/// either `"16.14"` or `"16.14.7949266"`.
///
/// An unrecognised version falls back to the legacy stride: that is what every
/// dump in the repository was produced with, so it is the choice that keeps
/// historical behaviour rather than silently switching layout.
pub fn property_stride_for(version: Option<&str>) -> usize {
    match version.and_then(parse_major_minor) {
        Some(v) if v >= PROPERTY_SHRANK_AT => PROPERTY_STRIDE_16_14,
        Some(_) => PROPERTY_STRIDE_LEGACY,
        None => {
            eprintln!(
                "WARNING: could not parse version {:?}; assuming the pre-16.14 \
                 {}-byte property layout. If this is a newer build the dump will \
                 be wrong or the dumper will crash.",
                version, PROPERTY_STRIDE_LEGACY
            );
            PROPERTY_STRIDE_LEGACY
        }
    }
}

pub fn set_property_stride(stride: usize) {
    PROPERTY_STRIDE.store(stride, Ordering::Relaxed);
}

pub fn property_stride() -> usize {
    PROPERTY_STRIDE.load(Ordering::Relaxed)
}

#[cfg(test)]
mod stride_tests {
    use super::*;

    #[test]
    fn legacy_versions_use_the_56_byte_record() {
        // find_version yields "16.13"; find_version2 yields "16.13.7915903".
        //
        // 16.12 is deliberately here: it crashes too, but with a null-deref
        // signature rather than the stride overshoot, so it is a separate bug
        // and must keep the layout 16.13 demonstrably dumps correctly.
        for v in [
            "16.12",
            "16.12.7884269",
            "16.13",
            "16.13.7915903",
            "16.1",
            "15.24",
            "13.14.5227601",
        ] {
            assert_eq!(property_stride_for(Some(v)), PROPERTY_STRIDE_LEGACY, "{v}");
        }
    }

    #[test]
    fn versions_from_16_14_use_the_48_byte_record() {
        for v in ["16.14", "16.14.7949266", "16.15", "16.20", "17.1"] {
            assert_eq!(property_stride_for(Some(v)), PROPERTY_STRIDE_16_14, "{v}");
        }
    }

    #[test]
    fn minor_versions_compare_numerically_not_lexically() {
        // "16.9" > "16.14" as strings; the ordering that matters is numeric.
        assert_eq!(property_stride_for(Some("16.9")), PROPERTY_STRIDE_LEGACY);
        assert_eq!(property_stride_for(Some("16.2")), PROPERTY_STRIDE_LEGACY);
        assert_eq!(property_stride_for(Some("16.100")), PROPERTY_STRIDE_16_14);
    }

    #[test]
    fn unparseable_versions_fall_back_to_legacy() {
        for v in [None, Some("unknown"), Some(""), Some("16"), Some("16.x")] {
            assert_eq!(property_stride_for(v), PROPERTY_STRIDE_LEGACY, "{v:?}");
        }
    }

    #[test]
    fn the_legacy_stride_matches_the_rust_struct() {
        // `Property` describes the 56-byte form; if a field is ever added to it
        // without revisiting the constants here, the strides silently disagree.
        assert_eq!(
            std::mem::size_of::<Property>(),
            PROPERTY_STRIDE_LEGACY,
            "Property struct changed size - update the stride constants"
        );
        assert_eq!(PROPERTY_STRIDE_LEGACY - PROPERTY_STRIDE_16_14, 8);
    }
}

#[repr(C)]
pub struct BaseOff(pub &'static Class, pub u32);

#[repr(C)]
pub struct Class {
    pub upcast_secondary_fn: Option<extern "C" fn(instance: usize) -> usize>,
    pub hash: u32,
    pub constructor_fn: Option<extern "C" fn() -> usize>,
    pub destructor_fn: Option<extern "C" fn(instance: usize)>,
    pub inplace_constructor_fn: Option<extern "C" fn(instance: usize)>,
    pub inplace_destructor_fn: Option<extern "C" fn(instance: usize)>,
    pub register_fn: Option<extern "C" fn(instance: usize)>,
    pub base_class: Option<&'static Class>,
    pub class_size: usize,
    pub alignment: usize,
    pub is_value: bool,
    pub is_secondary_base: bool,
    pub is_unk5: bool,
    pub properties: RiotVector<Property>,
    pub secondary_bases: RiotVector<BaseOff>,
    pub secondary_children: RiotVector<BaseOff>,
}

impl Class {
    /// The class's properties, walked at the stride this image actually uses.
    ///
    /// Always prefer this over `self.properties.slice()`, which assumes the
    /// record is `size_of::<Property>()` bytes and is wrong from 16.14 on.
    pub fn iter_properties(&self) -> impl Iterator<Item = &'static Property> {
        self.properties.iter_strided(property_stride())
    }

    pub fn create_instance(&self) -> usize {
        let ctor = self
            .constructor_fn
            .expect("Can not create instance (it might be interface)!");
        (ctor)()
    }

    pub fn destroy_instance(&self, instance: usize) {
        let dtor = self
            .destructor_fn
            .expect("Can not destroy instance (it might be interface)!");
        (dtor)(instance)
    }
}

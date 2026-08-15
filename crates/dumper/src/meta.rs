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

    /// Backing pointer and element count, for raw memory dumps.
    pub fn raw_parts(&self) -> (usize, usize) {
        (self.data as usize, self.size())
    }

    /// Borrows a slice as a vector image code can read.
    ///
    /// Argument use only: the image must not keep or outlive the borrow.
    pub fn borrowed(slice: &'a [T]) -> Self {
        RiotVector {
            data: slice.as_ptr(),
            size: slice.len() as u32,
            capacity: slice.len() as u32,
        }
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

/// Hash algorithms a helper has been observed to use.
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum HashAlgorithm {
    Fnv1a32,
    Xxh64,
    Xxh3,
    Unknown,
}

/// The result of probing a helper: which hash, and whether it lowercases first.
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct HashFunction {
    pub algorithm: HashAlgorithm,
    pub lowercased: bool,
}

/// Probe strings for identifying a helper's hash function.
///
/// Mixed case so lowercasing is observable; both probes must agree or the
/// answer is [`HashAlgorithm::Unknown`].
const HASH_PROBES: [&str; 2] = ["Characters/Ahri/Skins/Skin01", "UX/TFT/Foo_Bar.TEX"];

/// Known value round-tripped through `from_hash` / `to_hash` before trusting
/// the vtable shape.
const HASH_SELFTEST: u64 = 0x1234_5678;

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    for &b in bytes {
        h = (h ^ b as u32).wrapping_mul(0x0100_0193);
    }
    h
}

impl HashedI {
    /// Returns the bytes the helper stores, from its `get_size`.
    pub fn storage_width(&self) -> usize {
        (self.vtable.get_size)(self, 0)
    }

    /// Returns the helper's hash of `text`, or `None` if the vtable fails the
    /// self-test.
    ///
    /// Calls into the loaded image. The result is read back through `to_hash`,
    /// so a truncating helper reports its truncated value.
    pub fn hash_of(&self, text: &str) -> Option<u64> {
        let mut scratch: u64 = 0;
        let slot = &mut scratch as *mut u64 as usize;

        // Bail if the vtable is not the layout `HashedIVtable` describes
        // (e.g. a build reordered it); the calls below would hit the wrong
        // functions.
        (self.vtable.from_hash)(self, slot, HASH_SELFTEST);
        if (self.vtable.to_hash)(self, slot) != HASH_SELFTEST {
            return None;
        }

        // The slot still holds the self-test value, so a `from_string` that
        // does not write matches no candidate instead of a plausible hash.
        let bytes = text.as_bytes();
        let string = AString {
            data: RiotVector::borrowed(bytes),
        };
        (self.vtable.from_string)(self, slot, &string);
        Some((self.vtable.to_hash)(self, slot))
    }

    /// Identifies the helper's hash function by probing it.
    pub fn hash_function(&self) -> HashFunction {
        let unknown = HashFunction {
            algorithm: HashAlgorithm::Unknown,
            lowercased: false,
        };
        let width = self.storage_width();
        let mask = match width {
            4 => u32::MAX as u64,
            8 => u64::MAX,
            _ => return unknown,
        };

        let mut agreed: Option<(HashAlgorithm, bool)> = None;
        for probe in HASH_PROBES {
            let Some(got) = self.hash_of(probe) else {
                return unknown;
            };
            let mut found = None;
            for lowercased in [false, true] {
                let text = if lowercased {
                    probe.to_ascii_lowercase()
                } else {
                    probe.to_string()
                };
                let bytes = text.as_bytes();
                for algorithm in [
                    HashAlgorithm::Fnv1a32,
                    HashAlgorithm::Xxh64,
                    HashAlgorithm::Xxh3,
                ] {
                    let want = match algorithm {
                        HashAlgorithm::Fnv1a32 => fnv1a32(bytes) as u64,
                        HashAlgorithm::Xxh64 => xxhash_rust::xxh64::xxh64(bytes, 0),
                        HashAlgorithm::Xxh3 => xxhash_rust::xxh3::xxh3_64(bytes),
                        HashAlgorithm::Unknown => continue,
                    };
                    if want & mask == got & mask {
                        found = Some((algorithm, lowercased));
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
            match (found, agreed) {
                (None, _) => return unknown,
                (Some(f), None) => agreed = Some(f),
                (Some(f), Some(a)) if f == a => {}
                _ => return unknown,
            }
        }

        match agreed {
            Some((algorithm, lowercased)) => HashFunction {
                algorithm,
                lowercased,
            },
            None => unknown,
        }
    }
}

// ---------------------------------------------------------------------------
// Layout dispatch
//
// Record layouts change between builds. Each is declared as its own
// `#[repr(C)]` struct so the compiler derives the offsets rather than them
// being written out by hand.
//
// `layout_selector!` defines the set of layouts plus the cell holding the one in
// use; `layout_ref!` defines a Copy handle that dispatches field reads over
// them. Dispatch stays dynamic so no generics leak into the dump pipeline.
//
// Adding a future layout is: declare the struct, add one variant to the
// selector, add one line to the `layout_ref!` variant list.
// ---------------------------------------------------------------------------

/// Define a layout selector enum and the cell holding the active choice.
macro_rules! layout_selector {
    (
        $(#[$meta:meta])*
        $Layout:ident in $CELL:ident, get = $get:ident, set = $set:ident {
            $( $Variant:ident ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        #[repr(usize)]
        pub enum $Layout { $( $Variant ),+ }

        impl $Layout {
            /// Every layout, in discriminant order.
            pub const ALL: &'static [$Layout] = &[ $( $Layout::$Variant ),+ ];
        }

        static $CELL: AtomicUsize = AtomicUsize::new(0);

        /// The layout in use. Set once before the walk, never changed during it.
        pub fn $get() -> $Layout {
            $Layout::ALL[$CELL.load(Ordering::Relaxed)]
        }

        pub fn $set(layout: $Layout) {
            $CELL.store(layout as usize, Ordering::Relaxed);
        }
    };
}

/// Define a Copy handle that reads a record through whichever layout is active.
///
/// Generates the handle, its constructor and its record size - everything that
/// varies only over the *variant* list. Field forwarding is deliberately not
/// here: it would need a repetition over variants nested inside a repetition
/// over fields, which `macro_rules!` cannot express (both sit at the same
/// depth). Use `forward_fields!` alongside this for that.
macro_rules! layout_ref {
    (
        $(#[$meta:meta])*
        $Ref:ident over $Layout:ident, active = $active:ident {
            $( $Variant:ident => $Struct:ident ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy)]
        pub enum $Ref { $( $Variant(&'static $Struct) ),+ }

        impl $Ref {
            /// Interpret a raw record pointer under the active layout.
            ///
            /// `None` for null, so malformed data reports itself rather than
            /// faulting on the first field read.
            pub fn from_ptr(ptr: *const ()) -> Option<Self> {
                if ptr.is_null() {
                    return None;
                }
                Some(match $active() {
                    $( $Layout::$Variant => $Ref::$Variant(unsafe { &*(ptr as *const $Struct) }) ),+
                })
            }

            pub fn as_ptr(self) -> *const () {
                match self { $( $Ref::$Variant(r) => r as *const _ as *const () ),+ }
            }

            /// Size of the record, derived from the struct rather than declared.
            pub fn record_size(layout: $Layout) -> usize {
                match layout {
                    $( $Layout::$Variant => core::mem::size_of::<$Struct>() ),+
                }
            }
        }
    };
}

/// Forward field reads across a handle's variants.
///
/// The variant arms are written out once per handle; the field list is what
/// repeats. Adding a layout means adding one arm here, whatever the field count.
macro_rules! forward_fields {
    // Internal: one field, expanded across the variant list.
    //
    // The variant list arrives as a single token tree and is destructured here.
    // That indirection is what makes this legal: a field name bound by the outer
    // repetition cannot be used inside a repetition over variants, since both
    // sit at the same depth, but a field bound singly in this rule can.
    (@one $Ref:ident, { $( $Variant:ident ),+ $(,)? }, $f:ident, $t:ty, by_value) => {
        pub fn $f(self) -> $t {
            match self { $( $Ref::$Variant(r) => r.$f ),+ }
        }
    };
    (@one $Ref:ident, { $( $Variant:ident ),+ $(,)? }, $f:ident, $t:ty, by_ref) => {
        pub fn $f(self) -> &'static $t {
            match self { $( $Ref::$Variant(r) => &r.$f ),+ }
        }
    };
    (
        $Ref:ident $variants:tt
        copy { $( $cf:ident : $cty:ty ),* $(,)? }
        by_ref { $( $rf:ident : $rty:ty ),* $(,)? }
    ) => {
        impl $Ref {
            $( forward_fields!(@one $Ref, $variants, $cf, $cty, by_value); )*
            $( forward_fields!(@one $Ref, $variants, $rf, $rty, by_ref); )*
        }
    };
}

fn parse_major_minor(version: &str) -> Option<(u32, u32)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

// ---------------------------------------------------------------------------
// Property layouts
//
// 16.14 shrank the record from 56 to 48 bytes: `container` and `map` share one
// slot at +0x20, discriminated by `value_type`.
// ---------------------------------------------------------------------------

layout_selector! {
    /// Which property record layout the image uses.
    PropertyLayout in PROPERTY_LAYOUT, get = property_layout, set = set_property_layout {
        Canonical,
        V16_14,
    }
}

/// First version using the 48-byte record.
const PROPERTY_SHRANK_AT: (u32, u32) = (16, 14);

/// Pick the property layout for a detected version string, which may be either
/// `"16.14"` or `"16.14.7949266"`.
///
/// An unrecognised version falls back to the canonical layout: that is what
/// every dump in the repository was produced with, so it keeps historical
/// behaviour rather than silently switching.
pub fn property_layout_for(version: Option<&str>) -> PropertyLayout {
    match version.and_then(parse_major_minor) {
        Some(v) if v >= PROPERTY_SHRANK_AT => PropertyLayout::V16_14,
        Some(_) => PropertyLayout::Canonical,
        None => {
            eprintln!(
                "WARNING: could not parse version {version:?}; assuming the \
                 pre-16.14 property layout. If this is a newer build the dump \
                 will be wrong or the dumper will crash."
            );
            PropertyLayout::Canonical
        }
    }
}

/// The property record up to and including 16.13: 56 bytes, `container` and
/// `map` in separate slots.
#[repr(C)]
pub struct PropertyCanonical {
    pub other_class_ptr: *const (),
    pub hash: u32,
    pub offset: u32,
    pub bitmask: u8,
    pub value_type: BinType,
    pub container: Option<&'static ContainerI>,
    pub map: Option<&'static MapI>,
    /// added in 13.13. Never read.
    pub hashed: Option<&'static HashedI>,
    /// added in 16.1. Never read.
    pub unkptr2: usize,
}

/// The property record from 16.14: 48 bytes. `container` and `map` share one
/// slot, and the slot that used to hold `container` is unused.
#[repr(C)]
pub struct Property1614 {
    pub other_class_ptr: *const (),
    pub hash: u32,
    pub offset: u32,
    pub bitmask: u8,
    pub value_type: BinType,
    /// Unused; always null.
    pub unk_18: usize,
    /// `container` or `map`, according to `value_type`.
    pub union_slot: usize,
    /// Owning pointer, move-transferred on reallocation. Never read.
    pub owned: usize,
}

layout_ref! {
    /// A property record, resolved to whichever layout this image uses.
    PropertyRef over PropertyLayout, active = property_layout {
        Canonical => PropertyCanonical,
        V16_14 => Property1614,
    }
}

forward_fields! {
    PropertyRef { Canonical, V16_14 }
    copy {
        hash: u32,
        offset: u32,
        bitmask: u8,
        value_type: BinType,
        other_class_ptr: *const (),
    }
    by_ref {}
}

impl PropertyRef {
    /// The class this property refers to, for `Embed`, `Pointer` and `Link`.
    pub fn other_class(self) -> Option<ClassRef> {
        ClassRef::from_ptr(self.other_class_ptr())
    }

    /// Raw value of the 16.14 union slot, for diagnostics. Zero pre-16.14.
    pub fn union_raw(self) -> usize {
        match self {
            PropertyRef::Canonical(_) => 0,
            PropertyRef::V16_14(p) => p.union_slot,
        }
    }

    /// The container backing a `List`, `List2` or `Option` property.
    ///
    /// Hand-written rather than forwarded: from 16.14 this pointer shares a slot
    /// with `map`, so only `value_type` says which one it is. Reading the slot
    /// for any other type would hand back an unrelated pointer.
    pub fn container(self) -> Option<&'static ContainerI> {
        match self {
            PropertyRef::Canonical(p) => p.container,
            PropertyRef::V16_14(p) => match p.value_type {
                BinType::List | BinType::List2 | BinType::Option => {
                    (p.union_slot != 0).then(|| unsafe { &*(p.union_slot as *const ContainerI) })
                }
                _ => None,
            },
        }
    }

    /// The map backing a `Map` property.
    pub fn map(self) -> Option<&'static MapI> {
        match self {
            PropertyRef::Canonical(p) => p.map,
            PropertyRef::V16_14(p) => match p.value_type {
                BinType::Map => {
                    (p.union_slot != 0).then(|| unsafe { &*(p.union_slot as *const MapI) })
                }
                _ => None,
            },
        }
    }

    /// The hash helper backing a `Hash` property.
    ///
    /// Type-gated on both layouts: from 16.14 the helper shares the union slot
    /// with `container` and `map`. Null on builds before 16.1, which never
    /// install one.
    pub fn hashed(self) -> Option<&'static HashedI> {
        match self {
            PropertyRef::Canonical(p) => match p.value_type {
                BinType::Hash => p.hashed,
                _ => None,
            },
            PropertyRef::V16_14(p) => match p.value_type {
                BinType::Hash => {
                    (p.union_slot != 0).then(|| unsafe { &*(p.union_slot as *const HashedI) })
                }
                _ => None,
            },
        }
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    /// Serialises the tests that mutate the global layout selection. Without it
    /// they race each other under the default parallel test runner.
    static LAYOUT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn lock_layout() -> std::sync::MutexGuard<'static, ()> {
        LAYOUT_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Byte offset of a field within its struct.
    macro_rules! offset_of_field {
        ($ty:ty, $field:ident) => {{
            let probe = std::mem::MaybeUninit::<$ty>::uninit();
            let base = probe.as_ptr() as usize;
            let field = unsafe { std::ptr::addr_of!((*probe.as_ptr()).$field) } as usize;
            field - base
        }};
    }

    // -- version -> layout selection -----------------------------------------

    #[test]
    fn property_layout_is_selected_by_version() {
        // 16.12 is deliberately canonical here: it crashes for an unrelated
        // reason (its Class layout), and must keep the property layout that
        // 16.13 either side of it demonstrably dumps correctly.
        for v in [
            "16.12",
            "16.12.7884269",
            "16.13",
            "16.13.7915903",
            "16.1",
            "15.24",
            "13.14.5227601",
        ] {
            assert_eq!(
                property_layout_for(Some(v)),
                PropertyLayout::Canonical,
                "{v}"
            );
        }
        for v in ["16.14", "16.14.7949266", "16.15", "16.20", "17.1"] {
            assert_eq!(property_layout_for(Some(v)), PropertyLayout::V16_14, "{v}");
        }
    }

    #[test]
    fn minor_versions_compare_numerically_not_lexically() {
        // "16.9" > "16.14" as strings; the ordering that matters is numeric.
        assert_eq!(property_layout_for(Some("16.9")), PropertyLayout::Canonical);
        assert_eq!(property_layout_for(Some("16.2")), PropertyLayout::Canonical);
        assert_eq!(
            property_layout_for(Some("16.100")),
            PropertyLayout::V16_14
        );
    }

    #[test]
    fn unparseable_versions_fall_back_to_canonical() {
        for v in [None, Some("unknown"), Some(""), Some("16"), Some("16.x")] {
            assert_eq!(property_layout_for(v), PropertyLayout::Canonical, "{v:?}");
        }
        assert_eq!(class_layout_for(None), ClassLayout::Canonical);
    }

    #[test]
    fn class_layout_is_selected_by_version() {
        assert_eq!(class_layout_for(Some("16.12")), ClassLayout::V16_12);
        assert_eq!(class_layout_for(Some("16.12.7884269")), ClassLayout::V16_12);
        for v in ["16.11", "16.13", "16.14", "16.2", "15.24"] {
            assert_eq!(class_layout_for(Some(v)), ClassLayout::Canonical, "{v}");
        }
    }

    // -- record layouts ------------------------------------------------------

    #[test]
    fn property_layouts_match_the_image() {
        assert_eq!(offset_of_field!(PropertyCanonical, hash), 0x08);
        assert_eq!(offset_of_field!(PropertyCanonical, offset), 0x0C);
        assert_eq!(offset_of_field!(PropertyCanonical, bitmask), 0x10);
        assert_eq!(offset_of_field!(PropertyCanonical, value_type), 0x11);
        assert_eq!(offset_of_field!(PropertyCanonical, container), 0x18);
        assert_eq!(offset_of_field!(PropertyCanonical, map), 0x20);
        assert_eq!(std::mem::size_of::<PropertyCanonical>(), 56);

        assert_eq!(offset_of_field!(Property1614, hash), 0x08);
        assert_eq!(offset_of_field!(Property1614, offset), 0x0C);
        assert_eq!(offset_of_field!(Property1614, bitmask), 0x10);
        assert_eq!(offset_of_field!(Property1614, value_type), 0x11);
        assert_eq!(offset_of_field!(Property1614, unk_18), 0x18);
        assert_eq!(offset_of_field!(Property1614, union_slot), 0x20);
        assert_eq!(std::mem::size_of::<Property1614>(), 48);

        // The record size the walk uses comes from the struct, not a constant.
        assert_eq!(PropertyRef::record_size(PropertyLayout::Canonical), 56);
        assert_eq!(PropertyRef::record_size(PropertyLayout::V16_14), 48);
    }

    #[test]
    fn canonical_class_layout_matches_the_image() {
        assert_eq!(offset_of_field!(ClassCanonical, hash), 0x08);
        assert_eq!(offset_of_field!(ClassCanonical, constructor_fn), 0x10);
        assert_eq!(offset_of_field!(ClassCanonical, destructor_fn), 0x18);
        assert_eq!(offset_of_field!(ClassCanonical, base_class_ptr), 0x38);
        assert_eq!(offset_of_field!(ClassCanonical, class_size), 0x40);
        assert_eq!(offset_of_field!(ClassCanonical, alignment), 0x48);
        assert_eq!(offset_of_field!(ClassCanonical, is_value), 0x50);
        assert_eq!(offset_of_field!(ClassCanonical, properties), 0x58);
        assert_eq!(offset_of_field!(ClassCanonical, secondary_bases), 0x68);
        assert_eq!(offset_of_field!(ClassCanonical, secondary_children), 0x78);
        assert_eq!(std::mem::size_of::<ClassCanonical>(), 0x88);
    }

    #[test]
    fn the_16_12_layout_is_the_canonical_one_shifted_from_0x18() {
        // The canonical layout with one extra slot at +0x18.
        assert_eq!(offset_of_field!(Class1612, hash), 0x08);
        assert_eq!(offset_of_field!(Class1612, constructor_fn), 0x10);
        assert_eq!(offset_of_field!(Class1612, unk_18), 0x18);
        assert_eq!(offset_of_field!(Class1612, destructor_fn), 0x20);
        assert_eq!(offset_of_field!(Class1612, class_size), 0x48);
        assert_eq!(offset_of_field!(Class1612, alignment), 0x50);
        assert_eq!(offset_of_field!(Class1612, properties), 0x60);
        assert_eq!(std::mem::size_of::<Class1612>(), 0x90);

        // Nothing before `destructor_fn` moves...
        assert_eq!(
            offset_of_field!(Class1612, hash),
            offset_of_field!(ClassCanonical, hash)
        );
        assert_eq!(
            offset_of_field!(Class1612, constructor_fn),
            offset_of_field!(ClassCanonical, constructor_fn)
        );
        // ...and everything from it on is displaced by exactly 8.
        for (a, b) in [
            (
                offset_of_field!(Class1612, destructor_fn),
                offset_of_field!(ClassCanonical, destructor_fn),
            ),
            (
                offset_of_field!(Class1612, properties),
                offset_of_field!(ClassCanonical, properties),
            ),
            (
                offset_of_field!(Class1612, secondary_children),
                offset_of_field!(ClassCanonical, secondary_children),
            ),
        ] {
            assert_eq!(a - b, 8);
        }
    }

    // -- union semantics -----------------------------------------------------

    /// A sentinel handed back as a pointer. Never dereferenced - these tests
    /// only check which slot the accessors select.
    const SLOT: usize = 0x1000;

    fn probe_16_14(value_type: BinType, union_slot: usize) -> Property1614 {
        Property1614 {
            other_class_ptr: std::ptr::null(),
            hash: 0xdead_beef,
            offset: 0,
            bitmask: 0,
            value_type,
            unk_18: 0,
            union_slot,
            owned: 0,
        }
    }

    #[test]
    fn union_slot_is_tagged_by_type_from_16_14() {
        let _guard = lock_layout();
        set_property_layout(PropertyLayout::V16_14);

        // Container types read the union...
        for t in [BinType::List, BinType::List2, BinType::Option] {
            let p = probe_16_14(t, SLOT);
            let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
            assert!(r.container().is_some(), "{t:?} should resolve a container");
            assert!(r.map().is_none(), "{t:?} must not resolve a map");
        }

        // ...Map reads the same slot as a map...
        let p = probe_16_14(BinType::Map, SLOT);
        let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
        assert!(r.map().is_some());
        assert!(r.container().is_none());

        // ...and anything else claims neither: the slot holds an unrelated
        // pointer for those types.
        for t in [BinType::Embed, BinType::Pointer, BinType::Link, BinType::U32] {
            let p = probe_16_14(t, SLOT);
            let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
            assert!(r.container().is_none(), "{t:?} must not resolve a container");
            assert!(r.map().is_none(), "{t:?} must not resolve a map");
        }

        // A NULL slot stays None even for a container type.
        let p = probe_16_14(BinType::List, 0);
        let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
        assert!(r.container().is_none());

        set_property_layout(PropertyLayout::Canonical);
    }

    /// Misreading the 16.14 union slot would report a container vtable as a
    /// hash helper and then call it.
    #[test]
    fn hash_helper_reads_the_union_only_for_hash() {
        let _guard = lock_layout();
        set_property_layout(PropertyLayout::V16_14);

        let p = probe_16_14(BinType::Hash, SLOT);
        let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
        assert!(r.hashed().is_some());
        assert!(r.container().is_none());
        assert!(r.map().is_none());

        for t in [
            BinType::List,
            BinType::Map,
            BinType::Option,
            BinType::File,
            BinType::String,
            BinType::Link,
        ] {
            let p = probe_16_14(t, SLOT);
            let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
            assert!(r.hashed().is_none(), "{t:?} must not resolve a hash helper");
        }

        // Pre-16.1 builds leave the slot empty on a Hash property.
        let p = probe_16_14(BinType::Hash, 0);
        let r = PropertyRef::V16_14(unsafe { &*(&p as *const _) });
        assert!(r.hashed().is_none());

        set_property_layout(PropertyLayout::Canonical);
    }

    /// A wrong FNV here would classify every helper as `Unknown`.
    #[test]
    fn fnv1a32_matches_the_known_bin_hash() {
        assert_eq!(fnv1a32(b"characterrecord"), 0x23ea_1915);
        assert_eq!(fnv1a32(b""), 0x811c_9dc5);
    }

    #[test]
    fn canonical_keeps_container_and_map_in_separate_slots() {
        let _guard = lock_layout();
        set_property_layout(PropertyLayout::Canonical);

        let p = PropertyCanonical {
            other_class_ptr: std::ptr::null(),
            hash: 0xdead_beef,
            offset: 0,
            bitmask: 0,
            value_type: BinType::List,
            container: None,
            map: None,
            hashed: None,
            unkptr2: 0,
        };
        let r = PropertyRef::Canonical(unsafe { &*(&p as *const _) });
        // Both come from their own fields, and neither is inferred from the type.
        assert!(r.container().is_none());
        assert!(r.map().is_none());
        assert_eq!(r.union_raw(), 0);
    }

    // -- selector storage ----------------------------------------------------

    #[test]
    fn selector_round_trips_every_layout() {
        let _guard = lock_layout();
        // Guards against storing the choice as a boolean, which silently caps
        // the design at two layouts.
        for &layout in ClassLayout::ALL {
            set_class_layout(layout);
            assert_eq!(class_layout(), layout);
        }
        set_class_layout(ClassLayout::Canonical);

        for &layout in PropertyLayout::ALL {
            set_property_layout(layout);
            assert_eq!(property_layout(), layout);
        }
        set_property_layout(PropertyLayout::Canonical);
    }
}

/// A `(class, offset)` pair. The class pointer is raw: it is only valid under
/// whichever layout the image uses, and a null is possible in malformed data.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct BaseOff {
    pub class: *const (),
    pub offset: u32,
}

// ---------------------------------------------------------------------------
// Class layouts
//
// 16.12 carries an extra 8-byte field at +0x18, pushing everything from
// `destructor_fn` onward down by 8. It is an outlier: 16.11 and 16.13 either
// side of it use the canonical layout, as does 16.14, so the alternate layout
// is keyed to exactly 16.12.
//
// Each layout is a plain `#[repr(C)]` struct so the compiler derives the field
// offsets. `ClassRef` then dispatches over them, which keeps the rest of the
// dumper free of both generics and hand-written offset arithmetic.
// ---------------------------------------------------------------------------

/// The metaclass layout used by every version except 16.12.
#[repr(C)]
pub struct ClassCanonical {
    pub upcast_secondary_fn: Option<extern "C" fn(instance: usize) -> usize>,
    pub hash: u32,
    pub constructor_fn: Option<extern "C" fn() -> usize>,
    pub destructor_fn: Option<extern "C" fn(instance: usize)>,
    pub inplace_constructor_fn: Option<extern "C" fn(instance: usize)>,
    pub inplace_destructor_fn: Option<extern "C" fn(instance: usize)>,
    pub register_fn: Option<extern "C" fn(instance: usize)>,
    pub base_class_ptr: *const (),
    pub class_size: usize,
    pub alignment: usize,
    pub is_value: bool,
    pub is_secondary_base: bool,
    /// Not a class flag: an "already processed" marker that the metaclass
    /// registry sets when it finalises the entry.
    pub is_finalized: bool,
    pub properties: RiotVector<u8>,
    pub secondary_bases: RiotVector<BaseOff>,
    pub secondary_children: RiotVector<BaseOff>,
}

/// The 16.12 metaclass layout: one extra pointer-sized field at `+0x18`.
#[repr(C)]
pub struct Class1612 {
    pub upcast_secondary_fn: Option<extern "C" fn(instance: usize) -> usize>,
    pub hash: u32,
    pub constructor_fn: Option<extern "C" fn() -> usize>,
    /// Unused; always null. A dead slot in the lifecycle-function block, and
    /// the whole difference between this layout and the canonical one. Gone
    /// again in 16.14, which is why that record is 8 bytes shorter.
    pub unk_18: usize,
    pub destructor_fn: Option<extern "C" fn(instance: usize)>,
    pub inplace_constructor_fn: Option<extern "C" fn(instance: usize)>,
    pub inplace_destructor_fn: Option<extern "C" fn(instance: usize)>,
    pub register_fn: Option<extern "C" fn(instance: usize)>,
    pub base_class_ptr: *const (),
    pub class_size: usize,
    pub alignment: usize,
    pub is_value: bool,
    pub is_secondary_base: bool,
    /// Not a class flag: an "already processed" marker that the metaclass
    /// registry sets when it finalises the entry.
    pub is_finalized: bool,
    pub properties: RiotVector<u8>,
    pub secondary_bases: RiotVector<BaseOff>,
    pub secondary_children: RiotVector<BaseOff>,
}

layout_selector! {
    /// Which metaclass layout the image being dumped uses.
    ClassLayout in CLASS_LAYOUT, get = class_layout, set = set_class_layout {
        Canonical,
        V16_12,
    }
}

/// The one version carrying the extra field.
const CLASS_LAYOUT_ALT_VERSION: (u32, u32) = (16, 12);

pub fn class_layout_for(version: Option<&str>) -> ClassLayout {
    match version.and_then(parse_major_minor) {
        Some(v) if v == CLASS_LAYOUT_ALT_VERSION => ClassLayout::V16_12,
        _ => ClassLayout::Canonical,
    }
}

layout_ref! {
    /// A metaclass, resolved to whichever layout this image uses.
    ClassRef over ClassLayout, active = class_layout {
        Canonical => ClassCanonical,
        V16_12 => Class1612,
    }
}

forward_fields! {
    ClassRef { Canonical, V16_12 }
    copy {
        hash: u32,
        upcast_secondary_fn: Option<extern "C" fn(usize) -> usize>,
        constructor_fn: Option<extern "C" fn() -> usize>,
        destructor_fn: Option<extern "C" fn(usize)>,
        inplace_constructor_fn: Option<extern "C" fn(usize)>,
        inplace_destructor_fn: Option<extern "C" fn(usize)>,
        register_fn: Option<extern "C" fn(usize)>,
        class_size: usize,
        alignment: usize,
        is_value: bool,
        is_secondary_base: bool,
        is_finalized: bool,
        base_class_ptr: *const (),
    }
    by_ref {
        properties: RiotVector<u8>,
        secondary_bases: RiotVector<BaseOff>,
        secondary_children: RiotVector<BaseOff>,
    }
}

impl ClassRef {
    /// The base class, resolved under the same layout.
    pub fn base_class(self) -> Option<ClassRef> {
        ClassRef::from_ptr(self.base_class_ptr())
    }

    /// The class's properties, walked at the record size this image uses.
    ///
    /// A non-zero count with a NULL backing pointer is reported rather than
    /// walked: the records would start at address 0 and the first field read
    /// would fault, which is a crash that says nothing about its cause.
    pub fn iter_properties(self) -> impl Iterator<Item = PropertyRef> {
        let (data, count) = self.properties().raw_parts();
        let count = if data == 0 && count != 0 {
            let message = format!(
                "class {:#x} declares {} properties but the array pointer is NULL",
                self.hash(),
                count
            );
            if crate::diag::tolerate_anomalies() {
                crate::diag::record_anomaly(&message);
                0
            } else {
                panic!("{message}");
            }
        } else {
            count
        };
        let stride = PropertyRef::record_size(property_layout());
        (0..count).filter_map(move |i| PropertyRef::from_ptr((data + i * stride) as *const ()))
    }

    pub fn create_instance(self) -> usize {
        let ctor = self
            .constructor_fn()
            .expect("Can not create instance (it might be interface)!");
        (ctor)()
    }

    pub fn destroy_instance(self, instance: usize) {
        let dtor = self
            .destructor_fn()
            .expect("Can not destroy instance (it might be interface)!");
        (dtor)(instance)
    }
}

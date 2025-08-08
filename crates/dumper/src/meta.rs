use serde::Serialize;

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
    pub destructor: extern "thiscall" fn(this: &ContainerI),
    pub deleter: extern "thiscall" fn(this: &ContainerI),
    pub get_size: extern "thiscall" fn(this: &ContainerI, instance: usize) -> usize,
    pub set_size: extern "thiscall" fn(this: &ContainerI, instance: usize, size: usize),
    pub get_mut: extern "thiscall" fn(this: &ContainerI, instance: usize, index: usize) -> usize,
    pub get_const: extern "thiscall" fn(this: &ContainerI, instance: usize, index: usize) -> usize,
    pub clear: extern "thiscall" fn(this: &ContainerI, instance: usize),
    pub push: extern "thiscall" fn(this: &ContainerI, instance: usize, value: usize) -> usize,
    pub pop: extern "thiscall" fn(this: &ContainerI, instance: usize),
    pub get_fixed_size: extern "thiscall" fn(this: &ContainerI) -> i32,
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
    pub destructor: extern "thiscall" fn(this: &mut MapConstIterI),
    pub deleter: extern "thiscall" fn(this: &MapConstIterI),
    pub has_next: extern "thiscall" fn(this: &MapConstIterI) -> bool,
    pub next: extern "thiscall" fn(this: &mut MapConstIterI) -> usize,
    pub get_key: extern "thiscall" fn(this: &MapConstIterI) -> usize,
    pub get_value: extern "thiscall" fn(this: &MapConstIterI) -> usize,
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
    pub destructor: extern "thiscall" fn(this: &MapI),
    pub deleter: extern "thiscall" fn(this: &MapI),
    pub get_size: extern "thiscall" fn(this: &MapI, instance: usize) -> usize,
    pub reserve_size: extern "thiscall" fn(this: &MapI, instance: usize, size: usize),
    pub finalize: extern "thiscall" fn(this: &MapI, instance: usize),
    pub find: extern "thiscall" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub clear: extern "thiscall" fn(this: &MapI, instance: usize),
    pub create: extern "thiscall" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub inplace_ctor: extern "thiscall" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub inplace_dtor: extern "thiscall" fn(this: &MapI, instance: usize, key: usize),
    pub erase: extern "thiscall" fn(this: &MapI, instance: usize, key: usize) -> usize,
    pub iter_mut: extern "thiscall" fn(this: &MapI, instance: usize) -> usize,
    pub iter_const: extern "thiscall" fn(this: &MapI, instance: usize) -> &mut MapConstIterI,
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
pub struct Property {
    pub other_class: Option<&'static Class>,
    pub hash: u32,
    pub offset: u32,
    pub bitmask: u8,
    pub value_type: BinType,
    pub container: Option<&'static ContainerI>,
    pub map: Option<&'static MapI>,
    // added in 13.13
    pub unkptr: usize,
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

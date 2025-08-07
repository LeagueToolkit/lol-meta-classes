use super::throw;

#[derive(Clone, Copy)]
pub struct Ptr<'a> {
    pub data: &'a [u8],
    pub index: usize,
}

pub struct Table<'a> {
    offset: Ptr<'a>,
    fields: Vec<u16>,
}

pub trait ReadPtr<'a>: Sized {
    const SIZE: usize;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String>;
}

impl<'a> Ptr<'a> {
    pub fn new(data: &'a [u8], index: usize) -> Result<Self, String> {
        if index > data.len() {
            throw("Index out of range!")
        } else {
            Ok(Self { data, index })
        }
    }

    pub fn get<T: ReadPtr<'a>>(self) -> Result<T, String> {
        ReadPtr::from_ptr(self)
    }

    pub fn add_relative(self, relative: isize) -> Result<Option<Self>, String> {
        let data = self.data;
        let index = self.index;
        if relative < 0 {
            if -relative as usize > self.index {
                throw("Failed to add relative would underflow")
            } else {
                Ok(Some(Ptr {
                    data,
                    index: index - (-relative as usize),
                }))
            }
        } else if relative > 0 {
            if relative as usize > data.len() - index {
                throw("Failed to add relative would overflow")
            } else {
                Ok(Some(Ptr {
                    data,
                    index: index + (relative as usize),
                }))
            }
        } else {
            Ok(None)
        }
    }

    pub fn add_offset(self, offset: usize) -> Result<Self, String> {
        let data = self.data;
        let index = self.index;
        if offset > data.len() - index {
            throw("Failed to add offset would overflow!")
        } else {
            Ok(Ptr {
                data,
                index: index + offset,
            })
        }
    }
}

impl<'a> Table<'a> {
    pub fn get_ptr(&self, index: usize) -> Result<Option<Ptr<'a>>, String> {
        if index >= self.fields.len() {
            Ok(None)
        } else {
            let offset = self.fields[index] as usize;
            if offset == 0 {
                Ok(None)
            } else {
                Ok(Some(self.offset.add_offset(offset)?))
            }
        }
    }

    pub fn get<T: ReadPtr<'a>>(&self, index: usize) -> Result<Option<T>, String> {
        Ok(if let Some(ptr) = self.get_ptr(index)? {
            Some(T::from_ptr(ptr)?)
        } else {
            None
        })
    }

    pub fn get_or_default<T: ReadPtr<'a> + Default>(&self, index: usize) -> Result<T, String> {
        Ok(if let Some(ptr) = self.get_ptr(index)? {
            T::from_ptr(ptr)?
        } else {
            T::default()
        })
    }

    pub fn get_or_error<T: ReadPtr<'a> + Default>(&self, index: usize) -> Result<T, String> {
        if let Some(ptr) = self.get_ptr(index)? {
            T::from_ptr(ptr)
        } else {
            throw("Can not be null!")
        }
    }
}

impl<'a> ReadPtr<'a> for bool {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if let Ok(result) = u8::from_ptr(offset) {
            Ok(result != 0)
        } else {
            throw("Failed to read bool")
        }
    }
}

impl<'a> ReadPtr<'a> for u8 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for i8 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for u16 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for i16 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for u32 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for i32 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for u64 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for i64 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for f32 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for f64 {
    const SIZE: usize = std::mem::size_of::<Self>();
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if offset.data.len() - offset.index >= Self::SIZE {
            let mut buffer = [0u8; Self::SIZE];
            buffer[..Self::SIZE].copy_from_slice(&offset.data[offset.index..][..Self::SIZE]);
            Ok(Self::from_le_bytes(buffer))
        } else {
            throw("Failed to read num")
        }
    }
}

impl<'a> ReadPtr<'a> for Option<Ptr<'a>> {
    const SIZE: usize = i32::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let relative = i32::from_ptr(offset)?;
        offset.add_relative(relative as isize)
    }
}

impl<'a> ReadPtr<'a> for String {
    const SIZE: usize = Option::<Ptr>::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if let Some(offset) = Option::<Ptr>::from_ptr(offset)? {
            let size = u32::from_ptr(offset)? as usize;
            let data = offset.data;
            let index = offset.index + Self::SIZE;
            if data.len() - index >= size {
                if let Ok(result) = std::str::from_utf8(&data[index..][..size]) {
                    Ok(result.to_string())
                } else {
                    throw("Failed to read str encoding")
                }
            } else {
                throw("Failed to read str data")
            }
        } else {
            Ok("".to_string())
        }
    }
}

impl<'a, T: ReadPtr<'a>> ReadPtr<'a> for Vec<T> {
    const SIZE: usize = Option::<Ptr>::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if let Some(offset) = Option::<Ptr>::from_ptr(offset)? {
            let size = u32::from_ptr(offset)? as usize;
            let mut offset = offset.add_offset(u32::SIZE)?;
            if (offset.data.len() - offset.index) / T::SIZE >= size {
                let mut results = Vec::new();
                for _ in 0..size {
                    results.push(T::from_ptr(offset)?);
                    offset = offset.add_offset(T::SIZE)?;
                }
                Ok(results)
            } else {
                throw("Not enough storage for vector!")
            }
        } else {
            Ok(Vec::new())
        }
    }
}

impl<'a> ReadPtr<'a> for Table<'a> {
    const SIZE: usize = Option::<Ptr>::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        if let Some(offset) = Option::<Ptr>::from_ptr(offset)? {
            let vtable_relative = i32::from_ptr(offset)? as isize;
            if let Some(vtable_offset) = offset.add_relative(-vtable_relative)? {
                let vtable_size = u16::from_ptr(vtable_offset)?;
                if vtable_size >= 4 {
                    let mut vtable_field_offset = vtable_offset.add_offset(4)?;
                    let mut fields = Vec::new();
                    for _ in 0..(vtable_size - 4) / 2 {
                        let vtable_field = u16::from_ptr(vtable_field_offset)?;
                        fields.push(vtable_field);
                        vtable_field_offset = vtable_field_offset.add_offset(2)?
                    }
                    Ok(Self { offset, fields })
                } else {
                    throw("Failed to read table because vtable is too small")
                }
            } else {
                throw("Vtable size offset can not be null!")
            }
        } else {
            throw("Table can not be null!")
        }
    }
}

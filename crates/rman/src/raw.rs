use super::{
    Chunk,
    fb::{Ptr, ReadPtr, Table},
    re_throw, throw,
};
use std::{
    collections::{HashMap, HashSet},
    io,
};
use zstd;

const CHUNK_LIMIT: u32 = 32 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub struct BundleChunk {
    pub id: u64,
    pub size_compressed: u32,
    pub size_uncompressed: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Bundle {
    pub id: u64,
    pub chunks: Vec<BundleChunk>,
}

#[derive(Clone, Debug, Default)]
pub struct Lang {
    pub id: u8,
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct File {
    pub id: u64,
    pub parent_id: u64,
    pub size: u32,
    pub name: String,
    pub lang_flags: u64,
    pub unk5: u8,
    pub unk6: u8,
    pub link: String,
    pub unk8: u8,
    pub chunk_ids: Vec<u64>,
    pub unk10: u8,
    pub params_index: u8,
    pub permissions: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Dir {
    pub id: u64,
    pub parent_id: u64,
    pub name: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Key {}

#[derive(Clone, Copy, Debug, Default)]
pub struct Params {
    pub unk0: u16,
    pub hash_type: u8,
    pub unk2: u8,
    pub unk3: u32,
    pub max_uncompressed: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Body {
    pub bundles: Vec<Bundle>,
    pub langs: Vec<Lang>,
    pub files: Vec<File>,
    pub dirs: Vec<Dir>,
    pub keys: Vec<Key>,
    pub params: Vec<Params>,
}

#[derive(Clone, Debug, Default)]
pub struct Manifest {
    pub id: u64,
    pub files: Vec<File>,
    chunks: HashMap<u64, Chunk>,
    langs: HashMap<u8, Lang>,
    dirs: HashMap<u64, Dir>,
    params: Vec<Params>,
}

struct Header {
    pub magic: [u8; 4],
    pub version: [u8; 2],
    pub flags: u16,
    pub offset: u32,
    pub size_compressed: u32,
    pub checksum: u64,
    pub size_uncompressed: u32,
}

impl<'a> ReadPtr<'a> for BundleChunk {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            id: table.get_or_default(0)?,
            size_compressed: table.get_or_default(1)?,
            size_uncompressed: table.get_or_default(2)?,
        })
    }
}

impl<'a> ReadPtr<'a> for Bundle {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            id: table.get_or_default(0)?,
            chunks: table.get_or_default(1)?,
        })
    }
}

impl<'a> ReadPtr<'a> for Lang {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            id: table.get_or_default(0)?,
            name: table.get_or_default(1)?,
        })
    }
}

impl<'a> ReadPtr<'a> for Dir {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            id: table.get_or_default(0)?,
            parent_id: table.get_or_default(1)?,
            name: table.get_or_default(2)?,
        })
    }
}

impl<'a> ReadPtr<'a> for File {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            id: table.get_or_default(0)?,
            parent_id: table.get_or_default(1)?,
            size: table.get_or_default(2)?,
            name: table.get_or_default(3)?,
            lang_flags: table.get_or_default(4)?,
            unk5: table.get_or_default(5)?,
            unk6: table.get_or_default(6)?,
            chunk_ids: table.get_or_default(7)?,
            unk8: table.get_or_default(8)?,
            link: table.get_or_default(9)?,
            unk10: table.get_or_default(10)?,
            params_index: table.get_or_default(11)?,
            permissions: table.get_or_default(12)?,
        })
    }
}

impl<'a> ReadPtr<'a> for Key {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let _table = offset.get::<Table>()?;
        Ok(Self {})
    }
}

impl<'a> ReadPtr<'a> for Params {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            unk0: table.get_or_default(0)?,
            hash_type: table.get_or_default(1)?,
            unk2: table.get_or_default(2)?,
            unk3: table.get_or_default(3)?,
            max_uncompressed: table.get_or_default(4)?,
        })
    }
}

impl<'a> ReadPtr<'a> for Body {
    const SIZE: usize = Table::SIZE;
    fn from_ptr(offset: Ptr<'a>) -> Result<Self, String> {
        let table = offset.get::<Table>()?;
        Ok(Self {
            bundles: table.get_or_default(0)?,
            langs: table.get_or_default(1)?,
            files: table.get_or_default(2)?,
            dirs: table.get_or_default(3)?,
            keys: table.get_or_default(4)?,
            params: table.get_or_default(5)?,
        })
    }
}

impl Header {
    pub const SIZE: u32 = 28;
    fn read<R: io::Read>(reader: &mut R) -> Result<Self, io::Error> {
        let mut magic = [0; 4];
        let mut version = [0; 2];
        let mut flags = [0; 2];
        let mut offset = [0; 4];
        let mut size_compressed = [0; 4];
        let mut checksum = [0; 8];
        let mut size_uncompressed = [0; 4];

        reader.read_exact(&mut magic)?;
        reader.read_exact(&mut version)?;
        reader.read_exact(&mut flags)?;
        reader.read_exact(&mut offset)?;
        reader.read_exact(&mut size_compressed)?;
        reader.read_exact(&mut checksum)?;
        reader.read_exact(&mut size_uncompressed)?;

        Ok(Header {
            magic,
            version,
            flags: u16::from_le_bytes(flags),
            offset: u32::from_le_bytes(offset),
            size_compressed: u32::from_le_bytes(size_compressed),
            checksum: u64::from_le_bytes(checksum),
            size_uncompressed: u32::from_le_bytes(size_uncompressed),
        })
    }
}

impl Manifest {
    fn verify_filename(name: &str) -> Result<(), String> {
        if name == "." || name == ".." {
            throw("Name can not be . or ..!")
        } else {
            for c in name.chars() {
                if c.is_alphanumeric() {
                    continue;
                }
                if c == '.' || c == ' ' || c == '+' || c == '-' || c == '_' {
                    continue;
                }
                return throw(format!("Illegal character in name: {}", c));
            }
            Ok(())
        }
    }

    pub fn read<R: io::Read>(reader: &mut R) -> Result<Self, String> {
        let header = re_throw(Header::read(reader), "Failed to read header")?;
        if header.offset < Header::SIZE {
            throw("Body offset at bad position!")?;
        }
        for _ in header.offset..Header::SIZE {
            let mut one = [0u8; 1];
            re_throw(reader.read(&mut one), "Failed to skip")?;
        }
        let mut data_compressed = Vec::new();
        data_compressed.resize(header.size_compressed as usize, 0u8);
        re_throw(
            reader.read_exact(&mut data_compressed),
            "Failed to read compressed",
        )?;
        let data = re_throw(
            zstd::stream::decode_all(io::Cursor::new(&data_compressed)),
            "Failed to read data",
        )?;
        let body = Ptr::new(&data, 0)?.get::<Body>()?;
        let mut chunks = HashMap::new();
        for bundle in body.bundles.iter() {
            if bundle.id == 0 {
                throw("Bundle id can not be 0!")?;
            }
            let mut offset_compressed = 0u64;
            for chunk in bundle.chunks.iter() {
                if chunk.id == 0 {
                    throw("Chunk id can not be 0!")?;
                }
                chunks.insert(
                    chunk.id,
                    Chunk {
                        chunk_id: chunk.id,
                        bundle_id: bundle.id,
                        size_compressed: chunk.size_compressed,
                        size_uncompressed: chunk.size_uncompressed,
                        offset_compressed: offset_compressed as u32,
                        offset_uncompressed: 0,
                    },
                );
                offset_compressed += chunk.size_compressed as u64;
                if offset_compressed > u32::MAX as u64 {
                    throw("Compressed offset would go out of 4GB boundary!")?;
                }
            }
        }
        let mut langs = HashMap::new();
        for lang in body.langs {
            //Self::verify_filename(&lang.name)?;
            langs.insert(lang.id, lang);
        }
        let mut dirs = HashMap::new();
        for dir in body.dirs {
            //Self::verify_filename(&dir.name)?;
            dirs.insert(dir.id, dir);
        }
        let mut params = Vec::new();
        for param in body.params {
            if param.max_uncompressed > CHUNK_LIMIT {
                throw("Chunk params go over uncompressed CHUNK_LIMIT!")?;
            }
            params.push(param);
        }
        for file in body.files.iter() {
            if file.id == 0 {
                throw("File id can not be 0!")?;
            }
            //Self::verify_filename(&file.name)?;
        }
        Ok(Self {
            id: header.checksum,
            files: body.files,
            chunks,
            langs,
            dirs,
            params,
        })
    }

    pub fn get_file_name(&self, name: &str, parent_id: u64) -> Result<String, String> {
        let mut name = name.to_string();
        let org_parent_id = parent_id;
        let mut parent_id = parent_id;
        loop {
            if let Some(dir) = self.dirs.get(&parent_id) {
                if dir.name == "" {
                    break;
                }
                name = format!("{}/{}", dir.name, name);
                parent_id = dir.parent_id;
                if parent_id == org_parent_id {
                    throw("Directory cycle detected!")?;
                }
            } else {
                throw("Failed to find dir by id!")?;
            }
        }
        Ok(name)
    }

    pub fn get_langs(&self, lang_flags: u64) -> Result<HashSet<String>, String> {
        let mut langs = HashSet::new();
        for i in 0..32 {
            if lang_flags & (1 << i) != 0 {
                if let Some(lang) = self.langs.get(&(i + 1)) {
                    langs.insert(lang.name.to_lowercase());
                } else {
                    throw("Failed to find lang by id!")?;
                }
            }
        }
        if langs.len() == 0 {
            langs.insert("none".to_string());
        }
        Ok(langs)
    }

    pub fn get_chunk(&self, chunk_id: u64) -> Result<Chunk, String> {
        if let Some(&chunk_data) = self.chunks.get(&chunk_id) {
            Ok(chunk_data)
        } else {
            throw("Failed to find chunk bundle by id!")
        }
    }

    pub fn get_chunks(&self, chunk_ids: &[u64]) -> Result<Vec<Chunk>, String> {
        let mut offset_uncompressed = 0u64;
        let mut results = Vec::new();
        for chunk_id in chunk_ids {
            let mut chunk = self.get_chunk(*chunk_id)?;
            chunk.offset_uncompressed = offset_uncompressed as u32;
            results.push(chunk);
            offset_uncompressed += chunk.size_uncompressed as u64;
            if offset_uncompressed > u32::MAX as u64 {
                throw("Uncompressed offset would go out of 4GB boundary!")?;
            }
        }
        Ok(results)
    }

    pub fn get_params(&self, params_id: u8) -> Result<Params, String> {
        if let Some(&params) = self.params.get(params_id as usize) {
            Ok(params)
        } else {
            throw("Failed to find chunk bundle by id!")
        }
    }
}

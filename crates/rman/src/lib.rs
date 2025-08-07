mod dl;
mod fb;
mod raw;
use core::fmt::Display;
pub use dl::*;
use sha2::{Digest, Sha256, Sha512};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    convert::{TryFrom, TryInto},
    fs,
    io::{self, Read},
};

fn throw<T, S: std::string::ToString>(msg: S) -> Result<T, String> {
    Err(msg.to_string())
}

fn re_throw<T, S: Display, E: Display>(value: Result<T, E>, msg: S) -> Result<T, String> {
    match value {
        Ok(value) => Ok(value),
        Err(err) => throw(format!("{}: {}", msg, err)),
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashType {
    NONE,
    SHA512,
    SHA256,
    HKDF,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Chunk {
    pub chunk_id: u64,
    pub bundle_id: u64,
    pub size_compressed: u32,
    pub size_uncompressed: u32,
    pub offset_compressed: u32,
    pub offset_uncompressed: u32,
}

#[derive(Clone, Debug)]
pub struct File {
    pub id: u64,
    pub name: String,
    pub link_name: String,
    pub size: u32,
    pub max_uncompressed: u32,
    pub hash_type: HashType,
    pub langs: HashSet<String>,
    pub chunks: Vec<Chunk>,
}

#[derive(Clone, Debug)]
pub struct Manifest {
    pub id: u64,
    pub files: Vec<File>,
}

impl Default for HashType {
    fn default() -> Self {
        HashType::NONE
    }
}

impl HashType {
    fn compute_sha256(input: &[u8]) -> u64 {
        let buffer = Sha256::digest(input);
        let mut result = [0u8; 8];
        result[..8].copy_from_slice(&buffer[..8]);
        u64::from_le_bytes(result)
    }

    fn compute_sha512(input: &[u8]) -> u64 {
        let buffer = Sha512::digest(input);
        let mut result = [0u8; 8];
        result[..8].copy_from_slice(&buffer[..8]);
        u64::from_le_bytes(result)
    }

    fn compute_hkdf(input: &[u8]) -> u64 {
        let key = Sha256::digest(input);
        let mut ipad = [0u8; 64];
        let mut opad = [0u8; 64];
        ipad.fill(0x36);
        opad.fill(0x5C);
        for i in 0..32 {
            ipad[i] ^= key[i];
            opad[i] ^= key[i];
        }
        let index = u32::to_be_bytes(1);
        let mut buffer = Sha256::new().chain(&ipad).chain(&index).finalize();
        buffer = Sha256::new().chain(&opad).chain(&buffer).finalize();
        let mut result = [0u8; 8];
        result.copy_from_slice(&buffer[..8]);
        for _ in 0..31 {
            buffer = Sha256::new().chain(&ipad).chain(&buffer).finalize();
            buffer = Sha256::new().chain(&opad).chain(&buffer).finalize();
            for i in 0..8 {
                result[i] ^= buffer[i];
            }
        }
        u64::from_le_bytes(result)
    }

    pub fn compute(self, input: &[u8]) -> u64 {
        match self {
            Self::NONE => 0,
            Self::SHA256 => Self::compute_sha256(input),
            Self::SHA512 => Self::compute_sha512(input),
            Self::HKDF => Self::compute_hkdf(input),
        }
    }
}

impl TryFrom<u8> for HashType {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HashType::NONE),
            1 => Ok(HashType::SHA512),
            2 => Ok(HashType::SHA256),
            3 => Ok(HashType::HKDF),
            _ => throw("Bad HashType"),
        }
    }
}

impl File {
    pub fn download_if<F: FnMut(&Chunk) -> bool>(&self, mut check_chunk: F) -> DownloadFile {
        let name = self.name.to_string();
        let size = self.size;
        let max_uncompressed = self.max_uncompressed;
        let mut bundles = HashMap::new();
        for chunk in &self.chunks {
            if check_chunk(chunk) {
                bundles
                    .entry(chunk.bundle_id)
                    .or_insert_with(|| DownloadBundle {
                        name: format!("{:016X}.bundle", chunk.bundle_id),
                        offset_compressed: BTreeMap::new(),
                    })
                    .offset_compressed
                    .entry(chunk.offset_compressed)
                    .or_insert_with(|| DownloadChunk {
                        size_compressed: chunk.size_compressed,
                        size_uncompressed: chunk.size_uncompressed,
                        offset_uncompressed: BTreeSet::new(),
                    })
                    .offset_uncompressed
                    .insert(chunk.offset_uncompressed);
            }
        }
        DownloadFile {
            name,
            size,
            max_uncompressed,
            bundles,
        }
    }

    pub fn download_all(&self) -> DownloadFile {
        self.download_if(|_| true)
    }

    pub fn download_checked<R: io::Read + io::Seek>(&self, reader: &mut R) -> DownloadFile {
        let mut buffer = Vec::with_capacity(self.max_uncompressed as usize);
        self.download_if(|chunk| {
            if let Ok(_) = reader.seek(io::SeekFrom::Start(chunk.offset_uncompressed as u64)) {
                buffer.resize(chunk.size_uncompressed as usize, 0u8);
                if let Ok(_) = reader.read_exact(&mut buffer) {
                    if self.hash_type.compute(&buffer) == chunk.chunk_id {
                        return false;
                    }
                }
            }
            true
        })
    }

    pub fn download_checked_in_dir(&self, dir: &str) -> DownloadFile {
        if let Ok(mut file) = fs::File::open(format!("{}/{}", dir, &self.name)) {
            self.download_checked(&mut file)
        } else {
            self.download_all()
        }
    }

    pub fn verify(&self, dir: &str) -> bool {
        if let Ok(mut file) = fs::File::open(format!("{}/{}", dir, &self.name)) {
            let mut buffer = Vec::with_capacity(self.max_uncompressed as usize);
            for chunk in &self.chunks {
                buffer.resize(chunk.size_uncompressed as usize, 0u8);
                if let Ok(_) = file.read_exact(&mut buffer) {
                    if self.hash_type.compute(&buffer) != chunk.chunk_id {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl Manifest {
    pub fn read<R: io::Read>(reader: &mut R) -> Result<Self, String> {
        let raw = raw::Manifest::read(reader)?;
        let mut files = Vec::new();
        for file in &raw.files {
            let id = file.id;
            let name = raw.get_file_name(&file.name, file.parent_id)?;
            let link_name = file.link.to_string();
            let size = file.size;
            let params = raw.get_params(file.params_index)?;
            let hash_type = params.hash_type.try_into()?;
            let langs = raw.get_langs(file.lang_flags)?;
            let max_uncompressed = params.max_uncompressed;
            let chunks = raw.get_chunks(&file.chunk_ids)?;
            for chunk in &chunks {
                if chunk.size_uncompressed > max_uncompressed {
                    return throw("Chunk too big!");
                }
                if chunk.offset_uncompressed + chunk.size_uncompressed > size {
                    return throw("Chunk would go outside the file!");
                }
            }
            files.push(File {
                id,
                name,
                link_name,
                size,
                max_uncompressed,
                hash_type,
                langs,
                chunks,
            });
        }
        Ok(Self { id: raw.id, files })
    }

    pub fn download(agent: &mut ureq::Agent, url: &str) -> Result<Self, String> {
        if url.starts_with("https://") || url.starts_with("http://") {
            let response = re_throw(agent.get(url).call(), "Failed to request manifest!")?;
            let mut reader = response.into_reader();
            Self::read(reader.by_ref())
        } else {
            let mut file = re_throw(fs::File::open(url), "Failed to open manifest file!")?;
            Self::read(&mut file)
        }
    }
}

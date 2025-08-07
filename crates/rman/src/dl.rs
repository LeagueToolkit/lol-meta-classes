use super::{re_throw, throw};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    io::{self, Read},
    ops::Range,
};
use ureq;
use zstd;

#[derive(Clone, Debug, Default)]
pub struct DownloadChunk {
    pub size_compressed: u32,
    pub size_uncompressed: u32,
    pub offset_uncompressed: BTreeSet<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadBundle {
    pub name: String,
    pub offset_compressed: BTreeMap<u32, DownloadChunk>,
}

#[derive(Clone, Debug, Default)]
pub struct DownloadFile {
    pub name: String,
    pub size: u32,
    pub max_uncompressed: u32,
    pub bundles: HashMap<u64, DownloadBundle>,
}

impl DownloadChunk {
    pub fn write_from<W: io::Write + io::Seek>(
        &self,
        src: &[u8],
        writer: &mut W,
    ) -> Result<(), String> {
        if src.len() < self.size_compressed as usize {
            return throw("Chunk compressed data too small!");
        }
        let compressed = &src[..self.size_compressed as usize];
        let uncompressed = re_throw(zstd::decode_all(compressed), "Failed to decompress chunk!")?;
        for &offset_uncompressed in &self.offset_uncompressed {
            re_throw(
                writer.seek(io::SeekFrom::Start(offset_uncompressed as u64)),
                "Failed to seek to chunk!",
            )?;
            re_throw(writer.write_all(&uncompressed), "Failed to write chunk!")?;
        }
        Ok(())
    }
}

impl DownloadBundle {
    pub fn get_range(&self) -> Range<u32> {
        if let Some((first_offset, _)) = self.offset_compressed.first_key_value() {
            if let Some((last_offset, last_chunk)) = self.offset_compressed.last_key_value() {
                return first_offset + 0..last_offset + last_chunk.size_compressed;
            }
        }
        0..0
    }

    pub fn download<W: io::Write + io::Seek>(
        &self,
        agent: &mut ureq::Agent,
        cdn: &str,
        writer: &mut W,
    ) -> Result<u32, String> {
        let range = self.get_range();
        let response = re_throw(
            agent
                .get(&format!("{}/{}", cdn, self.name))
                .set("Range", &format!("bytes={}-{}", range.start, range.end - 1))
                .call(),
            "Failed to download!",
        )?;
        let mut buffer = Vec::with_capacity(range.len());
        buffer.resize(range.len(), 0u8);
        re_throw(
            response.into_reader().read_exact(&mut buffer),
            "Failed to read response!",
        )?;
        for (&offset_compressed, chunk) in &self.offset_compressed {
            let compressed = &buffer[(offset_compressed - range.start) as usize..];
            chunk.write_from(compressed, writer)?;
        }
        Ok(range.len() as u32)
    }
}

impl DownloadFile {
    pub fn get_total_size(&self) -> u32 {
        self.bundles
            .values()
            .map(|bundle| bundle.get_range().len())
            .sum::<usize>() as u32
    }

    pub fn download_with_progress<W: io::Write + io::Seek, F: FnMut(u32)>(
        &self,
        agent: &mut ureq::Agent,
        cdn: &str,
        writer: &mut W,
        mut progress: F,
    ) -> Result<(), String> {
        for (_, bundle) in &self.bundles {
            let done_count = bundle.download(agent, cdn, writer)?;
            progress(done_count);
        }
        Ok(())
    }

    pub fn download<W: io::Write + io::Seek>(
        &self,
        agent: &mut ureq::Agent,
        cdn: &str,
        writer: &mut W,
    ) -> Result<(), String> {
        self.download_with_progress(agent, cdn, writer, |_| ())
    }

    pub fn download_in_dir_with_progress<F: FnMut(u32)>(
        &self,
        dir: &str,
        agent: &mut ureq::Agent,
        cdn: &str,
        mut progress: F,
    ) -> Result<(), String> {
        let path = format!("{}/{}", dir, &self.name);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            re_throw(fs::create_dir_all(parent), "Failed to create file dirs!")?;
        }
        let mut writer = re_throw(fs::File::create(&path), "Failed to create file!")?;
        self.download_with_progress(agent, cdn, &mut writer, progress)?;
        re_throw(writer.set_len(self.size as u64), "Failed to set file len!")?;
        Ok(())
    }

    pub fn download_in_dir(
        &self,
        dir: &str,
        agent: &mut ureq::Agent,
        cdn: &str,
    ) -> Result<(), String> {
        self.download_in_dir_with_progress(dir, agent, cdn, |_| ())
    }
}

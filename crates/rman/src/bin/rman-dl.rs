//! Download every file in an RMAN manifest from a Riot CDN.
//!
//! ```bash
//! cargo run --release --bin rman-dl -- <manifest-url-or-path> <out-dir> <cdn-bundles-url> [langs]
//! ```
//!
//! `langs` is an optional comma-separated allowlist (e.g. `en_us`). Files with no
//! language tag are always kept; language-tagged files are kept only if they match.
//! Pass `-` as the out-dir to list the manifest instead of downloading.
//!
//! Parallelism is per-bundle *within* each file, so a single multi-GiB file
//! (e.g. a UE5 `.ucas`) still saturates the link.
//!
//! Each file is staged as `<name>.part` and renamed on completion, so an interrupted
//! run resumes correctly — the target is pre-sized with `set_len`, and a length check
//! alone cannot distinguish a finished file from an abandoned one.

use rayon::prelude::*;
use std::{
    collections::HashSet,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

const RETRIES: usize = 5;

fn get_range(agent: &ureq::Agent, url: &str, start: u32, end: u32) -> Result<Vec<u8>, String> {
    let mut last = String::new();
    for attempt in 0..RETRIES {
        let req = agent
            .get(url)
            .set("Range", &format!("bytes={}-{}", start, end - 1));
        match req.call() {
            Ok(response) => {
                let mut buffer = vec![0u8; (end - start) as usize];
                match response.into_reader().read_exact(&mut buffer) {
                    Ok(_) => return Ok(buffer),
                    Err(e) => last = format!("read: {}", e),
                }
            }
            Err(e) => last = format!("http: {}", e),
        }
        std::thread::sleep(std::time::Duration::from_millis(300 * (1 << attempt) as u64));
    }
    Err(format!("{} [{}-{}]: {}", url, start, end, last))
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: rman-dl <manifest-url-or-path> <out-dir> <cdn-bundles-url> [langs]");
        std::process::exit(2);
    }
    let (src, out_dir, cdn) = (&args[1], &args[2], args[3].trim_end_matches('/').to_string());
    let allow: Option<HashSet<String>> = args.get(4).map(|s| {
        s.split(',')
            .map(|l| l.trim().to_lowercase())
            .filter(|l| !l.is_empty())
            .collect()
    });

    let mut agent = ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(30))
        .timeout_read(std::time::Duration::from_secs(120))
        .build();
    let manifest = rman::Manifest::download(&mut agent, src)?;

    // "none" is the sentinel the parser assigns to files with no language flags.
    let wanted: Vec<&rman::File> = manifest
        .files
        .iter()
        .filter(|f| match &allow {
            None => true,
            Some(allow) => f.langs.iter().any(|l| l == "none" || allow.contains(l)),
        })
        .collect();

    let total: u64 = wanted.iter().map(|f| f.size).sum();
    let skipped: u64 = manifest.files.iter().map(|f| f.size).sum::<u64>() - total;
    eprintln!(
        "manifest {:016X}: {}/{} files, {:.2} GiB ({:.2} GiB filtered out by lang)",
        manifest.id,
        wanted.len(),
        manifest.files.len(),
        total as f64 / (1u64 << 30) as f64,
        skipped as f64 / (1u64 << 30) as f64
    );

    // `-` as the out-dir lists the manifest instead of downloading it.
    if out_dir == "-" {
        for file in &wanted {
            let mut langs: Vec<&String> = file.langs.iter().collect();
            langs.sort();
            println!(
                "{}\t{}\t{}\t{}",
                file.size,
                file.chunks.len(),
                langs
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(","),
                file.name
            );
        }
        return Ok(());
    }

    let done = AtomicU64::new(0);
    let started = Instant::now();
    // Largest first: the big files dominate wall-clock, start them early.
    let mut files = wanted;
    files.sort_by_key(|f| std::cmp::Reverse(f.size));

    for file in &files {
        let path = format!("{}/{}", out_dir, file.name);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {}", path, e))?;
        }

        // Only a renamed (never a `.part`) file counts as complete.
        if file.size > 0 && fs::metadata(&path).map(|m| m.len()).unwrap_or(0) == file.size {
            let n = done.fetch_add(file.size, Ordering::Relaxed) + file.size;
            eprintln!(
                "[{:5.1}%] have {} ({} bytes)",
                n as f64 * 100.0 / total as f64,
                file.name,
                file.size
            );
            continue;
        }

        let part = format!("{}.part", path);
        let handle = fs::File::create(&part).map_err(|e| format!("create {}: {}", part, e))?;
        handle
            .set_len(file.size)
            .map_err(|e| format!("set_len {}: {}", part, e))?;
        let writer = Mutex::new(handle);

        let dl = file.download_all();
        let errors: Vec<String> = dl
            .bundles
            .par_iter()
            .map(|(_, bundle)| -> Result<(), String> {
                let range = bundle.get_range();
                let url = format!("{}/{}", cdn, bundle.name);
                let buffer = get_range(&agent, &url, range.start, range.end)?;
                for (&offset_compressed, chunk) in &bundle.offset_compressed {
                    let at = (offset_compressed - range.start) as usize;
                    let compressed = &buffer[at..at + chunk.size_compressed as usize];
                    // Decompress outside the writer lock.
                    let plain = zstd::decode_all(compressed)
                        .map_err(|e| format!("{}: decompress: {}", bundle.name, e))?;
                    if plain.len() != chunk.size_uncompressed as usize {
                        return Err(format!(
                            "{}: chunk size {} != expected {}",
                            bundle.name,
                            plain.len(),
                            chunk.size_uncompressed
                        ));
                    }
                    let mut w = writer.lock().unwrap();
                    for &offset in &chunk.offset_uncompressed {
                        w.seek(SeekFrom::Start(offset))
                            .and_then(|_| w.write_all(&plain))
                            .map_err(|e| format!("{}: write: {}", part, e))?;
                    }
                }
                Ok(())
            })
            .filter_map(Result::err)
            .collect();

        writer
            .into_inner()
            .unwrap()
            .flush()
            .map_err(|e| format!("flush {}: {}", part, e))?;

        if !errors.is_empty() {
            // Leave the .part behind so a retry does not treat it as complete.
            return Err(format!("{}: {}", file.name, errors.join("; ")));
        }
        fs::rename(&part, &path).map_err(|e| format!("rename {}: {}", part, e))?;

        let n = done.fetch_add(file.size, Ordering::Relaxed) + file.size;
        let secs = started.elapsed().as_secs_f64();
        eprintln!(
            "[{:5.1}%] {} ({} bytes, {} bundles) — {:.1} MiB/s",
            n as f64 * 100.0 / total as f64,
            file.name,
            file.size,
            dl.bundles.len(),
            n as f64 / (1u64 << 20) as f64 / secs.max(0.001)
        );
    }

    // Report the langs actually written, not every lang the manifest mentions.
    let langs: HashSet<&String> = files.iter().flat_map(|f| f.langs.iter()).collect();
    let mut langs: Vec<&&String> = langs.iter().collect();
    langs.sort();
    eprintln!(
        "done in {:.0}s; {} files, langs written: {}",
        started.elapsed().as_secs_f64(),
        files.len(),
        langs
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    Ok(())
}

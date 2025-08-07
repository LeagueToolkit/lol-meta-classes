use clap::Parser;
use regex::bytes::Regex;
use serde_json::json;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
#[macro_use]
extern crate lazy_static;

/// A tool to dump metaclass information from League of Legends executables
#[derive(Parser)]
#[command(name = "dumper")]
#[command(about = "Dumps metaclass information from League of Legends executables")]
struct Args {
    /// Input executable file to analyze
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file for the dumped metadata (JSON format)
    #[arg(short, long, value_name = "OUTPUT")]
    output: PathBuf,
}

#[allow(dead_code)]
mod loader;
#[allow(dead_code)]
mod meta;
#[allow(dead_code)]
mod meta_dump;

type MetaVector = meta::RiotVector<&'static meta::Class>;

const PATTERN_CLASSES: &str =
    r"(?s-u)\x48\x8D\x3D(....)\x48?\x89\xDE\xE8....\x48\x83\xC4\x08\x5B\x5D\xFF\x60\x10";

#[allow(dead_code)]
const PATTERN_VERSION: &str = r"(?s-u)\x00Releases/(\d+(\.\d+)+)\x00";

/*
version_tag     db 'VersionInfoTag!',0
version_patch   dd 6AABBCh
build_date      db '16:49:39',0
build_time      db 'Jul 24 2025',0
                db    2
version_major   dw 0Fh
version_minor   dw 0Fh
                dq 0
*/
#[allow(dead_code)]
const PATTERN_VERSION2: &str = r"(?s-u)VersionInfoTag!\x00(....)\d{1,2}:\d{1,2}:\d{1,2}\x00\w{1,4} \d{1,2} \d{4}\x00\x02(..)(..)\x00{8,8}";

fn find_version(data: &[u8]) -> Option<String> {
    Regex::new(PATTERN_VERSION)
        .expect("Bad regex PATTERN_VERSION!")
        .captures(data)
        .and_then(|captures| captures.get(1))
        .map(|x| { String::from_utf8_lossy(x.as_bytes()) }.to_string())
}

fn find_version2(data: &[u8]) -> Option<String> {
    Regex::new(PATTERN_VERSION2)
        .expect("Bad regex PATTERN_VERSION2!")
        .captures(data)
        .map(|captures| {
            let patch = captures
                .get(1)
                .expect("PATTERN_VERSION2 missing capture group1")
                .as_bytes();
            let major = captures
                .get(2)
                .expect("PATTERN_VERSION2 missing capture group2")
                .as_bytes();
            let minor = captures
                .get(3)
                .expect("PATTERN_VERSION2 missing capture group3")
                .as_bytes();
            let patch = u32::from_le_bytes(patch.try_into().expect("Invalid patch length!"));
            let major = u16::from_le_bytes(major.try_into().expect("Invalid major length!"));
            let minor = u16::from_le_bytes(minor.try_into().expect("Invalid minor length!"));
            format!("{}.{}.{}", major, minor, patch)
        })
}

fn find_classes(data: &[u8]) -> &MetaVector {
    Regex::new(PATTERN_CLASSES)
        .expect("Bad regex PATTERN_CLASSES!")
        .captures(data)
        .and_then(|captures| captures.get(1))
        .map(|x| unsafe {
            let base = data.as_ptr().offset(x.end() as _);
            let rel = x.as_bytes().as_ptr().cast::<i32>().read_unaligned();
            base.offset(rel as _)
        })
        .map(|x| x as *const MetaVector)
        .and_then(|x| unsafe { x.as_ref() })
        .expect("Failed to find PATTERN_CLASSES!")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    eprintln!("Mapping image...");
    let map = loader::map_image(&args.input)?;
    let data = unsafe { &*std::ptr::slice_from_raw_parts(map.data(), map.len()) };
    eprintln!("Mapped at: {:#x}", data.as_ptr() as usize);

    eprintln!("Extracting version info...");
    let version = find_version(data).or_else(|| find_version2(data));
    eprintln!("Found version: {:?}", version);

    eprintln!("Finding metaclasses...");
    let classes = find_classes(data);
    eprintln!(
        "Found classes at {:#x} len {:#x}",
        classes as *const _ as usize,
        classes.slice().len()
    );

    eprintln!("Processing classes...");
    let meta_info = json!({
        "version": version.unwrap_or_else(|| "unknown".to_string()),
        "classes": meta_dump::dump_class_list(data.as_ptr() as usize, classes.slice()),
    });

    eprintln!("Writing output to {}...", args.output.display());
    let output_file = File::create(&args.output)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &meta_info).expect("Failed to serialize json!");

    eprintln!("Done!");
    Ok(())
}

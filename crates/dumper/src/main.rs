use clap::Parser;
use regex::bytes::Regex;
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

mod diag;
#[allow(dead_code)]
mod loader;
#[allow(dead_code)]
mod meta;
#[allow(dead_code)]
mod meta_dump;

/// The registry: a vector of raw class pointers, interpreted through
/// [`meta::ClassRef`] once the image's layout is known.
type MetaVector = meta::RiotVector<*const ()>;

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

The pattern ends at `version_minor`. The number of zero bytes that follow it
depends on the build: 16.14 has 10, 16.21 has 3. 16.21 is also the first build
without the `Releases/` string, so this record is its only version source.
*/
#[allow(dead_code)]
const PATTERN_VERSION2: &str = r"(?s-u)VersionInfoTag!\x00(....)\d{1,2}:\d{1,2}:\d{1,2}\x00\w{1,4} \d{1,2} \d{4}\x00\x02(..)(..)";

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

    // Before anything maps or runs shipped code, so every fault below is caught.
    diag::install();
    if diag::poison_enabled() {
        eprintln!("Import poisoning ENABLED (DUMPER_POISON_IMPORTS)");
    }

    eprintln!("Mapping image...");
    let map = loader::map_image(&args.input)?;
    let data = unsafe { &*std::ptr::slice_from_raw_parts(map.data(), map.len()) };
    eprintln!("Mapped at: {:#x}", data.as_ptr() as usize);
    diag::set_image_range(data.as_ptr() as usize, data.len());

    eprintln!("Extracting version info...");
    let version = find_version(data).or_else(|| find_version2(data));
    eprintln!("Found version: {:?}", version);

    // Record layouts vary by build, so both must be selected before any class
    // or property is read.
    let property_layout = meta::property_layout_for(version.as_deref());
    meta::set_property_layout(property_layout);
    eprintln!(
        "Property layout: {:?} ({} bytes)",
        property_layout,
        meta::PropertyRef::record_size(property_layout)
    );

    // 16.12 uses a different Class layout: one extra field at +0x18.
    let class_layout = meta::class_layout_for(version.as_deref());
    meta::set_class_layout(class_layout);
    eprintln!("Class layout: {:?}", class_layout);

    eprintln!("Finding metaclasses...");
    let classes = find_classes(data);
    eprintln!(
        "Found classes at {:#x} len {:#x}",
        classes as *const _ as usize,
        classes.slice().len()
    );

    // Re-assert our handlers: mod_init and the entry point have run by now, and
    // shipped code (sentry/crashpad) installs its own. Without this the class
    // walk - the part we actually need diagnosed - could fault into their
    // handler instead of ours.
    diag::install();

    eprintln!("Processing classes...");
    let meta_info = meta_dump::dump_meta(
        data.as_ptr() as usize,
        classes.slice(),
        version.unwrap_or_else(|| "unknown".to_string()),
    );

    eprintln!("Writing output to {}...", args.output.display());
    diag::set_phase(diag::Phase::Writing);
    let output_file = File::create(&args.output)?;
    let writer = BufWriter::new(output_file);
    serde_json::to_writer_pretty(writer, &meta_info).expect("Failed to serialize json!");

    let anomalies = diag::anomaly_count();
    if anomalies > 0 {
        eprintln!(
            "WARNING: {} anomal{} tolerated - this dump is NOT trustworthy.",
            anomalies,
            if anomalies == 1 { "y was" } else { "ies were" }
        );
    }

    eprintln!("Done!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a `VersionInfoTag!` record followed by `tail`.
    fn version_record(
        patch: u32,
        time: &str,
        date: &str,
        major: u16,
        minor: u16,
        tail: &[u8],
    ) -> Vec<u8> {
        let mut record = b"0123456789ABCDEFVersionInfoTag!\x00".to_vec();
        record.extend_from_slice(&patch.to_le_bytes());
        record.extend_from_slice(time.as_bytes());
        record.push(0);
        record.extend_from_slice(date.as_bytes());
        record.push(0);
        record.push(2);
        record.extend_from_slice(&major.to_le_bytes());
        record.extend_from_slice(&minor.to_le_bytes());
        record.extend_from_slice(tail);
        record
    }

    #[test]
    fn find_version2_returns_version_if_10_zero_bytes_follow() {
        // The record of 16.14.7949266.
        let mut tail = vec![0u8; 10];
        tail.extend_from_slice(b"St11range_error\x00");
        let data = version_record(7949266, "10:58:30", "Jul 13 2026", 16, 14, &tail);
        assert_eq!(find_version2(&data).as_deref(), Some("16.14.7949266"));
    }

    #[test]
    fn find_version2_returns_version_if_3_zero_bytes_follow() {
        // The record of 16.21.8255794.
        let data = version_record(
            8255794,
            "05:02:55",
            "Oct 06 2026",
            16,
            21,
            b"\x00\x00\x00St11range_error\x00",
        );
        assert_eq!(find_version2(&data).as_deref(), Some("16.21.8255794"));
    }

    #[test]
    fn find_version2_returns_none_if_record_is_not_stamped() {
        // The arm64 slice keeps the placeholder that the build did not fill.
        let data = version_record(0, "HR:MN:SC", "MNT DY YEAR", 0, 0, &[0u8; 16]);
        assert_eq!(find_version2(&data), None);
    }

    #[test]
    fn property_layout_for_returns_v16_14_for_find_version2_output() {
        let data = version_record(8255794, "05:02:55", "Oct 06 2026", 16, 21, &[0u8; 3]);
        let version = find_version2(&data);
        assert_eq!(
            meta::property_layout_for(version.as_deref()),
            meta::PropertyLayout::V16_14
        );
    }
}

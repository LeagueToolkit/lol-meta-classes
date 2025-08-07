use regex::bytes::Regex;
use serde_json::{json, Value};
use std::path::Path;
#[macro_use]
extern crate lazy_static;

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

fn find_version(data: &[u8]) -> String {
    Regex::new(PATTERN_VERSION)
        .expect("Bad regex PATTERN_VERSION!")
        .captures(data)
        .and_then(|captures| captures.get(1))
        .map(|x| { String::from_utf8_lossy(x.as_bytes()) }.to_string())
        .expect("Failed to find PATTERN_CLASSES!")
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

pub fn dump_classes_from_file(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let map = loader::map_image(path)?;
    let data = unsafe { &*std::ptr::slice_from_raw_parts(map.data(), map.len()) };

    let version = find_version(data);
    let classes = find_classes(data);

    Ok(json!({
        "version": version,
        "classes": meta_dump::dump_class_list(data.as_ptr() as usize, classes.slice()),
    }))
}

use mmap::MemoryMap;
use std::fs;
use std::path::Path;

mod macho;
mod stubs;
mod newer;

pub fn map_image<P: AsRef<Path>>(path: P) -> anyhow::Result<MemoryMap> {
    // Read macho
    let data = fs::read(path)?;
    let macho = macho::MachOImage::new(&data)?;
    let map = macho.map_image(&data)?;
    let image = unsafe { &mut *std::ptr::slice_from_raw_parts_mut(map.data(), map.len()) };
    macho.resolve_imports(image, stubs::resolve);
    macho.resolve_exports(image, newer::install_hook);
    macho.fixup_tlv(image);
    macho.run_mod_init(image);

    unsafe {
        let run_until_alert_addr = stubs::resolve("run_until_alert");
        let run_until_alert: extern "C" fn(func: extern "C" fn()) = std::mem::transmute(run_until_alert_addr);
        let entry = macho.get_entry(image);
        run_until_alert(entry);
    }
    Ok(map)
}

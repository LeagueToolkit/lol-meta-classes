use mmap::MemoryMap;
use std::fs;
use std::path::Path;

mod macho;
mod newer;
mod stubs;

pub fn map_image<P: AsRef<Path>>(path: P) -> anyhow::Result<MemoryMap> {
    // Read macho
    eprintln!("  Reading file...");
    let data = fs::read(path)?;
    eprintln!("  Parsing Mach-O ({} bytes)...", data.len());
    let macho = macho::MachOImage::new(&data)?;
    eprintln!("  Parsed: vmbase={:#x}, vmsize={:#x}, rebase={}",
              macho.vmbase, macho.vmsize, macho.rebase.len());
    eprintln!("  Mapping image...");
    let map = macho.map_image(&data)?;
    eprintln!("  Mapped at {:#x}", map.data() as usize);
    let image = unsafe { &mut *std::ptr::slice_from_raw_parts_mut(map.data(), map.len()) };
    eprintln!("  Resolving imports...");
    macho.resolve_imports(image, stubs::resolve);
    eprintln!("  Resolving exports...");
    macho.resolve_exports(image, newer::install_hook);
    eprintln!("  Fixing up TLV...");
    macho.fixup_tlv(image);
    eprintln!("  Running mod_init...");
    crate::diag::set_phase(crate::diag::Phase::ModInit);
    macho.run_mod_init(image);

    eprintln!("  Running entry point...");
    crate::diag::set_phase(crate::diag::Phase::EntryPoint);
    unsafe {
        let run_until_alert_addr = stubs::resolve("run_until_alert");
        let run_until_alert: extern "C" fn(func: extern "C" fn()) =
            std::mem::transmute(run_until_alert_addr);
        let entry = macho.get_entry(image);
        run_until_alert(entry);
    }
    eprintln!("  Done.");
    Ok(map)
}

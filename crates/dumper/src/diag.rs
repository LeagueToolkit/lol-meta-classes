//! Crash diagnostics for the dumper.
//!
//! The dumper maps a Mach-O image itself and then *calls into it* - static
//! initializers, the entry point, and every metaclass constructor. When that
//! faults, the default output is a bare `signal: 11 (SIGSEGV)` with nothing to
//! act on. This module turns such a crash into an actionable report:
//!
//! * a `SA_SIGINFO` fault handler that prints the faulting address, and
//! * the metaclass currently being processed, so a fault inside a constructor
//!   names the offending class, and
//! * optional "poisoning" of unresolved imports, so a call through a missing
//!   symbol reports *which* symbol instead of jumping to garbage.
//!
//! ## Poisoning
//!
//! [`MachOImage::resolve_imports`](crate::loader::macho) leaves unresolved
//! import slots holding whatever the bind encoding left there. Reading such a
//! slot yields garbage; calling through it jumps somewhere arbitrary. When
//! `DUMPER_POISON_IMPORTS=1` is set, each unresolved slot is instead filled
//! with a unique unmapped sentinel derived from its index, so the faulting
//! address decodes straight back to a symbol name.
//!
//! Poisoning is opt-in because it is not behaviour-preserving. Plenty of the
//! ~208 unresolved symbols are data (`_NSZeroRect`, `_kSec*`) that shipped code
//! reads without ever dereferencing meaningfully; leaving them as garbage is
//! survivable, whereas a poisoned pointer faults the moment it is dereferenced.
//! Turning it on unconditionally could break versions that currently dump fine.

use std::ffi::c_void;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::OnceLock;

/// Base of the poison window. Canonical (below 2^47, so a fault is a normal
/// page fault rather than a #GP with a useless `si_addr`) and far above the
/// mapped image, which lives at 0x100000000 + vmsize.
const POISON_BASE: usize = 0x0000_6BAD_0000_0000;

/// One page per import. Generous on purpose: a dereference of `slot + small
/// offset` - reading a field off a missing Objective-C class pointer, say -
/// still decodes to the right symbol instead of aliasing onto its neighbour.
const POISON_STRIDE: usize = 0x1000;

/// Names of unresolved imports, indexed by poison slot. Published once, after
/// import resolution finishes and before any game code runs.
static POISON_NAMES: OnceLock<Vec<String>> = OnceLock::new();

/// Hash of the metaclass currently being processed, or 0 outside the walk.
static CURRENT_CLASS: AtomicU32 = AtomicU32::new(0);

/// Where the target image is mapped, so a faulting instruction pointer can be
/// reported as an image-relative address.
static IMAGE_BASE: AtomicUsize = AtomicUsize::new(0);
static IMAGE_LEN: AtomicUsize = AtomicUsize::new(0);

/// What the dumper is currently doing, as a [`Phase`] discriminant.
static CURRENT_PHASE: AtomicUsize = AtomicUsize::new(Phase::Startup as usize);

/// Coarse progress marker, reported on crash to localise the fault.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Phase {
    Startup = 0,
    ModInit = 1,
    EntryPoint = 2,
    /// Reading a class's static metadata - no game code runs here.
    ClassMetadata = 3,
    /// Calling the class constructor. Runs shipped game code.
    ClassConstructor = 4,
    /// Walking a constructed instance's properties. Runs shipped game code.
    ClassDefaults = 5,
    /// Calling the class destructor. Runs shipped game code.
    ClassDestructor = 6,
    Writing = 7,
}

impl Phase {
    fn name(self) -> &'static [u8] {
        match self {
            Phase::Startup => b"startup",
            Phase::ModInit => b"running mod_init",
            Phase::EntryPoint => b"running entry point",
            Phase::ClassMetadata => b"reading class metadata",
            Phase::ClassConstructor => b"calling class constructor",
            Phase::ClassDefaults => b"dumping class default values",
            Phase::ClassDestructor => b"calling class destructor",
            Phase::Writing => b"writing output",
        }
    }

    fn from_usize(v: usize) -> Phase {
        match v {
            1 => Phase::ModInit,
            2 => Phase::EntryPoint,
            3 => Phase::ClassMetadata,
            4 => Phase::ClassConstructor,
            5 => Phase::ClassDefaults,
            6 => Phase::ClassDestructor,
            7 => Phase::Writing,
            _ => Phase::Startup,
        }
    }
}

/// Whether import poisoning was requested via `DUMPER_POISON_IMPORTS=1`.
pub fn poison_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        matches!(
            std::env::var("DUMPER_POISON_IMPORTS").as_deref(),
            Ok("1") | Ok("true")
        )
    })
}

/// Sentinel address for the `index`-th unresolved import.
pub fn poison_for(index: usize) -> usize {
    POISON_BASE + index * POISON_STRIDE
}

/// Publish the unresolved-import name table. Call once, after resolution.
pub fn set_poison_names(names: Vec<String>) {
    let _ = POISON_NAMES.set(names);
}

/// Map a faulting address back to a poison slot index, if it lands in the
/// window covered by `count` poisoned imports.
fn poison_index(addr: usize, count: usize) -> Option<usize> {
    if addr < POISON_BASE {
        return None;
    }
    let index = (addr - POISON_BASE) / POISON_STRIDE;
    (index < count).then_some(index)
}

/// Decode a faulting address back to an unresolved import name, if it lands in
/// the poison window.
fn poison_lookup(addr: usize) -> Option<(usize, Option<&'static str>)> {
    let names = POISON_NAMES.get()?;
    let index = poison_index(addr, names.len())?;
    Some((index, names.get(index).map(|s| s.as_str())))
}

pub fn set_phase(phase: Phase) {
    CURRENT_PHASE.store(phase as usize, Ordering::Relaxed);
}

pub fn set_current_class(hash: u32) {
    CURRENT_CLASS.store(hash, Ordering::Relaxed);
}

/// Record where the target image is mapped.
pub fn set_image_range(base: usize, len: usize) {
    IMAGE_BASE.store(base, Ordering::Relaxed);
    IMAGE_LEN.store(len, Ordering::Relaxed);
}

/// Faulting instruction pointer from the signal context.
///
/// Worth more than `si_addr` for locating the crash: `si_addr` is the address
/// that was touched, whereas this is the instruction that touched it. The image
/// maps at its own `vmbase`, so the image-relative form printed alongside it
/// lines up 1:1 with a disassembler's addresses.
#[cfg(target_arch = "x86_64")]
unsafe fn fault_rip(ctx: *mut c_void) -> Option<usize> {
    if ctx.is_null() {
        return None;
    }
    let uc = ctx as *const libc::ucontext_t;
    Some((*uc).uc_mcontext.gregs[libc::REG_RIP as usize] as usize)
}

#[cfg(not(target_arch = "x86_64"))]
unsafe fn fault_rip(_ctx: *mut c_void) -> Option<usize> {
    None
}

// ---------------------------------------------------------------------------
// Signal handling
//
// Everything below runs in a fault handler, so it sticks to `write(2)` and
// stack buffers. No allocation, no `eprintln!`: the fault may well have
// happened inside malloc or stdio, and re-entering either would deadlock
// instead of printing the one message we actually need.
// ---------------------------------------------------------------------------

fn w(s: &[u8]) {
    unsafe {
        libc::write(2, s.as_ptr() as *const c_void, s.len());
    }
}

fn w_hex(mut value: usize) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut buf = [0u8; 18];
    buf[0] = b'0';
    buf[1] = b'x';
    for i in 0..16 {
        buf[17 - i] = HEX[(value & 0xf) as usize];
        value >>= 4;
    }
    w(&buf);
}

fn w_dec(value: usize) {
    let mut buf = [0u8; 20];
    let mut len = 0;
    let mut v = value;
    loop {
        buf[buf.len() - 1 - len] = b'0' + (v % 10) as u8;
        len += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    w(&buf[buf.len() - len..]);
}

extern "C" fn fault_handler(sig: libc::c_int, info: *mut libc::siginfo_t, ctx: *mut c_void) {
    let addr = unsafe {
        if info.is_null() {
            0
        } else {
            (*info).si_addr() as usize
        }
    };

    w(b"\n!!! FAULT: signal ");
    w_dec(sig as usize);
    w(b" accessing ");
    w_hex(addr);

    if let Some(rip) = unsafe { fault_rip(ctx) } {
        w(b"\n    rip:   ");
        w_hex(rip);
        let base = IMAGE_BASE.load(Ordering::Relaxed);
        let len = IMAGE_LEN.load(Ordering::Relaxed);
        if base != 0 && rip >= base && rip < base + len {
            w(b" (image+");
            w_hex(rip - base);
            w(b")");
        } else if base != 0 {
            w(b" (outside the mapped image)");
        }
    }

    w(b"\n    phase: ");
    w(Phase::from_usize(CURRENT_PHASE.load(Ordering::Relaxed)).name());

    let class = CURRENT_CLASS.load(Ordering::Relaxed);
    if class != 0 {
        w(b"\n    class: ");
        w_hex(class as usize);
    }

    match poison_lookup(addr) {
        Some((index, Some(name))) => {
            w(b"\n    cause: UNRESOLVED IMPORT #");
            w_dec(index);
            w(b" -> ");
            w(name.as_bytes());
            w(b"\n    fix:   add a stub for this symbol in loader/stubs.rs\n");
        }
        Some((index, None)) => {
            w(b"\n    cause: unresolved import #");
            w_dec(index);
            w(b" (name unavailable)\n");
        }
        None => {
            if poison_enabled() {
                w(b"\n    cause: not an unresolved import - genuine bad access\n");
            } else {
                w(b"\n    note:  re-run with DUMPER_POISON_IMPORTS=1 to test whether\n           \
                    this is a call through a missing import\n");
            }
        }
    }

    // _exit, not exit: atexit handlers would run shipped game code again.
    unsafe { libc::_exit(139) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poison_round_trips() {
        for i in [0usize, 1, 42, 207, 2681] {
            assert_eq!(poison_index(poison_for(i), 2682), Some(i));
        }
    }

    #[test]
    fn poison_tolerates_offsets_within_a_slot() {
        // A missing Objective-C class pointer is typically dereferenced at a
        // small offset rather than address zero; that must still decode.
        let base = poison_for(17);
        for off in [0usize, 8, 0x40, 0xfff] {
            assert_eq!(poison_index(base + off, 2682), Some(17));
        }
        // One byte further is the next slot, not a misattribution to 17.
        assert_eq!(poison_index(base + 0x1000, 2682), Some(18));
    }

    #[test]
    fn addresses_outside_the_window_are_rejected() {
        assert_eq!(poison_index(0, 2682), None);
        assert_eq!(poison_index(0x100000000, 2682), None);
        assert_eq!(poison_index(POISON_BASE - 1, 2682), None);
        // Past the number of poisoned imports.
        assert_eq!(poison_index(poison_for(2682), 2682), None);
    }

    #[test]
    fn poison_window_clears_the_mapped_image() {
        // The image maps at 0x100000000 with a vmsize under 0x103000000; the
        // poison window must not overlap it, or a genuine bad access inside the
        // image would be misreported as a missing import.
        assert!(POISON_BASE > 0x100000000 + 0x103000000);
        // ...and must stay canonical, so faults arrive with a usable si_addr.
        assert!(POISON_BASE + 2682 * POISON_STRIDE < 1usize << 47);
    }
}

/// Install fault handlers for SIGSEGV, SIGBUS and SIGILL.
pub fn install() {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = fault_handler as *const () as usize;
        action.sa_flags = libc::SA_SIGINFO | libc::SA_RESETHAND;
        libc::sigemptyset(&mut action.sa_mask);
        for sig in [libc::SIGSEGV, libc::SIGBUS, libc::SIGILL] {
            libc::sigaction(sig, &action, std::ptr::null_mut());
        }
    }
}

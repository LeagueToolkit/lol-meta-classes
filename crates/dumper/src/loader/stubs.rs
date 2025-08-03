extern crate libc;
use libc::RTLD_NOW;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::{self};

lazy_static! {
    static ref DLLS: Box<[usize]> = {
        [
            "libc++.so.1\0",
            "libc++abi.so.1\0",
            "libm.so.6\0",
            "libc++.so\0",
            "libc++abi.so\0",
            "libm.so\0",
        ]
        .iter()
        .map(|s| {
            let result = unsafe { libc::dlopen(s.as_ptr() as _, RTLD_NOW) as usize };
            result
        })
        .filter(|x| *x != 0)
        .collect()
    };
}

unsafe extern "C" fn stub_atexit(_lpfunc: Option<unsafe extern "C" fn()>) -> c_int {
    // NOTE: do nothing, don't care about exits
    0
}

unsafe extern "C" fn stub_cxa_atexit(
    _lpfunc: Option<unsafe extern "C" fn(*mut c_void)>,
    _obj: *mut c_void,
    _lpdso_handle: *mut c_void,
) -> c_int {
    // NOTE: do nothing, don't care about exits
    0
}

extern "C" {
    fn dlysm_stub_darwin(symbol: *const c_char) -> *const c_void;
}

unsafe extern "C" fn stub_dlsym(_handle: *mut c_void, symbol: *const c_char) -> *const c_void {
    if symbol == ptr::null() {
        return ptr::null();
    }
    resolve_impl(CStr::from_ptr(symbol))
}

unsafe extern "C" fn stub_dladdr(_c: *mut c_void, _info: *mut c_void) -> c_int {
    // sentry/crashpad bs, allways fail, dont care
    0
}

unsafe extern "C" fn stub_dlclose(_handle: *mut c_void) -> c_int {
    0 // Always success
}

unsafe extern "C" fn stub_dlerror() -> *const c_char {
    b"Not implemented!\0".as_ptr() as *const c_char
}

unsafe extern "C" fn stub_dlopen(path: *const c_char, _mode: c_int) -> *mut c_void {
    if path.is_null() {
        return ptr::null_mut();
    }
    match std::ffi::CStr::from_ptr(path).to_str() {
        Ok(s) if s.contains("libc") => "libc\0".as_ptr() as *mut c_void,
        _ => ptr::null_mut(),
    }
}

unsafe fn resolve_impl(name: &CStr) -> *const c_void {
    match name.to_bytes() {
        b"__stderrp" => libc::dlsym(ptr::null_mut(), b"stderr\0".as_ptr() as *const _),
        b"__stdinp" => libc::dlsym(ptr::null_mut(), b"stdin\0".as_ptr() as *const _),
        b"__stdoutp" => libc::dlsym(ptr::null_mut(), b"stdout\0".as_ptr() as *const _),

        b"atexit" => stub_atexit as *const () as *const c_void,
        b"__cxa_atexit" => stub_cxa_atexit as *const () as *const c_void,

        b"dladdr" => stub_dladdr as *const () as *const c_void,
        b"dlclose" => stub_dlclose as *const () as *const c_void,
        b"dlerror" => stub_dlerror as *const () as *const c_void,
        b"dlopen" => stub_dlopen as *const () as *const c_void,
        b"dlsym" => stub_dlsym as *const () as *const c_void,

        _ => {
            // These would need more complex to handle properly, don't really care about them for just init.
            // Just allways resolve them to null, that way diagnosing the crash gets much easier.
            const BLOCKLIST: &[&str] = &[
                // block pthread
                "pthread_",
                // block dyld stubs
                "tlv_bootstrap",
                "dyld_",
                // filter unimplemented C++ thread stubs
                "_ZNSt3__15mutex",
                "_ZNSt3__115recursive_mutex",
                "_ZNSt3__121recursive_timed_mutex",
                "_ZNSt3__16thread",
                "_ZNSt3__118condition_variable",
                // block fork
                "fork",
                "exec",
                "posix_spawn",
            ];
            // Make sure to process $darwin$ overrides BEFORE blacklist, this allows 
            let darwin = dlysm_stub_darwin(name.as_ptr());
            if darwin != ptr::null() {
                return darwin;
            }
            for blocked in BLOCKLIST {
                if name.to_bytes().starts_with(blocked.as_bytes()) {
                    return ptr::null();
                }
            }
            for &dll in DLLS.iter() {
                let result = libc::dlsym(dll as _, name.as_ptr());
                if result != ptr::null_mut() {
                    return result as _;
                }
            }
            libc::dlsym(ptr::null_mut(), name.as_ptr())
        }
    }
}

pub fn resolve(name: &str) -> usize {
    lazy_static::initialize(&DLLS);

    let name = if name.starts_with("_") {
        &name[1..]
    } else {
        name
    };
    let cname = CString::new(name).unwrap();
    let result = unsafe { resolve_impl(&cname) as _ };
    result
}

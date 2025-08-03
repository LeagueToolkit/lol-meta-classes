use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use libc::{malloc, free, posix_memalign};
use std::ptr;

#[repr(C)]
pub struct NoThrow;

// void* operator new    (size_t size);
static mut HOOK_ZNWM: Option<unsafe extern "C" fn(size: usize) -> *mut c_void> = None;

// void* operator new    (size_t size, std::nothrow_t const& nt);
static mut HOOK_ZNWM_RKST9NOTHROW_T: Option<unsafe extern "C" fn(size: usize, nt: &NoThrow) -> *mut c_void> = None;

// void* operator new    (size_t size, std::align_val_t align);
static mut HOOK_ZNWM_ST11ALIGN_VAL_T: Option<unsafe extern "C" fn(size: usize, align: usize) -> *mut c_void> = None;

// void* operator new    (size_t size, std::align_val_t align, std::nothrow_t const& nt);
static mut HOOK_ZNWM_ST11ALIGN_VAL_T_RKST9NOTHROW_T: Option<unsafe extern "C" fn(size: usize, align: usize, nt: &NoThrow) -> *mut c_void> = None;

// void* operator new[]  (size_t size);
static mut HOOK_ZNAM: Option<unsafe extern "C" fn(size: usize) -> *mut c_void> = None;

// void* operator new[]  (size_t size, std::nothrow_t const& nt);
static mut HOOK_ZNAM_RKST9NOTHROW_T: Option<unsafe extern "C" fn(size: usize, nt: &NoThrow) -> *mut c_void> = None;

// void* operator new[]  (size_t size, std::align_val_t align);
static mut HOOK_ZNAM_ST11ALIGN_VAL_T: Option<unsafe extern "C" fn(size: usize, align: usize) -> *mut c_void> = None;

// void* operator new[]  (size_t size, std::align_val_t align, std::nothrow_t const& nt);
static mut HOOK_ZNAM_ST11ALIGN_VAL_T_RKST9NOTHROW_T: Option<unsafe extern "C" fn(size: usize, align: usize, nt: &NoThrow) -> *mut c_void> = None;

// void operator delete  (void * ptr);
static mut HOOK_ZDL_PV: Option<unsafe extern "C" fn(ptr: *mut c_void)> = None;

// void operator delete  (void * ptr, std::nothrow_t const& nt);
static mut HOOK_ZDL_PV_RKST9NOTHROW_T: Option<unsafe extern "C" fn(ptr: *mut c_void, nt: &NoThrow)> = None;

// void operator delete  (void * ptr, size_t size);
static mut HOOK_ZDL_PVM: Option<unsafe extern "C" fn(ptr: *mut c_void, size: usize)> = None;

// void operator delete  (void * ptr, size_t size, std::align_val_t align);
static mut HOOK_ZDL_PVM_ST11ALIGN_VAL_T: Option<unsafe extern "C" fn(ptr: *mut c_void, size: usize, align: usize)> = None;

// void operator delete  (void * ptr, std::align_val_t align);
static mut HOOK_ZDL_PV_ST11ALIGN_VAL_T: Option<unsafe extern "C" fn(ptr: *mut c_void, align: usize)> = None;

// void operator delete  (void * ptr, std::align_val_t align, std::nothrow_t const& nt);
static mut HOOK_ZDL_PV_ST11ALIGN_VAL_T_RKST9NOTHROW_T: Option<unsafe extern "C" fn(ptr: *mut c_void, align: usize, nt: &NoThrow)> = None;

// void operator delete[](void * ptr);
static mut HOOK_ZDA_PV: Option<unsafe extern "C" fn(ptr: *mut c_void)> = None;

// void operator delete[](void * ptr, std::nothrow_t const& nt);
static mut HOOK_ZDA_PV_RKST9NOTHROW_T: Option<unsafe extern "C" fn(ptr: *mut c_void, nt: &NoThrow)> = None;

// void operator delete[](void * ptr, size_t size);
static mut HOOK_ZDA_PVM: Option<unsafe extern "C" fn(ptr: *mut c_void, size: usize)> = None;

// void operator delete[](void * ptr, size_t size, std::align_val_t align);
static mut HOOK_ZDA_PVM_ST11ALIGN_VAL_T: Option<unsafe extern "C" fn(ptr: *mut c_void, size: usize, align: usize)> = None;

// void operator delete[](void * ptr, std::align_val_t align);
static mut HOOK_ZDA_PV_ST11ALIGN_VAL_T: Option<unsafe extern "C" fn(ptr: *mut c_void, align: usize)> = None;

// void operator delete[](void * ptr, std::align_val_t align, std::nothrow_t const& nt);
static mut HOOK_ZDA_PV_ST11ALIGN_VAL_T_RKST9NOTHROW_T: Option<unsafe extern "C" fn(ptr: *mut c_void, align: usize, nt: &NoThrow)> = None;

//#[linkage = "weak"]
#[no_mangle]
pub extern "C" fn _Znwm(size: usize) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNWM {
            hook(size)
        } else {
            malloc(size) as *mut c_void
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZnwmRKSt9nothrow_t(size: usize, nt: &NoThrow) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNWM_RKST9NOTHROW_T {
            hook(size, nt)
        } else {
            _Znwm(size)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZnwmSt11align_val_t(size: usize, align: usize) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNWM_ST11ALIGN_VAL_T {
            hook(size, align)
        } else {
            let mut ptr: *mut c_void = ptr::null_mut();
            posix_memalign(&mut ptr, align, size);
            ptr
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZnwmSt11align_val_tRKSt9nothrow_t(size: usize, align: usize, nt: &NoThrow) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNWM_ST11ALIGN_VAL_T_RKST9NOTHROW_T {
            hook(size, align, nt)
        } else {
            _ZnwmSt11align_val_t(size, align)
        }
    }
}

#[no_mangle]
pub extern "C" fn _Znam(size: usize) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNAM {
            hook(size)
        } else {
            _Znwm(size)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZnamRKSt9nothrow_t(size: usize, nt: &NoThrow) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNAM_RKST9NOTHROW_T {
            hook(size, nt)
        } else {
            _Znam(size)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZnamSt11align_val_t(size: usize, align: usize) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNAM_ST11ALIGN_VAL_T {
            hook(size, align)
        } else {
            _ZnwmSt11align_val_t(size, align)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZnamSt11align_val_tRKSt9nothrow_t(size: usize, align: usize, nt: &NoThrow) -> *mut c_void {
    unsafe {
        if let Some(hook) = HOOK_ZNAM_ST11ALIGN_VAL_T_RKST9NOTHROW_T {
            hook(size, align, nt)
        } else {
            _ZnamSt11align_val_t(size, align)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdlPv(ptr: *mut c_void) {
    unsafe {
        if let Some(hook) = HOOK_ZDL_PV {
            hook(ptr)
        } else {
            free(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdlPvRKSt9nothrow_t(ptr: *mut c_void, nt: &NoThrow) {
    unsafe {
        if let Some(hook) = HOOK_ZDL_PV_RKST9NOTHROW_T {
            hook(ptr, nt)
        } else {
            _ZdlPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdlPvm(ptr: *mut c_void, size: usize) {
    unsafe {
        if let Some(hook) = HOOK_ZDL_PVM {
            hook(ptr, size)
        } else {
            _ZdlPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdlPvmSt11align_val_t(ptr: *mut c_void, size: usize, align: usize) {
    unsafe {
        if let Some(hook) = HOOK_ZDL_PVM_ST11ALIGN_VAL_T {
            hook(ptr, size, align)
        } else {
            _ZdlPvm(ptr, size)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdlPvSt11align_val_t(ptr: *mut c_void, align: usize) {
    unsafe {
        if let Some(hook) = HOOK_ZDL_PV_ST11ALIGN_VAL_T {
            hook(ptr, align)
        } else {
            _ZdlPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdlPvSt11align_val_tRKSt9nothrow_t(ptr: *mut c_void, align: usize, nt: &NoThrow) {
    unsafe {
        if let Some(hook) = HOOK_ZDL_PV_ST11ALIGN_VAL_T_RKST9NOTHROW_T {
            hook(ptr, align, nt)
        } else {
            _ZdlPvSt11align_val_t(ptr, align)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdaPv(ptr: *mut c_void) {
    unsafe {
        if let Some(hook) = HOOK_ZDA_PV {
            hook(ptr)
        } else {
            _ZdlPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdaPvRKSt9nothrow_t(ptr: *mut c_void, nt: &NoThrow) {
    unsafe {
        if let Some(hook) = HOOK_ZDA_PV_RKST9NOTHROW_T {
            hook(ptr, nt)
        } else {
            _ZdaPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdaPvm(ptr: *mut c_void, size: usize) {
    unsafe {
        if let Some(hook) = HOOK_ZDA_PVM {
            hook(ptr, size)
        } else {
            _ZdaPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdaPvmSt11align_val_t(ptr: *mut c_void, size: usize, align: usize) {
    unsafe {
        if let Some(hook) = HOOK_ZDA_PVM_ST11ALIGN_VAL_T {
            hook(ptr, size, align)
        } else {
            _ZdaPvm(ptr, size)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdaPvSt11align_val_t(ptr: *mut c_void, align: usize) {
    unsafe {
        if let Some(hook) = HOOK_ZDA_PV_ST11ALIGN_VAL_T {
            hook(ptr, align)
        } else {
            _ZdaPv(ptr)
        }
    }
}

#[no_mangle]
pub extern "C" fn _ZdaPvSt11align_val_tRKSt9nothrow_t(ptr: *mut c_void, align: usize, nt: &NoThrow) {
    unsafe {
        if let Some(hook) = HOOK_ZDA_PV_ST11ALIGN_VAL_T_RKST9NOTHROW_T {
            hook(ptr, align, nt)
        } else {
            _ZdaPvSt11align_val_t(ptr, align)
        }
    }
}

pub fn install_hook(name: &str, ptr: usize) -> bool {
    macro_rules! check_hook {
        ($hook:ident, $name:expr) => {
            if name == $name {
                $hook = Some(std::mem::transmute(ptr));
                return true;
            }
        };
    }

    unsafe {
        check_hook!(HOOK_ZNWM, "__Znwm");
        check_hook!(HOOK_ZNWM_RKST9NOTHROW_T, "__ZnwmRKSt9nothrow_t");
        check_hook!(HOOK_ZNWM_ST11ALIGN_VAL_T, "__ZnwmSt11align_val_t");
        check_hook!(HOOK_ZNWM_ST11ALIGN_VAL_T_RKST9NOTHROW_T, "__ZnwmSt11align_val_tRKSt9nothrow_t");
        check_hook!(HOOK_ZNAM, "__Znam");
        check_hook!(HOOK_ZNAM_RKST9NOTHROW_T, "__ZnamRKSt9nothrow_t");
        check_hook!(HOOK_ZNAM_ST11ALIGN_VAL_T, "__ZnamSt11align_val_t");
        check_hook!(HOOK_ZNAM_ST11ALIGN_VAL_T_RKST9NOTHROW_T, "__ZnamSt11align_val_tRKSt9nothrow_t");
        check_hook!(HOOK_ZDL_PV, "__ZdlPv");
        check_hook!(HOOK_ZDL_PV_RKST9NOTHROW_T, "__ZdlPvRKSt9nothrow_t");
        check_hook!(HOOK_ZDL_PVM, "__ZdlPvm");
        check_hook!(HOOK_ZDL_PVM_ST11ALIGN_VAL_T, "__ZdlPvmSt11align_val_t");
        check_hook!(HOOK_ZDL_PV_ST11ALIGN_VAL_T, "__ZdlPvSt11align_val_t");
        check_hook!(HOOK_ZDL_PV_ST11ALIGN_VAL_T_RKST9NOTHROW_T, "__ZdlPvSt11align_val_tRKSt9nothrow_t");
        check_hook!(HOOK_ZDA_PV, "__ZdaPv");
        check_hook!(HOOK_ZDA_PV_RKST9NOTHROW_T, "__ZdaPvRKSt9nothrow_t");
        check_hook!(HOOK_ZDA_PVM, "__ZdaPvm");
        check_hook!(HOOK_ZDA_PVM_ST11ALIGN_VAL_T, "__ZdaPvmSt11align_val_t");
        check_hook!(HOOK_ZDA_PV_ST11ALIGN_VAL_T, "__ZdaPvSt11align_val_t");
        check_hook!(HOOK_ZDA_PV_ST11ALIGN_VAL_T_RKST9NOTHROW_T, "__ZdaPvSt11align_val_tRKSt9nothrow_t");
    }

    // f*ck you lto
    std::hint::black_box(_Znwm);
    std::hint::black_box(_ZnwmRKSt9nothrow_t);
    std::hint::black_box(_ZnwmSt11align_val_t);
    std::hint::black_box(_ZnwmSt11align_val_tRKSt9nothrow_t);
    std::hint::black_box(_Znam);
    std::hint::black_box(_ZnamRKSt9nothrow_t);
    std::hint::black_box(_ZnamSt11align_val_t);
    std::hint::black_box(_ZnamSt11align_val_tRKSt9nothrow_t);
    std::hint::black_box(_ZdlPv);
    std::hint::black_box(_ZdlPvRKSt9nothrow_t);
    std::hint::black_box(_ZdlPvm);
    std::hint::black_box(_ZdlPvmSt11align_val_t);
    std::hint::black_box(_ZdlPvSt11align_val_t);
    std::hint::black_box(_ZdlPvSt11align_val_tRKSt9nothrow_t);
    std::hint::black_box(_ZdaPv);
    std::hint::black_box(_ZdaPvRKSt9nothrow_t);
    std::hint::black_box(_ZdaPvm);
    std::hint::black_box(_ZdaPvmSt11align_val_t);
    std::hint::black_box(_ZdaPvSt11align_val_t);
    std::hint::black_box(_ZdaPvSt11align_val_tRKSt9nothrow_t);

    false
}
//! Convenience functions for interacting with ffi calls to vulkan.
//!
//! Here be dragons. Read the comments on functions to use them correctly!

use std::{ffi::CString, os::raw::c_char};

/// Build a vector of pointers to c-style strings from a vector of rust strings.
///
/// Unsafe because the returned vector of pointers is only valid while the
/// cstrings are alive.
pub unsafe fn to_os_ptrs(
    strings: &Vec<String>,
) -> (Vec<CString>, Vec<*const c_char>) {
    let cstrings = strings
        .iter()
        .cloned()
        .map(|str| CString::new(str).unwrap())
        .collect::<Vec<CString>>();
    let ptrs = cstrings
        .iter()
        .map(|cstr| cstr.as_ptr())
        .collect::<Vec<*const c_char>>();
    (cstrings, ptrs)
}

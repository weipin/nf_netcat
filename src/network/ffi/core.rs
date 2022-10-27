#![allow(non_camel_case_types)]

use std::ffi::{c_int, c_void};

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub(crate) fn nw_release(object: *mut c_void);
    pub(crate) fn nw_retain(object: *mut c_void) -> *mut c_void;
    pub(crate) fn nw_error_get_error_code(error: nw_error_t) -> c_int;
}

pub(crate) enum nw_object {}
pub(crate) type nw_object_t = *mut nw_object;
pub(crate) type nw_error_t = *mut nw_object;

use dispatch::ffi::dispatch_queue_t;
use std::ffi::c_void;

#[link(name = "System", kind = "dylib")]
extern "C" {
    pub(crate) fn dispatch_data_create(
        buffer: *const c_void,
        size: usize,
        queue: dispatch_queue_t,
        destructor: *const c_void,
    ) -> dispatch_data_t;

    pub(crate) fn dispatch_data_create_map(
        data: dispatch_data_t,
        buffer_ptr: *mut *mut c_void,
        size_ptr: *mut usize,
    ) -> dispatch_data_t;
}

pub(crate) enum OS_dispatch_data {}
pub(crate) type dispatch_data_t = *mut OS_dispatch_data;

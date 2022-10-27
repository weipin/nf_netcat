use super::core::{nw_error_t, nw_object_t};
use super::dispatch::dispatch_data_t;
use super::endpoint::nw_endpoint_t;
use block::Block;
use dispatch::ffi::dispatch_queue_t;
use std::ffi::c_void;
use super::parameters::nw_parameters_t;

#[link(name = "Network", kind = "framework")]
extern "C" {
    static _nw_content_context_default_message: *mut c_void;

    pub(crate) fn nw_connection_create(endpoint: nw_endpoint_t, parameters: nw_parameters_t) -> nw_connection_t ;
    pub(crate) fn nw_connection_cancel(connection: nw_connection_t);

    pub(crate) fn nw_connection_set_queue(connection: nw_connection_t, queue: dispatch_queue_t);
    pub(crate) fn nw_connection_set_state_changed_handler(
        connection: nw_connection_t,
        handler: &Block<(nw_connection_state_t, nw_error_t), ()>,
    );

    pub(crate) fn nw_connection_copy_endpoint(connection: nw_connection_t) -> nw_endpoint_t;

    pub(crate) fn nw_connection_start(connection: nw_connection_t);

    pub(crate) fn nw_connection_send(
        connection: nw_connection_t,
        content: dispatch_data_t,
        context: nw_content_context_t,
        is_complete: bool,
        completion: &Block<(nw_error_t,), ()>,
    );

    pub(crate) fn nw_connection_receive_message(
        connection: nw_connection_t,
        completion: &Block<(dispatch_data_t, nw_content_context_t, bool, nw_error_t), ()>,
    );
}

#[allow(non_camel_case_types)]
pub(crate) type nw_connection_t = nw_object_t;
pub(crate) type nw_content_context_t = *const c_void;

#[allow(non_camel_case_types)]
#[repr(u32)]
pub(crate) enum nw_connection_state_t {
    nw_connection_state_invalid = 0,
    nw_connection_state_waiting = 1,
    nw_connection_state_preparing = 2,
    nw_connection_state_ready = 3,
    nw_connection_state_failed = 4,
    nw_connection_state_cancelled = 5,
}

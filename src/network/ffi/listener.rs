use super::connection::nw_connection_t;
use super::core::nw_error_t;
use super::core::nw_object_t;
use super::parameters::nw_parameters_t;
use block::Block;
use dispatch::ffi::dispatch_queue_t;

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub(crate) fn nw_listener_create(parameters: nw_parameters_t) -> nw_listener_t;
    pub(crate) fn nw_listener_set_queue(listener: nw_listener_t, queue: dispatch_queue_t);
    pub(crate) fn nw_listener_set_state_changed_handler(
        listener: nw_listener_t,
        handler: &Block<(nw_listener_state_t, nw_error_t), ()>,
    );
    pub(crate) fn nw_listener_get_port(listener: nw_listener_t) -> u16;
    pub(crate) fn nw_listener_set_new_connection_handler(
        listener: nw_listener_t,
        handler: &Block<(nw_connection_t,), ()>,
    );

    pub(crate) fn nw_listener_start(listener: nw_listener_t);
}

#[allow(non_camel_case_types)]
pub(crate) type nw_listener_t = nw_object_t;

#[allow(non_camel_case_types)]
#[repr(u32)]
pub(crate) enum nw_listener_state_t {
    nw_listener_state_invalid = 0,
    nw_listener_state_waiting = 1,
    nw_listener_state_ready = 2,
    nw_listener_state_failed = 3,
    nw_listener_state_cancelled = 4,
}

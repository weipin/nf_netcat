#![allow(non_camel_case_types)]

use super::core::nw_object_t;
use crate::network::ffi::endpoint::nw_endpoint_t;
use std::ffi::c_void;

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub(crate) static _nw_parameters_configure_protocol_disable: *mut c_void;
    pub(crate) static _nw_parameters_configure_protocol_default_configuration: *mut c_void;

    pub(crate) fn nw_parameters_create() -> nw_parameters_t;
    pub(crate) fn nw_parameters_create_secure_udp(
        configure_dtls: *mut c_void,
        configure_udp: *mut c_void,
    ) -> nw_parameters_t;

    pub(crate) fn nw_parameters_copy_default_protocol_stack(
        parameters: nw_parameters_t,
    ) -> nw_protocol_stack_t;
    pub(crate) fn nw_protocol_stack_set_transport_protocol(
        stack: nw_protocol_stack_t,
        protocol: nw_protocol_options_t,
    );
    pub(crate) fn nw_protocol_stack_clear_application_protocols(stack: nw_protocol_stack_t);
    pub(crate) fn nw_protocol_stack_copy_internet_protocol(
        stack: nw_protocol_stack_t,
    ) -> nw_protocol_options_t;

    pub(crate) fn nw_parameters_set_local_endpoint(
        parameters: nw_parameters_t,
        local_endpoint: nw_endpoint_t,
    );
}

pub(crate) type nw_parameters_t = nw_object_t;
pub(crate) type nw_protocol_stack_t = nw_object_t;
pub(crate) type nw_protocol_options_t = nw_object_t;

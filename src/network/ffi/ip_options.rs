use super::parameters::nw_protocol_options_t;

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub(crate) fn nw_ip_options_set_version(
        options: nw_protocol_options_t,
        version: nw_ip_version_t,
    );
}

#[allow(non_camel_case_types)]
#[repr(u32)]
pub(crate) enum nw_ip_version_t {
    nw_ip_version_any = 0,
    nw_ip_version_4 = 4,
    nw_ip_version_6 = 6,
}

use super::core::nw_object_t;
use std::ffi::c_char;

#[link(name = "Network", kind = "framework")]
extern "C" {
    pub(crate) fn nw_endpoint_create_host(
        hostname: *const c_char,
        port: *const c_char,
    ) -> nw_endpoint_t;
    pub(crate) fn nw_endpoint_get_hostname(endpoint: nw_endpoint_t) -> *const c_char;
    pub(crate) fn nw_endpoint_get_port(endpoint: nw_endpoint_t) -> u16;
}

#[allow(non_camel_case_types)]
pub(crate) type nw_endpoint_t = nw_object_t;

#[cfg(test)]
mod tests {
    use crate::network::ffi::endpoint::{nw_endpoint_create_host, nw_endpoint_get_hostname};
    use crate::os::apple::network::ffi::core::nw_release;
    use crate::os::apple::network::ffi::endpoint::{
        nw_endpoint_create_host, nw_endpoint_get_hostname,
    };
    use std::ffi::{c_void, CStr, CString};

    #[test]
    fn test_nw_endpoint() {
        // TODO: release CString memory
        // # Safety
        // The ptr should be a valid pointer to the string allocated by rust
        /*
        #[no_mangle]
        pub unsafe extern fn free_string(ptr: *const c_char) {
            // Take the ownership back to rust and drop the owner
            let _ = CString::from_raw(ptr as *mut _);
        }
        */
        let hostname = CString::new("127.0.0.1").unwrap();
        let port = CString::new("1234").unwrap();
        let endpoint = unsafe { nw_endpoint_create_host(hostname.as_ptr(), port.as_ptr()) };
        let hostname_cstr = unsafe {
            let hostname_ptr = nw_endpoint_get_hostname(endpoint);
            CStr::from_ptr(hostname_ptr)
        };
        let hostname = hostname_cstr.to_str().unwrap();
        assert_eq!(hostname, "127.0.0.1");

        unsafe {
            nw_release(endpoint as *mut c_void);
        }
    }
}

//! Netcat
//!
//! # Examples
//!
//! # server
//! cargo run -- -l 12345
//!
//! # client
//! cargo run -- localhost 12345

mod network;

use crate::network::ffi::connection::{
    nw_connection_cancel, nw_connection_copy_endpoint, nw_connection_create,
    nw_connection_receive_message, nw_connection_send, nw_connection_set_queue,
    nw_connection_set_state_changed_handler, nw_connection_start, nw_connection_state_t,
    nw_connection_t, nw_content_context_t,
};
use crate::network::ffi::core::{nw_error_get_error_code, nw_error_t, nw_release, nw_retain};
use crate::network::ffi::dispatch::{
    dispatch_data_create, dispatch_data_create_map, dispatch_data_t,
};
use crate::network::ffi::endpoint::{
    nw_endpoint_create_host, nw_endpoint_get_hostname, nw_endpoint_get_port, nw_endpoint_t,
};
use crate::network::ffi::ip_options::{nw_ip_options_set_version, nw_ip_version_t};
use crate::network::ffi::listener::{
    nw_listener_create, nw_listener_set_new_connection_handler, nw_listener_set_queue,
    nw_listener_set_state_changed_handler, nw_listener_start, nw_listener_state_t, nw_listener_t,
};
use crate::network::ffi::parameters::{
    _nw_parameters_configure_protocol_default_configuration,
    _nw_parameters_configure_protocol_disable, nw_parameters_copy_default_protocol_stack,
    nw_parameters_create_secure_udp, nw_parameters_set_local_endpoint, nw_parameters_t,
    nw_protocol_stack_copy_internet_protocol,
};
use block::{Block, ConcreteBlock};
use dispatch::ffi::{dispatch_get_main_queue, dispatch_main};
use dispatch::{Queue, QueueAttribute};
use std::ffi::{c_void, CStr, CString};
use std::{io, ptr};

static mut G_CONNECTION: Option<nw_connection_t> = None;

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("Error: {err}");
            print_usage(0);
            std::process::exit(1);
        }
    };

    // Validates options
    if args.server {
        if args.port.is_none() {
            eprintln!("Missing port with option -l");
            print_usage(1);
        }
        if args.local_address.is_some() {
            eprintln!("Cannot use -s and -l");
            print_usage(1);
        }
        if args.local_port.is_some() {
            eprintln!("Cannot use -p and -l");
            print_usage(1);
        }
    } else {
        if args.hostname.is_none() || args.port.is_none() {
            eprintln!("Missing hostname and port");
            print_usage(1);
        }
    }

    if args.server {
        let listener = create_and_start_listener(&args);
        if listener.is_null() {
            eprintln!("Creating listener failed");
            std::process::exit(1);
        }
        unsafe { dispatch_main() };
    } else {
        create_outbound_connection(&args);
        unsafe { dispatch_main() };
    }
}

/// Returns a retained listener on a local port and optional address.
/// Schedules listener on main queue and starts it.
///  -> nw_listener_t
fn create_and_start_listener(args: &AppArgs) -> nw_listener_t {
    let parameters = unsafe {
        nw_parameters_create_secure_udp(
            _nw_parameters_configure_protocol_disable,
            _nw_parameters_configure_protocol_default_configuration,
        )
    };

    match (args.ipv4_only, args.ipv6_only) {
        (true, true) | (false, false) => {}
        (true, false) | (false, true) => {
            let protocol_stack = unsafe { nw_parameters_copy_default_protocol_stack(parameters) };
            let ip_options = unsafe { nw_protocol_stack_copy_internet_protocol(protocol_stack) };
            if args.ipv4_only {
                unsafe { nw_ip_options_set_version(ip_options, nw_ip_version_t::nw_ip_version_4) };
            } else {
                unsafe { nw_ip_options_set_version(ip_options, nw_ip_version_t::nw_ip_version_6) };
            }
            unsafe { nw_release(ip_options as *mut c_void) };
            unsafe { nw_release(protocol_stack as *mut c_void) };
        }
    }

    // Bind to local address and port
    if args.hostname.is_some() || args.port.is_some() {
        let local_address_cstr =
            CString::new(args.hostname.clone().unwrap_or("::".to_string())).unwrap();
        let local_port_cstr = CString::new(args.port.clone().unwrap_or("0".to_string())).unwrap();
        let local_endpoint = unsafe {
            nw_endpoint_create_host(local_address_cstr.as_ptr(), local_port_cstr.as_ptr())
        };
        unsafe { nw_parameters_set_local_endpoint(parameters, local_endpoint) };
        unsafe { nw_release(local_endpoint as *mut c_void) };
    }

    let listener = unsafe { nw_listener_create(parameters) };
    unsafe { nw_release(parameters as *mut c_void) };

    unsafe { nw_listener_set_queue(listener, dispatch_get_main_queue()) };

    unsafe { nw_retain(listener as *mut c_void) }; // Hold a reference until cancelled
    let mut state_changed_handler =
        ConcreteBlock::new(|state: nw_listener_state_t, error: nw_error_t| {
            if !error.is_null() {
                let error_no = unsafe { nw_error_get_error_code(error) };
                eprintln!("listener error: {error_no}");
            }
            match state {
                nw_listener_state_t::nw_listener_state_invalid => {
                    eprintln!("listener invalid state");
                }
                nw_listener_state_t::nw_listener_state_waiting => {
                    // let port = unsafe { nw_listener_get_port(listener) };
                    // eprintln!("Listener on port {port} waiting");
                }
                nw_listener_state_t::nw_listener_state_ready => {
                    // let port = unsafe { nw_listener_get_port(listener) };
                    // eprintln!("Listener on port {port} ready!");
                    println!("Listener ready!");
                }
                nw_listener_state_t::nw_listener_state_failed => {
                    eprintln!("listener failed");
                }
                nw_listener_state_t::nw_listener_state_cancelled => {
                    // Release the primary reference on the listener
                    // that was taken at creation time
                    // unsafe { nw_release(listener as *mut c_void) };
                }
            }
        });
    let block = state_changed_handler.copy();
    unsafe { nw_listener_set_state_changed_handler(listener, &block) };

    let mut connection_handler = ConcreteBlock::new(|connection: nw_connection_t| {
        unsafe {
            if G_CONNECTION.is_some() {
                // We only support one connection at a time, so if we already
                // have one, reject the incoming connection.
                nw_connection_cancel(G_CONNECTION.unwrap());
                println!("Connection rejected!");
            } else {
                // Accept the incoming connection and start sending
                // and receiving on it.
                // println!("Accepting incoming connection!");
                G_CONNECTION = Some(connection);
                nw_retain(connection as *mut c_void);

                start_connection(connection);
                start_send_receive_loop(connection);
            }
        }
    });
    let block = connection_handler.copy();
    unsafe { nw_listener_set_new_connection_handler(listener, &block) };

    unsafe { nw_listener_start(listener) };
    listener
}

fn create_outbound_connection(args: &AppArgs) {
    let parameters = unsafe {
        nw_parameters_create_secure_udp(
            _nw_parameters_configure_protocol_disable,
            _nw_parameters_configure_protocol_default_configuration,
        )
    };

    match (args.ipv4_only, args.ipv6_only) {
        (true, true) | (false, false) => {}
        (true, false) | (false, true) => {
            let protocol_stack = unsafe { nw_parameters_copy_default_protocol_stack(parameters) };
            let ip_options = unsafe { nw_protocol_stack_copy_internet_protocol(protocol_stack) };
            if args.ipv4_only {
                unsafe { nw_ip_options_set_version(ip_options, nw_ip_version_t::nw_ip_version_4) };
            } else {
                unsafe { nw_ip_options_set_version(ip_options, nw_ip_version_t::nw_ip_version_6) };
            }
            unsafe { nw_release(ip_options as *mut c_void) };
            unsafe { nw_release(protocol_stack as *mut c_void) };
        }
    }

    let address_cstr = CString::new(args.hostname.clone().unwrap_or("::".to_string())).unwrap();
    let port_cstr = CString::new(args.port.clone().unwrap_or("0".to_string())).unwrap();
    let endpoint = unsafe { nw_endpoint_create_host(address_cstr.as_ptr(), port_cstr.as_ptr()) };

    let connection = unsafe { nw_connection_create(endpoint, parameters) };
    unsafe { nw_release(endpoint as *mut c_void) };
    unsafe { nw_release(parameters as *mut c_void) };
    unsafe {
        G_CONNECTION = Some(connection);
    }

    start_connection(connection);
    start_send_receive_loop(connection);
}

fn start_connection(connection: nw_connection_t) {
    unsafe { nw_connection_set_queue(connection, dispatch_get_main_queue()) };

    unsafe { nw_retain(connection as *mut c_void) }; // Hold a reference until cancelled
    let mut connection_state_changed_handler =
        ConcreteBlock::new(|state: nw_connection_state_t, error: nw_error_t| {
            if !error.is_null() {
                let error_no = unsafe { nw_error_get_error_code(error) };
                eprintln!("connection error: {error_no}");
            }

            let remote = unsafe { nw_connection_copy_endpoint(G_CONNECTION.unwrap()) };
            let hostname_cstr = unsafe {
                let hostname_ptr = nw_endpoint_get_hostname(remote);
                CStr::from_ptr(hostname_ptr)
            };
            let hostname = hostname_cstr.to_str().unwrap();
            let port = unsafe { nw_endpoint_get_port(remote) };

            match state {
                nw_connection_state_t::nw_connection_state_invalid => {
                    eprintln!("connection invalid state");
                }
                nw_connection_state_t::nw_connection_state_waiting => {
                    println!("connecting to {hostname} port {port}: waiting");
                }
                nw_connection_state_t::nw_connection_state_preparing => {
                    println!("connecting to {hostname} port {port}: preparing");
                }
                nw_connection_state_t::nw_connection_state_ready => {
                    println!("connecting to {hostname} port {port}: ready");
                }
                nw_connection_state_t::nw_connection_state_failed => {
                    println!("connecting to {hostname} port {port}: failed");
                }
                nw_connection_state_t::nw_connection_state_cancelled => {
                    unsafe { nw_release(G_CONNECTION.unwrap() as *mut c_void) };
                }
            }

            unsafe { nw_release(remote as *mut c_void) };
        });
    let block = connection_state_changed_handler.copy();
    unsafe { nw_connection_set_state_changed_handler(connection, &block) };

    unsafe { nw_connection_start(connection) };
}

fn start_send_receive_loop(connection: nw_connection_t) {
    send_loop(connection);
    receive_loop(connection);
}

fn send_loop(connection: nw_connection_t) {
    let stdin_queue = Queue::create("stdin queue", QueueAttribute::Serial);
    stdin_queue.exec_async(|| loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let buffer = input.as_bytes();
        let dispatch_data = unsafe {
            dispatch_data_create(
                buffer.as_ptr() as *const c_void,
                buffer.len(),
                dispatch_get_main_queue(),
                ptr::null(),
            )
        };

        let mut connection_send_completion_handler = ConcreteBlock::new(|error: nw_error_t| {
            if !error.is_null() {
                let error_no = unsafe { nw_error_get_error_code(error) };
                eprintln!("connection send error: {error_no}");
            }
            // println!("connection send completed");
        });
        let block = connection_send_completion_handler.copy();
        unsafe {
            nw_connection_send(
                G_CONNECTION.unwrap(),
                dispatch_data,
                ptr::null() as *const c_void,
                true,
                &block,
            )
        };
    });
}

fn receive_loop(connection: nw_connection_t) {
    let mut nw_connection_receive_completion_handler = ConcreteBlock::new(
        |content: dispatch_data_t,
         context: nw_content_context_t,
         is_complete: bool,
         error: nw_error_t| {
            if !error.is_null() {
                let error_no = unsafe { nw_error_get_error_code(error) };
                eprintln!("connection message receiving error: {error_no}");
            }
            // println!("connection message receiving completed");

            let mut buffer = 0 as *mut c_void;
            let mut buffer_len: usize = 0;
            let data = unsafe { dispatch_data_create_map(content, &mut buffer, &mut buffer_len) };
            if buffer.is_null() {
                println!("connection received null message");
            } else {
                let s =
                    unsafe { String::from_raw_parts(buffer as *mut u8, buffer_len, buffer_len) };
                println!("[REV]: {s}");
                receive_loop(unsafe { G_CONNECTION.unwrap() });
            }
        },
    );

    unsafe {
        let block = nw_connection_receive_completion_handler.copy();
        nw_connection_receive_message(G_CONNECTION.unwrap(), &block)
    };
}

fn parse_args() -> Result<AppArgs, pico_args::Error> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print_help_and_exit_process();
    }

    let args = AppArgs {
        ipv4_only: pargs.contains("-4"),
        ipv6_only: pargs.contains("-6"),
        detach_from_stdin: pargs.contains("-d"),
        server: pargs.contains("-l"),
        verbose: pargs.contains("-v"),
        local_address: pargs.opt_value_from_str("-s")?,
        local_port: pargs.opt_value_from_str("-p")?,
        hostname: {
            let remaining_len = pargs.clone().finish().len();
            if remaining_len > 1 {
                pargs.opt_free_from_str()?
            } else {
                None
            }
        },
        port: pargs.opt_free_from_str()?,
    };

    let remaining = pargs.finish();
    if !remaining.is_empty() {
        eprintln!("Warning: unused arguments left: {remaining:?}.");
    }

    Ok(args)
}

fn print_usage(ret: i32) {
    print!("{USAGE}");
    if ret != 0 {
        std::process::exit(ret);
    }
}

fn print_help_and_exit_process() {
    print_usage(0);
    print!("{HELP}");
    std::process::exit(1);
}

#[derive(Debug)]
struct AppArgs {
    ipv4_only: bool,
    ipv6_only: bool,
    detach_from_stdin: bool,
    server: bool,
    local_address: Option<String>,
    local_port: Option<String>,
    verbose: bool,
    hostname: Option<String>,
    port: Option<String>,
}

static USAGE: &str = "\
USAGE:
  netcat [-46dhlv] [-p source_port]
         [-s source_ip_address] [hostname] [port]
";

static HELP: &str = "\
Command Summary:
         -4         Use IPv4 only
         -6         Use IPv6 only
         -d         Detach from stdin
         -h         Print this help text
         -l         Create a listener to accept inbound connections
         -p port    Use a local port for outbound connections
         -s addr    Set local address for outbound connections
         -v         Verbose
";

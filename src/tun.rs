use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;
use std::sync::{Arc, mpsc};
use std::thread;

use crate::logging::Logging;
use crate::protocol::internet::Datagram;
use crate::thread_pool::ThreadPool;

const MTU: usize = 1500;

pub fn main(fd: c_int, log_path: *const c_char) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let mut interface = unsafe { File::from_raw_fd(raw_fd) };

    let c_str = unsafe { CStr::from_ptr(log_path) };
    let logging_path = c_str.to_string_lossy().into_owned();
    let mut logging = Logging::new(&logging_path);
    logging.i(format!("Hello tun2socks main, fd({logging_path}), logging_path({logging_path})"));

    let (reporter, events) = mpsc::channel();
    let reporter = Arc::new(reporter);
    let pool = ThreadPool::new(10, Arc::clone(&reporter));

    let mut cloned_interface = interface.try_clone().unwrap();
    let mut cloned_logging = logging.clone();
    thread::spawn(move || {
        ThreadPool::run(&mut cloned_interface, &mut cloned_logging, events);
    });

    let mut buf = vec![0; MTU]; // Usual internet header length
    let mut last_err = Error::new(ErrorKind::InvalidInput, "Oh no");

    loop {
        if !isRunning() {
            logging.i("tun2socks recv stop signal".to_string());
            break;
        }
        
        match interface.read(&mut buf) {
            Ok(0) => {
                logging.i("reach end".to_string());
                break;
            }
            Ok(n) => {
                let bytes = &buf[..n];

                #[cfg(any(target_os = "macos", target_os = "ios"))]
                    let bytes = &buf[4..n];

                if n < 20 {
                    logging.e(format!("error internet datagram(len[{n}]): {:?}", bytes));
                    continue;
                }
                logging.i(format!("--->> Recv: len({})\n{:?}", n, bytes));

                let version = (bytes[0] >> 4) & 0b1111;
                if version != 4 {
                    logging.w(format!("Unsupported version {version}"));
                    continue;
                };

                let datagram = Datagram::new(&bytes);
                ThreadPool::execute(datagram, &mut logging);
            }
            Err(err) => {
                match err.kind() {
                    ErrorKind::WouldBlock => {}
                    ErrorKind::InvalidInput => {
                        logging.i("tun read error InvalidInput".to_string());
                        break;
                    }
                    ErrorKind::Other => {
                        logging.e(format!("tun read error other: {:#?}", err));
                    }
                    _ => {
                        logging.e(format!("tun read error _: {:#?}", err));
                        break;
                    }
                }

                if err.kind() != last_err.kind() {
                    last_err = err;
                    logging.i(format!("tun read error, kind: {}, err: {}", last_err.kind(), last_err));
                };
            }
        }
    }

    drop(interface);
    drop(reporter);

    ThreadPool::stop()
}

pub fn stop() {
    unsafe {
        RUNNING = false;
    }
}

pub fn isRunning() -> bool {
    unsafe { RUNNING }
}

static mut RUNNING: bool = true;
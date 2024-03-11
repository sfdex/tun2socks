use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;
use std::thread;
use std::time::SystemTime;
use crate::logging::Logging;

const MTU: usize = 1500;

pub fn main(fd: c_int, log_path: *const c_char) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let c_str = unsafe { CStr::from_ptr(log_path) };
    let path = c_str.to_string_lossy().into_owned();

    let time = SystemTime::now();

    let mut logging = Logging::new(&path);
    logging.d("Hello tun2socks".to_string());
    logging.i(format!("main({}, {})", fd, &path));

    let mut stream = unsafe { File::from_raw_fd(raw_fd) };
    let mut buf = vec![0; MTU]; // Usual internet header length
    let mut last_err = Error::new(ErrorKind::InvalidInput, "Oh no");
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                logging.i("reach end".to_string());
            }
            Ok(n) => {
                let datagram = &buf[4..n];
                if n < 20 {
                    logging.e(format!("error internet datagram(len[{n}]): {:?}", datagram));
                    continue;
                }

                let id = time.elapsed().unwrap().as_millis() as u32;

                handle_datagram(&datagram, id, &mut stream, &mut logging);
            }
            Err(err) => {
                if err.kind() != last_err.kind() {
                    last_err = err;
                    logging.i(format!("tun read error, kind: {}, err: {}", last_err.kind(), last_err));
                };
                // Socket operation on non-socket (os error 88)
            }
        }
    }
}

pub fn handle_datagram(datagram: &[u8], id: u32, stream: &mut File, logging: &mut Logging) {
    logging.i(format!("Datagram: len({}), {:?}", (&datagram).len(), &datagram));

    // msg[0] & 4 == 4 #ipv4
    // msg[0] & 6 == 6 #ipv6
    let version = (datagram[0] >> 4) & 0b1111;
    match version {
        4 => {
            // stream.read(&mut buf2) // Read remaining bytes error: OS(11), Operation would block. Don't know why
            let data = datagram.to_vec();
            let mut copy_stream = stream.try_clone().unwrap();
            let mut copy_logging = logging.clone();

            let s = thread::spawn(move || {
                // let result = panic::catch_unwind(|| {
                crate::dispatcher::dispatch(data, id, &mut copy_stream, &mut copy_logging);
                // });
                // if let Err(err) = result {
                //     copy_logging.e(format!("Error on thread: {:?}", err));
                // }
            });

            s.join().expect("TODO: panic message");
        }
        6 => {
            logging.w("Unsupported version ipv6".to_string());
        }
        _ => {
            logging.e(format!("Error version {}", version));
        }
    }
}
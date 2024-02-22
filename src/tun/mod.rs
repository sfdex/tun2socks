use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;
use std::thread;
use crate::dispatcher::dispath;
use crate::logging::Logging;

pub fn main(fd: c_int, log_path: *const c_char) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let c_str = unsafe { CStr::from_ptr(log_path) };
    let path = c_str.to_string_lossy().into_owned();
    // let mut f = File::create(&path).unwrap();
    // let mut f = OpenOptions::new().append(true).create(true).open(&path).unwrap();
    // writeln!(&mut f, "Hello tun2socks").unwrap();
    // writeln!(&mut f, "main({}, {})", fd, path).unwrap();

    let mut logging = Logging::new(&path);
    logging.d("Hello tun2socks".to_string());
    logging.i(format!("main({}, {})", fd, &path));

    let mut stream = unsafe { File::from_raw_fd(raw_fd) };
    let mut buf = vec![0; 20]; // Usual internet header length
    let mut last_err = Error::new(ErrorKind::InvalidInput, "Oh no");
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                print!("tun read reach end");
                logging.i("reach end".to_string());
            }
            Ok(n) => {
                let header = &buf[..n];
                if n != 20 {
                    logging.e(format!("error internet datagram(len[{n}]): {:?}", header));
                    continue;
                }

                println!("tun read msg: {:?}", header);
                logging.i(format!("header{:?}", &header));

                // msg[0] & 4 == 4 #ipv4
                // msg[0] & 6 == 6 #ipv6
                let version = (header[0] >> 4) & 0b1111;
                match version {
                    4 => {
                        // Read remaining bytes
                        let total_length = header[2] as u16 * 256 + header[3] as u16;
                        let remaining_length = (total_length - 20) as usize;
                        let mut buf = vec![0; remaining_length];
                        if let Ok(n) = stream.read(&mut buf) {
                            if n != remaining_length {
                                logging.e(format!("Remaining packet read error, remaining {remaining_length}, actually {n}"));
                                continue;
                            }

                            // Merge internet datagram header and payload
                            let mut data = Vec::new();
                            data.extend_from_slice(&header);
                            data.extend_from_slice(&buf);

                            let mut copy_stream = stream.try_clone().unwrap();
                            let mut copy_logging = logging.clone();
                            thread::spawn(move || {
                                dispath(data, &mut copy_stream, &mut copy_logging);
                            });
                        } else {
                            logging.e("Remaining packet read error".to_string());
                        }
                    }
                    6 => {
                        logging.w("Unsupported version ipv6".to_string());
                    }
                    _ => {
                        logging.e(format!("Error version {}", version));
                    }
                }
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
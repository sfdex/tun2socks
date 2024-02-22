use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;
use std::thread;
use crate::logging::Logging;

const MTU: usize = 1500;

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
    let mut buf = vec![0; MTU]; // Usual internet header length
    let mut last_err = Error::new(ErrorKind::InvalidInput, "Oh no");
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                print!("tun read reach end");
                logging.i("reach end".to_string());
            }
            Ok(n) => {
                let header = &buf[..n];
                if n < 20 {
                    logging.e(format!("error internet datagram(len[{n}]): {:?}", header));
                    continue;
                }

                println!("tun read msg: {:?}", header);
                logging.i(format!("Datagram: len({n}), {:?}", &header));

                // msg[0] & 4 == 4 #ipv4
                // msg[0] & 6 == 6 #ipv6
                let version = (header[0] >> 4) & 0b1111;
                match version {
                    4 => {
                        // Read remaining bytes
                        let total_length = header[2] as usize * 256 + header[3] as usize;
                        let remaining_length = total_length - n;
                        let mut buf2 = vec![0; remaining_length];
                        logging.i(format!("packet: total[{total_length}], remaining[{remaining_length}], buf2[{}]", buf2.len()));

                        // match stream.read(&mut buf2) {
                        //     Ok(n) => {
                        //         if n != remaining_length {
                        //             logging.e(format!("Remaining packet read error, remaining {remaining_length}, actually {n}"));
                        //             continue;
                        //         }

                        // Merge internet datagram header and payload
                        let mut data = Vec::new();
                        data.extend_from_slice(&header);
                        // data.extend_from_slice(&buf2[..n]);

                        let mut copy_stream = stream.try_clone().unwrap();
                        let mut copy_logging = logging.clone();
                        thread::spawn(move || {
                            crate::dispatcher::dispatch(data, &mut copy_stream, &mut copy_logging);
                        });
                        //     }
                        //     Err(err) => {
                        //         logging.e(format!("Remaining packet read error(kind:{}, msg:{}, desc:{})", err.kind(), err, err.to_string()));
                        //     }
                        // }
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
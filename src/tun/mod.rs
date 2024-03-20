use std::ffi::{c_char, CStr};
use std::fs::File;
use std::io::{Error, ErrorKind, Read};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;
use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use std::time::SystemTime;
use crate::logging::Logging;
use crate::protocol::internet::Datagram;
use crate::thread_pool::ThreadPool;

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

    let (reporter, events) = mpsc::channel();
    let pool = ThreadPool::new(10, Arc::new(Mutex::new(reporter)));
    let mut cloned_stream = stream.try_clone().unwrap();
    let mut cloned_logging = logging.clone();
    
    thread::spawn(move || {
        ThreadPool::run(&mut cloned_stream, &mut cloned_logging, events);
    });
    
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                logging.i("reach end".to_string());
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

                // handle_datagram(&bytes, &mut stream, &mut logging);

                let version = (bytes[0] >> 4) & 0b1111;
                if version != 4 {
                    logging.w(format!("Unsupported version {version}"));
                    continue;
                };

                let datagram = Datagram::new(&bytes);
                ThreadPool::execute(datagram, &mut logging);
                // tx.send(datagram).unwrap();
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

    // pool.run();
}

pub fn handle_datagram(datagram: &[u8], stream: &mut File, logging: &mut Logging) {
    logging.i(format!("--->> Recv: len({}), {:?}", (&datagram).len(), &datagram));

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
                crate::dispatcher::dispatch(data, &mut copy_stream, &mut copy_logging);
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
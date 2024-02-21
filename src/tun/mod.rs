use std::ffi::{c_char, CStr};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;

pub fn main(fd: c_int, log_path: *const c_char) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let c_str = unsafe { CStr::from_ptr(log_path) };
    let path = c_str.to_string_lossy().into_owned();
    // let mut f = File::create(&path).unwrap();
    let mut f = OpenOptions::new().append(true).create(true).open(&path).unwrap();
    writeln!(&mut f, "Hello tun2socks").unwrap();
    writeln!(&mut f, "main({}, {})", fd, path).unwrap();

    let mut stream = unsafe { File::from_raw_fd(raw_fd) };
    let mut buf = vec![0; 5]; // Usual ip header length
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                print!("tun read reach end");
                writeln!(&mut f, "reach end").unwrap();
            }
            Ok(n) => {
                let msg = &buf[..n];
                println!("tun read msg: {:?}", msg);
                writeln!(&mut f, "{:?}", &msg).unwrap();
            }
            Err(err) => {
                println!("tun read err: {}", err);
                writeln!(&mut f, "tun read err: {}", err).unwrap();
                // Socket operation on non-socket (os error 88)
            }
        }
    }
}
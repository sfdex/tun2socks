use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::raw::c_int;

pub fn main(fd: c_int) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let mut stream = unsafe { TcpStream::from_raw_fd(raw_fd) };
    let mut buf = vec![0; 5]; // Usual ip header length
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                print!("tun read reach end");
            }
            Ok(n) => {
                let msg = buf[n];
                println!("tun read msg: {:?}", msg);
            }
            Err(err) => {
                println!("tun read err: {}", err)
            }
        }
    }
}
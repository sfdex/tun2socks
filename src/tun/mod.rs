use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::raw::c_int;

fn read(fd: c_int) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let mut stream = unsafe { TcpStream::from_raw_fd(raw_fd) };
    let mut buf = vec![0; 1024];
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

fn write(fd: c_int, data: Vec<u8>) {
    let raw_fd = RawFd::from(fd).as_raw_fd();
    let mut stream = unsafe { TcpStream::from_raw_fd(raw_fd) };
    match stream.write_all(&data) {
        Ok(_) => {
            println!("tun write success");
        }
        Err(err) => {
            println!("tun write err: {}", err)
        }
    }
}
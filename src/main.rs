use std::ffi::CStr;
use std::os::fd::AsRawFd;
use tun2socks_rust::tun;
use t::Configuration;
use t::platform::Device;

extern crate tun as t;

fn entry(){
    let mut config = Configuration::default();
    config.address((10,0,0,1))
        .destination((10,0,0,9))
        .netmask((255,255,255,0))
        .up();

    let mut dev = Device::new(&config).unwrap();
    // dev.set_nonblock().unwrap();
    tun::main(dev.as_raw_fd(), CStr::from_bytes_with_nul(b"build/logging.txt\0").unwrap().as_ptr());
}


fn main() {
    // dns_resolve();
    // test_datagram_tcp()
    entry();
}

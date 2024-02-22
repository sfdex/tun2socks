use std::ffi::c_char;
use std::os::raw::c_int;
mod tun;
pub mod dns;
mod socks;
pub mod protocol;

mod dispatcher;

mod logging;

#[no_mangle]
pub extern "C" fn tun2socks(fd: c_int, log_path: *const c_char) {
    tun::main(fd, log_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_checksum() {
        let bytes: Vec<u8> = vec![69, 0, 0, 60, 177, 109, 64, 0, 64, 6, 50, 219, 10, 0, 0, 2, 106, 75, 226, 38];
        let checksum = protocol::internet::Datagram::calc_checksum(&bytes); // 0
        let bytes: Vec<u8> = vec![69, 0, 0, 60, 177, 109, 64, 0, 64, 6, 0, 0, 10, 0, 0, 2, 106, 75, 226, 38]; // set checksum to 0
        // let checksum = protocol::internet::Datagram::calc_checksum(&bytes); // 13019
        assert_eq!(checksum, 0)
    }
}
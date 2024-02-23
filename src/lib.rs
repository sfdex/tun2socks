use std::ffi::c_char;
use std::os::raw::c_int;

mod tun;
pub mod dns;
mod socks;
pub mod protocol;

mod dispatcher;

mod logging;

mod util;

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
        assert_eq!(checksum, [0, 0])
    }

    #[test]
    fn test_int_to_bytes() {
        assert_eq!(util::u32_to_bytes(4294967295), [255, 255, 255, 255]);
        // assert_eq!(4294967295u32.to_be_bytes(), [255, 255, 255, 255]);
        assert_eq!(16777215u32.to_be_bytes(), [0, 255, 255, 255]);
        // assert_eq!(util::u16_to_bytes(65535), [255, 255]);
    }
}
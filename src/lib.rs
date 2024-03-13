use std::ffi::c_char;
use std::os::raw::c_int;

pub mod tun;
pub mod dns;
mod socks;
pub mod protocol;

mod dispatcher;

pub mod logging;

pub mod util;

#[no_mangle]
pub extern "C" fn tun2socks(fd: c_int, log_path: *const c_char) {
    tun::main(fd, log_path);
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::logging::Logging;
    use crate::protocol::internet::Datagram;
    use crate::util::bytes_to_u32;
    use super::*;

    #[test]
    fn ip_checksum() {
        let mut bytes = [69, 0, 0, 60, 74, 107, 64, 0, 64, 6, 34, 152, 10, 0, 2, 16, 192, 168, 1, 1];
        let rs = Datagram::verify_checksum(&bytes.to_vec());
        assert_eq!(rs, true);
        (bytes[10], bytes[11]) = (0, 0);
        let checksum = Datagram::calc_checksum(&bytes.to_vec());
        assert_eq!(checksum, [34, 152]);
    }

    #[test]
    fn int_to_bytes() {
        assert_eq!(util::u32_to_bytes(4294967295), [255, 255, 255, 255]);
        // assert_eq!(4294967295u32.to_be_bytes(), [255, 255, 255, 255]);
        assert_eq!(16777215u32.to_be_bytes(), [0, 255, 255, 255]);
        // assert_eq!(util::u16_to_bytes(65535), [255, 255]);
    }

    #[test]
    fn bytes_to_int() {
        assert_eq!(util::bytes_to_u32(&[248, 139, 81, 202]), 4169880010);
    }

    #[test]
    fn datagram_tcp() {
        let datagram = [];
        let mut stream = File::create("build/stream.txt").unwrap();
        let mut logging = Logging::new("build/logging.txt");
        tun::handle_datagram(&datagram, &mut stream, &mut logging);
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn tcp_checksum() {
        let mut header = Vec::new();

        let mut pseudo_header = Vec::new();
        pseudo_header.extend_from_slice(&[10, 0, 2, 16]);
        pseudo_header.extend_from_slice(&[192, 168, 1, 1]);
        pseudo_header.push(0u8);
        pseudo_header.push(6);
        pseudo_header.extend_from_slice(&[0, 40]);

        let tcp_header = [133, 156, 3, 85, 238, 109, 198, 113, 0, 0, 0, 0, 160, 2, 255, 255, 42, 67, 0, 0, 2, 4, 4, 196, 4, 2, 8, 10, 198, 91, 76, 200, 0, 0, 0, 0, 1, 3, 3, 6];

        header.extend_from_slice(&pseudo_header);
        header.extend_from_slice(&tcp_header);

        let rs = Datagram::verify_checksum(&header);
        assert_eq!(rs, true);

        (header[28], header[29]) = (0, 0);
        let checksum = Datagram::calc_checksum(&header);
        assert_eq!(checksum, [42, 67]);
    }

    #[test]
    fn tcp_checksum_blank() {
        let mut bytes = [10, 0, 0, 1, 10, 0, 0, 9, 0, 6, 0, 34, 156, 108, 0, 203, 81, 127, 41, 26, 0, 0, 11, 186, 128, 24, 1, 246, 176, 23, 0, 0, 1, 1, 8, 10, 176, 3, 121, 24, 0, 0, 2, 235, 97, 10];
        println!("bytes: len({})", bytes.len());
        let rs = Datagram::verify_checksum(&bytes.to_vec());
        assert_eq!(rs, true);
        println!("src checksum: {:?}", &bytes[28..30]);
        (bytes[28], bytes[29]) = (0, 0);
        let checksum = Datagram::calc_checksum(&bytes.to_vec());
        println!("dst checksum: {:?}, hex({:x})", checksum, (bytes_to_u32(&checksum) as u16));
        assert_eq!(checksum, [176, 23]);
    }
}
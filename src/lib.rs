mod tun;
pub mod dns;
mod socks;
pub mod protocol;

pub fn tun2socks() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_checksum() {
        let bytes: Vec<u8> = vec![69, 0, 0, 60, 177, 109, 64, 0, 64, 6, 50, 219, 10, 0, 0, 2, 106, 75, 226, 38];
        let checksum = protocol::ip::Datagram::calc_checksum(&bytes); // 0
        let bytes: Vec<u8> = vec![69, 0, 0, 60, 177, 109, 64, 0, 64, 6, 0, 0, 10, 0, 0, 2, 106, 75, 226, 38]; // set checksum to 0
        // let checksum = protocol::ip::Datagram::calc_checksum(&bytes); // 13019
        assert_eq!(checksum, 0)
    }
}
mod tun;
mod socks;
pub mod protocol;

pub fn tun2socks() {

}

#[cfg(test)]
mod tests{
    use super::*;
    fn calc_checksum() {
        let bytes:Vec<u8> = [].to_vec();
        protocol::ip::Datagram::calc_checksum(&bytes);
    }
}
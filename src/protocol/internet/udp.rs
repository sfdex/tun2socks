use crate::protocol::internet::{Datagram, PseudoHeader};
use crate::util::bytes_to_u32;

pub struct Udp {
    pub header: Header,
    pub payload: Vec<u8>,
}

struct Header {
    pub src_port: [u8; 2],
    pub dst_port: [u8; 2],
    pub length: [u8; 2],
    pub checksum: [u8; 2],
}

impl Udp {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            header: Header {
                src_port: [bytes[0], bytes[1]],
                dst_port: [bytes[2], bytes[3]],
                length: [bytes[4], bytes[5]],
                checksum: [bytes[6], bytes[7]],
            },
            payload: bytes[8..bytes.len()].to_vec(),
        }
    }

    pub fn info(&self) -> String {
        let mut info = String::new();
        info.push_str("UDP info:\n");
        info.push_str(&format!("\tPort({}) => Port({})\n", bytes_to_u32(&self.header.src_port), bytes_to_u32(&self.header.dst_port)));
        info.push_str(&format!("\tPayload: {}\n", String::from_utf8_lossy(&self.payload)));
        info
    }

    pub fn pack(&self, pseudo_header: &mut PseudoHeader, payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&self.header.dst_port);
        packet.extend_from_slice(&self.header.src_port);

        let length = ((8 + payload.len()) as u16).to_be_bytes();
        packet.extend_from_slice(&length);
        packet.extend_from_slice(&[0, 0]); // checksum

        packet.extend_from_slice(&payload);

        // Checksum
        pseudo_header.length = length;
        let mut header = pseudo_header.to_be_bytes();
        header.extend_from_slice(&packet[..packet.len()]);
        if header.len() % 2 != 0 { header.push(0); }
        let checksum = Datagram::calc_checksum(&header);
        (packet[6], packet[7]) = (checksum[0], checksum[1]);

        packet
    }
}
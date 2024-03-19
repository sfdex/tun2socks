use std::net::SocketAddr;
use crate::protocol::internet::{Datagram, Packet, PseudoHeader};
use crate::util::bytes_to_u32;

/*
                      User Datagram Header Format
                      
                  0      7 8     15 16    23 24    31
                 +--------+--------+--------+--------+
                 |     Source      |   Destination   |
                 |      Port       |      Port       |
                 +--------+--------+--------+--------+
                 |                 |                 |
                 |     Length      |    Checksum     |
                 +--------+--------+--------+--------+
                 |                                   :
                 :               Data                :
                 :                                   |
                 +------------------+--------+-------+
 */
pub struct Udp {
    header: Header,
    pseudo_header: PseudoHeader,
    payload: Vec<u8>,
}

struct Header {
    pub src_port: [u8; 2],
    pub dst_port: [u8; 2],
    pub length: [u8; 2],
    pub checksum: [u8; 2],
}

impl Udp {
    pub fn new(bytes: &[u8], pseudo_header: PseudoHeader) -> Self {
        Self {
            header: Header {
                src_port: [bytes[0], bytes[1]],
                dst_port: [bytes[2], bytes[3]],
                length: [bytes[4], bytes[5]],
                checksum: [bytes[6], bytes[7]],
            },
            pseudo_header,
            payload: bytes[8..bytes.len()].to_vec(),
        }
    }
}

impl Packet for Udp {
    fn dst_addr(&self) -> SocketAddr {
        SocketAddr::new(self.pseudo_header.dst_ip.into(), bytes_to_u32(&self.header.dst_port) as u16)
    }
    fn dst_port(&self) -> u16 {
        bytes_to_u32(&self.header.dst_port) as u16
    }

    fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    fn info(&self) -> String {
        let mut info = String::new();
        info.push_str("UDP info:\n");
        info.push_str(&format!("\tPort({}) => Port({})\n", bytes_to_u32(&self.header.src_port), bytes_to_u32(&self.header.dst_port)));
        info.push_str(&format!("\tPayload: {}\n", String::from_utf8_lossy(&self.payload)));
        info
    }

    fn pack(&self, _: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&self.header.dst_port);
        packet.extend_from_slice(&self.header.src_port);

        let length = ((8 + payload.len()) as u16).to_be_bytes();
        packet.extend_from_slice(&length);
        packet.extend_from_slice(&[0, 0]); // checksum

        packet.extend_from_slice(&payload);

        // Checksum
        let mut header = self.pseudo_header.to_be_bytes();
        (header[10], header[11]) = (length[0], length[1]); // Set length
        header.extend_from_slice(&packet[..packet.len()]);
        if header.len() % 2 != 0 { header.push(0); }
        let checksum = Datagram::calc_checksum(&header);
        (packet[6], packet[7]) = (checksum[0], checksum[1]);

        packet
    }
}
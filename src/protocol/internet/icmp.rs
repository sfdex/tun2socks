use crate::protocol::internet::Datagram;

pub struct Icmp<T: IcmpType> {
    header: Header,
    payload: T,
}

pub trait IcmpType {
    const ECHO_REPLY: u8 = 0;
    const DESTINATION_UNREACHABLE: u8 = 3;
    const SOURCE_QUENCH: u8 = 4;
    const REDIRECT: u8 = 5;
    const ECHO_REQUEST: u8 = 8;
    const ROUTER_ADVERTISEMENT: u8 = 9;
    const ROUTER_SOLICITATION: u8 = 10;
    const TIME_EXCEEDED: u8 = 11;
    const PARAMETER_PROBLEM: u8 = 12;
    const TIMESTAMP_REQUEST: u8 = 13;
    const TIMESTAMP_REPLY: u8 = 14;
    const INFORMATION_REQUEST: u8 = 15;
    const INFORMATION_REPLY: u8 = 16;
    const ADDRESS_MASK_REQUEST: u8 = 17;
    const ADDRESS_MASK_REPLY: u8 = 18;

    fn new(payload: &[u8]) -> Self;
    fn pack(&self) -> Box<[u8]>;
}

struct Header {
    tp: u8,
    code: u8,
    checksum: [u8; 2],
}

pub struct Echo {
    id: [u8; 2],
    seq: [u8; 2],
    data: Vec<u8>,
    len: usize,
}

impl IcmpType for Echo {
    const ECHO_REPLY: u8 = 0;
    const ECHO_REQUEST: u8 = 8;

    fn new(payload: &[u8]) -> Self {
        Self {
            id: [payload[0], payload[1]],
            seq: [payload[2], payload[3]],
            data: payload[4..].to_vec(),
            len: payload.len()
        }
    }

    fn pack(&self) -> Box<[u8]> {
        let mut packet = Vec::new();
        packet.extend_from_slice(&self.id);
        packet.extend_from_slice(&self.seq);
        packet.extend_from_slice(&self.data);
        packet.into_boxed_slice()
    }
}

impl<T: IcmpType> Icmp<T> {
    pub fn new(bytes: &[u8]) -> Self {
        let header = Header {
            tp: bytes[0],
            code: bytes[1],
            checksum: [bytes[2], bytes[3]],
        };

        let payload = T::new(&bytes[4..]);

        Self {
            header,
            payload,
        }
    }
    
    pub fn pack(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push(0);
        packet.push(self.header.code);
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&self.payload.pack());

        let checksum = if packet.len() % 2 != 0 {
            packet.push(0);
            let result = Datagram::calc_checksum(&packet);
            packet.pop().unwrap();
            result
        } else {
            Datagram::calc_checksum(&packet)
        };

        (packet[2], packet[3]) = (checksum[0], checksum[1]);
        
        packet
    }
}
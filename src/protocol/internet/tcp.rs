use crate::protocol::internet::Datagram;

pub struct Tcp {
    pub header: Header,
    pub payload: Vec<u8>,
}

pub struct Header {
    pub src_port: [u8; 2], pub dst_port: [u8; 2],
    pub seq_no: [u8; 4],
    pub ack_no: [u8; 4],
    pub data_offset: u8, pub flags: u8, pub window: [u8; 2],
    pub checksum: [u8; 2], pub urgent_pointer: [u8; 2],
    pub options: Vec<u8>,
}

impl Tcp {
    pub fn pack(&self, flags: u8) -> Vec<u8>{
        let mut pack = Vec::new();
        let header = &self.header;
        pack.extend_from_slice(&header.dst_port);
        pack.extend_from_slice(&header.src_port);
        pack.extend_from_slice(&header.seq_no);
        pack.extend_from_slice(&header.ack_no);
        pack.extend_from_slice(&[0, flags, header.window[0], header.window[1]]);
        pack.extend_from_slice(&[0, 0, header.urgent_pointer[0], header.urgent_pointer[1]]);
        pack.extend_from_slice(&header.options);

        // Set header checksum
        let checksum = Datagram::calc_checksum(&pack);
        (pack[16], pack[17]) = (checksum[0], checksum[1]);

        pack
    }
    pub fn new(bytes: Vec<u8>) -> Self {
        let data_offset = (bytes[12] >> 4 & 0b1111) as usize;
        let options = bytes[20..data_offset].to_vec();

        let header = Header{
            src_port: [bytes[0], bytes[1]],
            dst_port: [bytes[2], bytes[3]],
            seq_no: [bytes[4], bytes[5], bytes[6], bytes[7]],
            ack_no: [bytes[8], bytes[9], bytes[10], bytes[11]],
            data_offset: data_offset as u8,
            flags: bytes[13] & 0b111111,
            window: [bytes[14], bytes[15]],
            checksum: [bytes[16], bytes[17]],
            urgent_pointer: [bytes[18], bytes[19]],
            options
        };

        let payload = bytes[data_offset..].to_vec();

        Self{
            header,
            payload,
        }
    }
}
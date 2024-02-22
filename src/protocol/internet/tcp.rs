pub struct Tcp {
    header: Header,
    payload: Vec<u8>,
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
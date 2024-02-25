use crate::protocol::internet::Datagram;
use crate::util::bytes_to_u32;

pub struct Tcp {
    pub header: Header,
    pub payload: Vec<u8>,
}

pub struct Header {
    pub src_port: [u8; 2], pub dst_port: [u8; 2],
    pub seq_no: [u8; 4],
    pub ack_no: [u8; 4],
    pub data_offset: u8, pub control_flags: u8, pub window: [u8; 2],
    pub checksum: [u8; 2], pub urgent_pointer: [u8; 2],
    pub options: Vec<u8>,
}

#[derive(Debug)]
pub enum ControlType{
    SYN,
    SACK,
    ACK,
    PUSH,
    FIN,
    RST,
    URG,
    UNKNOWN,
}

impl Tcp {
    pub fn new(bytes: &[u8]) -> Self {
        let data_offset = (bytes[12] >> 4 & 0b1111) as usize;
        println!("data_offset: {data_offset}");
        let options = bytes[20..(data_offset * 4)].to_vec();

        let header = Header{
            src_port: [bytes[0], bytes[1]],
            dst_port: [bytes[2], bytes[3]],
            seq_no: [bytes[4], bytes[5], bytes[6], bytes[7]],
            ack_no: [bytes[8], bytes[9], bytes[10], bytes[11]],
            data_offset: data_offset as u8,
            control_flags: bytes[13] & 0b111111,
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

    pub fn pack(&self, flags: u8) -> Vec<u8>{
        let mut pack = Vec::new();
        let header = &self.header;
        let mut seq_no = bytes_to_u32(&header.ack_no);
        if seq_no == 0 {
            seq_no = 1;
        } else {
            seq_no = seq_no + 1
        }
        pack.extend_from_slice(&header.dst_port);
        pack.extend_from_slice(&header.src_port);
        pack.extend_from_slice(&(3001u32.to_be_bytes()));
        pack.extend_from_slice(&seq_no.to_be_bytes());
        pack.extend_from_slice(&[0, flags, header.window[0], header.window[1]]);
        pack.extend_from_slice(&[0, 0, header.urgent_pointer[0], header.urgent_pointer[1]]);
        pack.extend_from_slice(&header.options);

        // Set header checksum
        let checksum = Datagram::calc_checksum(&pack);
        (pack[16], pack[17]) = (checksum[0], checksum[1]);

        pack
    }

    pub fn control_type(&self) -> ControlType {
        let flags = self.header.control_flags;
        let is_urg = flags & 0b100000 == 0b100000;
        let is_ack = flags & 0b010000 == 0b010000;
        let is_psh = flags & 0b001000 == 0b001000;
        let is_rst = flags & 0b000100 == 0b000100;
        let is_syn = flags & 0b000010 == 0b000010;
        let is_fin = flags & 0b000001 == 0b000001;

        if is_syn {
            if is_ack {
                return ControlType::SACK;
            }
            return ControlType::SYN;
        };

        if is_ack {
            if is_psh {
                return ControlType::PUSH;
            }
            return ControlType::ACK;
        };

        ControlType::UNKNOWN
    }

    pub fn info(&self) -> String {
        let mut info = String::new();
        let header = &self.header;
        info.push_str("tcp info: \n");
        info.push_str(&format!("\tseq_no: {}, ", bytes_to_u32(&header.seq_no)));
        info.push_str(&format!("\tack_no: {}\n", bytes_to_u32(&header.ack_no)));
        info.push_str(&format!("\toffset: {}\n", header.data_offset));

        let flags = header.control_flags;
        info.push_str(&format!(
            "\tURG:{}, ACK:{}, PSH:{}, RST:{}, SYN:{}, FIN:{}\n",
            (flags & 32) >> 5, (flags & 16) >> 4, (flags & 8) >> 3, (flags & 4) >> 2, (flags & 2) >> 1, flags & 1
        ));
        info.push_str(&format!("\tcontrol type: {:?}\n", self.control_type()));

        info
    }
}
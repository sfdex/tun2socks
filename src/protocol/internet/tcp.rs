use std::fmt::Display;
use std::ops::Deref;
use crate::protocol::internet::{Datagram, Packet, PseudoHeader};
use crate::util::bytes_to_u32;

/*
   TCP Header Format

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |          Source Port          |       Destination Port        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                        Sequence Number                        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Acknowledgment Number                      |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |  Data |       |C|E|U|A|P|R|S|F|                               |
   | Offset| Rsrvd |W|C|R|C|S|S|Y|I|            Window             |
   |       |       |R|E|G|K|H|T|N|N|                               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |           Checksum            |         Urgent Pointer        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                           [Options]                           |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                                                               :
   :                             Data                              :
   :                                                               |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 */

pub struct Tcp {
    header: Header,
    pseudo_header: PseudoHeader,
    payload: Vec<u8>,
    len: usize,
}

struct Header {
    pub src_port: [u8; 2], pub dst_port: [u8; 2],
    pub seq_no: [u8; 4],
    pub ack_no: [u8; 4],
    pub data_offset: u8, pub flags: u8, pub window: [u8; 2],
    pub checksum: [u8; 2], pub urgent_pointer: [u8; 2],
    pub options: Vec<Option>,
}

struct Option {
    pub kind: u8,
    pub length: u8,
    pub data: Vec<u8>,
}

impl Tcp {
    pub fn new(bytes: &[u8], pseudo_header: PseudoHeader) -> Self {
        let data_offset = (bytes[12] >> 4 & 0b1111) as usize;
        let data_begin_idx = data_offset * 4;

        let options_bytes = bytes[20..data_begin_idx].to_vec();
        let mut options = Vec::new();
        let mut option_idx = 0usize;
        loop {
            if option_idx >= options_bytes.len() { break; }

            let kind = options_bytes[option_idx];
            if kind == 1 || kind == 0 { // A No-Operation Option: This option code can be used between options
                option_idx = option_idx + 1;
                options.push(Option { kind, length: 0, data: Vec::new() });
                continue;
            };

            let length = options_bytes[option_idx + 1];
            let data = (options_bytes[option_idx + 2..option_idx + length as usize]).to_vec();
            options.push(Option { kind, length, data });
            option_idx = option_idx + length as usize;
        };

        let header = Header {
            src_port: [bytes[0], bytes[1]],
            dst_port: [bytes[2], bytes[3]],
            seq_no: [bytes[4], bytes[5], bytes[6], bytes[7]],
            ack_no: [bytes[8], bytes[9], bytes[10], bytes[11]],
            data_offset: data_offset as u8,
            flags: bytes[13],
            window: [bytes[14], bytes[15]],
            checksum: [bytes[16], bytes[17]],
            urgent_pointer: [bytes[18], bytes[19]],
            options,
        };

        let payload = bytes[data_begin_idx..].to_vec();

        Self{
            header,
            pseudo_header,
            payload,
            len: bytes.len()
        }
    }
}

impl Packet for Tcp {
    fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    fn flags_type(&self) -> FlagsType {
        FlagsType(self.header.flags)
    }

    fn  info(&self) -> String {
        let mut info = String::new();
        let header = &self.header;
        info.push_str("TCP info: \n");
        info.push_str(&format!("\tlength: {}\n", self.len));
        info.push_str(&format!("\tseq_no: {}, ", bytes_to_u32(&header.seq_no)));
        info.push_str(&format!("\tack_no: {}\n", bytes_to_u32(&header.ack_no)));
        info.push_str(&format!("\toffset: {}\n", header.data_offset));

        let flags = &header.flags;
        info.push_str(&format!(
            "\tCWR:{}, ECE:{}, URG:{}, ACK:{}, PSH:{}, RST:{}, SYN:{}, FIN:{}\n",
            (flags & (1 << 7)) >> 7, (flags & (1 << 6)) >> 6, (flags & (1 << 5)) >> 5, (flags & (1 << 4)) >> 4,
            (flags & (1 << 3)) >> 3, (flags & (1 << 2)) >> 2, (flags & (1 << 1)) >> 1, flags & 1
        ));
        info.push_str(&format!("\tflags type: {}\n", FlagsType(header.flags)));

        info.push_str("\t---<options>---\n");
        for option in &header.options {
            info.push_str(&format!("\tkind:{}, length:{}, data:{:?}\n", option.kind, option.length, option.data));
            if option.kind == 8 {
                info.push_str(&format!("\t\tTimestamp: TSVal({}), TSecr({})\n", bytes_to_u32(&option.data[0..4]), bytes_to_u32(&option.data[4..8])));
            };
        }
        info.push_str("\t---<options>---\n");
        info.push_str(&format!("\tdata: len({}) {:?}\n", &self.payload.len(), &self.payload));

        info
    }
    
    fn pack(&self, flags: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut pack = Vec::new();
        let header = &self.header;

        let seq_no:u32 = bytes_to_u32(&header.ack_no);
        let seq_no = if seq_no == 0 {
            3001
        } else {
            seq_no
        };

        let ack_no:u32 = bytes_to_u32(&header.seq_no);
        let ack_no = if ack_no == 0 {
            1
        } else if self.payload.len() > 0{
            ack_no + self.payload.len() as u32
        }else {
            ack_no + 1
        };

        pack.extend_from_slice(&header.dst_port);
        pack.extend_from_slice(&header.src_port);
        pack.extend_from_slice(&(seq_no.to_be_bytes()));
        pack.extend_from_slice(&(ack_no.to_be_bytes()));
        pack.extend_from_slice(&[0, flags[0], header.window[0], header.window[1]]);
        pack.extend_from_slice(&[0, 0, header.urgent_pointer[0], header.urgent_pointer[1]]);

        for option in &header.options {
            if option.kind == 1 || option.kind == 0 {
                pack.push(option.kind);
                continue;
            }
            pack.push(option.kind);
            pack.push(option.length);
            if option.kind == 8 {
                pack.extend_from_slice(&seq_no.to_be_bytes());
                pack.extend_from_slice(&option.data[0..4]);
            } else {
                pack.extend_from_slice(&option.data);
            };
        }

        // Set data offset
        let offset = (pack.len() as u8 / 4) << 4;
        pack[12] = offset;

        // Add payload
        pack.extend_from_slice(&payload);

        // Set header checksum
        let mut header = self.pseudo_header.to_be_bytes();
        let length = (pack.len() as u16).to_be_bytes();
        (header[10], header[11]) = (length[0], length[1]); // Set length
        header.extend_from_slice(&pack);
        if header.len() % 2 != 0 { header.push(0) }
        let checksum = Datagram::calc_checksum(&header);
        (pack[16], pack[17]) = (checksum[0], checksum[1]);

        pack
    }
}

#[derive(PartialEq, Eq)]
pub struct FlagsType(pub u8);

pub const ACK: FlagsType = FlagsType(0b00010000);       // .
pub const SYN: FlagsType = FlagsType(0b00000010);       // S
pub const SEW: FlagsType = FlagsType(0b11000010);       // SEW
pub const FIN: FlagsType = FlagsType(0b00000001);       // F
pub const RST: FlagsType = FlagsType(0b00000100);       // R
pub const SYN_ACK: FlagsType = FlagsType(0b00010010);   // S.
pub const PSH_ACK: FlagsType = FlagsType(0b00011000);   // P.
pub const FIN_ACK: FlagsType = FlagsType(0b00010001);   // F.
pub const RST_ACK: FlagsType = FlagsType(0b00010100);   // R.

impl Deref for FlagsType{
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FlagsType {
    fn info(&self) -> &str {
        match *self {
            ACK => "ACK",
            SYN => "SYN",
            SEW => "SEW",
            FIN => "FIN",
            RST => "RST",
            SYN_ACK => "SYN_ACK",
            PSH_ACK => "PSH_ACK",
            FIN_ACK => "FIN_ACK",
            RST_ACK => "RST_ACK",
            _ => "UNKNOWN"
        }
    }
}

impl Display for FlagsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({:08b})", self.info().to_string(), self.0)
    }
}
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use crate::protocol::internet::icmp::Icmp;
use crate::protocol::internet::tcp::{FlagsType, Tcp};
use crate::protocol::internet::udp::Udp;
use crate::util::bytes_to_u32;

pub mod tcp;
pub mod udp;
pub mod icmp;

/*
   Internet Header Format

    0                   1                   2                   3
    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |Version|  IHL  |Type of Service|          Total Length         |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |         Identification        |Flags|      Fragment Offset    |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |  Time to Live |    Protocol   |         Header Checksum       |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                       Source Address                          |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Destination Address                        |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
   |                    Options                    |    Padding    |
   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+


   Assigned Internet Protocol Numbers

      Decimal    Octal      Protocol Numbers                  References
      -------    -----      ----------------                  ----------
           0       0         Reserved                              [JBP]
           1       1         ICMP                               [53,JBP]
           2       2         Unassigned                            [JBP]
           3       3         Gateway-to-Gateway              [48,49,VMS]
           4       4         CMCC Gateway Monitoring Message [18,19,DFP]
           5       5         ST                                 [20,JWF]
           6       6         TCP                                [34,JBP]
           7       7         UCL                                    [PK]
           8      10         Unassigned                            [JBP]
           9      11         Secure                                [VGC]
          10      12         BBN RCC Monitoring                    [VMS]
          11      13         NVP                                 [12,DC]
          12      14         PUP                                [4,EAT3]
          13      15         Pluribus                             [RDB2]
          14      16         Telenet                              [RDB2]
          15      17         XNET                              [25,JFH2]
          16      20         Chaos                                [MOON]
          17      21         User Datagram                      [42,JBP]
          18      22         Multiplexing                       [13,JBP]
          19      23         DCN                                  [DLM1]
          20      24         TAC Monitoring                     [55,RH6]
       21-62   25-76         Unassigned                            [JBP]
          63      77         any local network                     [JBP]
          64     100         SATNET and Backroom EXPAK            [DM11]
          65     101         MIT Subnet Support                    [NC3]
       66-68 102-104         Unassigned                            [JBP]
          69     105         SATNET Monitoring                    [DM11]
          70     106         Unassigned                            [JBP]
          71     107         Internet Packet Core Utility         [DM11]
       72-75 110-113         Unassigned                            [JBP]
          76     114         Backroom SATNET Monitoring           [DM11]
          77     115         Unassigned                            [JBP]
          78     116         WIDEBAND Monitoring                  [DM11]
          79     117         WIDEBAND EXPAK                       [DM11]
      80-254 120-376         Unassigned                            [JBP]
         255     377         Reserved                              [JBP]
*/

pub struct Datagram {
    pub header: Header,
    pub pseudo_header: PseudoHeader,
    pub payload: Payload,
}

pub struct Header {
    pub version_ihl: u8,
    pub dscp_ecn: u8,
    pub total_length: [u8; 2],
    pub identification: [u8; 2],
    pub flags_fragment_offset: [u8; 2],
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: [u8; 2],
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    // options: [u8],
    pub options: Vec<u8>,
}

pub type Payload = Arc<Box<dyn Packet + Send + Sync>>;

/*
                          IPv4 Pseudo-header
                +--------+--------+--------+--------+
                |           Source Address          |
                +--------+--------+--------+--------+
                |         Destination Address       |
                +--------+--------+--------+--------+
                |  zero  |  PTCL  |  Payload Length |
                +--------+--------+--------+--------+
 */

#[derive(Debug, Copy, Clone)]
pub struct PseudoHeader {
    pub src_ip: [u8; 4],
    pub dst_ip: [u8; 4],
    pub protocol: u8,
    pub length: [u8; 2],
}

impl Datagram {
    pub fn new(bytes: &[u8]) -> Self {
        let ihl = bytes[0] & 0x0F;
        let options_len = (ihl - 5) as usize;
        // Self::verify_checksum(&bytes);

        let src_ip = [bytes[12], bytes[13], bytes[14], bytes[15]];
        let dst_ip = [bytes[16], bytes[17], bytes[18], bytes[19]];
        let protocol = bytes[9];
        let payload = bytes[(20 + options_len)..].to_owned();

        let pseudo_header = PseudoHeader {
            src_ip,
            dst_ip,
            protocol,
            length: [0, 0],
        };

        let protocol = Self::get_protocol(protocol);
        let payload = Self::build_payload(protocol, pseudo_header, &payload);

        Self {
            header: Header {
                version_ihl: bytes[0],
                dscp_ecn: bytes[1],
                total_length: [bytes[2], bytes[3]],
                identification: [bytes[4], bytes[5]],
                flags_fragment_offset: [bytes[6], bytes[7]],
                ttl: bytes[8],
                protocol: bytes[9],
                checksum: [bytes[10], bytes[11]],
                src_ip,
                dst_ip,
                options: bytes[20..(20 + options_len)].to_owned(),
            },
            pseudo_header,
            payload: Arc::new(payload),
        }
    }

    fn build_payload(protocol: Protocol, pseudo_header: PseudoHeader, bytes: &[u8]) -> Box<dyn Packet + Send + Sync> {
        match protocol {
            Protocol::TCP => {
                Box::new(Tcp::new(bytes, pseudo_header))
            }
            Protocol::UDP => {
                Box::new(Udp::new(bytes, pseudo_header))
            }
            Protocol::ICMP => {
                Box::new(Icmp::new(bytes))
            }
            Protocol::UNKNOWN => {
                Box::new(Udp::new(bytes, pseudo_header))
            }
        }
    }

    pub fn protocol(&self) -> Protocol {
        return match self.header.protocol {
            1 => Protocol::ICMP,
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            _ => Protocol::UNKNOWN
        };
    }

    pub fn get_protocol(byte: u8) -> Protocol {
        return match byte {
            1 => Protocol::ICMP,
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            _ => Protocol::UNKNOWN
        };
    }

    pub fn verify_checksum(header: &Vec<u8>) -> bool {
        let checksum = Self::calc_checksum(header);
        return if header[10] != 0 || header[11] != 0 {
            bytes_to_u32(&checksum) == 0
        } else {
            false
        };
    }

    pub fn calc_checksum(header: &[u8]) -> [u8; 2] {
        let mut binary_u16_segments = vec![];
        // Merge two u8 to u16
        for i in (0..header.len()).step_by(2) {
            let segment = (header[i] as u16) * 256 + header[i + 1] as u16;
            binary_u16_segments.push(segment);
        }

        // Calculate the checksum
        let mut checksum = 0;
        for segment in binary_u16_segments {
            let sum: u32 = checksum as u32 + segment as u32;
            // handle overflow
            if sum > 0xFFFF {
                checksum = ((sum & 0xFFFF) + 1) as u16
            } else {
                checksum = sum as u16
            }
        }

        // Bitwise and to bytes
        (!checksum).to_be_bytes()
    }

    pub fn resp_header(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        let header = &self.header;

        packet.extend_from_slice(&[header.version_ihl, header.dscp_ecn, 0, 0]);
        packet.extend_from_slice(&[0, 0, header.flags_fragment_offset[0], header.flags_fragment_offset[1]]);
        packet.extend_from_slice(&[header.ttl, header.protocol, 0, 0]);
        packet.extend_from_slice(&header.dst_ip);
        packet.extend_from_slice(&header.src_ip);
        packet.extend_from_slice(&header.options);

        packet
    }

    pub fn pack(header: &[u8], payload: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.extend_from_slice(header);
        packet.extend_from_slice(payload);

        // Set total length
        let length = (packet.len() as u16).to_be_bytes();
        (packet[2], packet[3]) = (length[0], length[1]);

        // Set checksum
        let checksum = Self::calc_checksum(&packet[..(packet.len() - payload.len())]);
        (packet[10], packet[11]) = (checksum[0], checksum[1]);

        // Packet information(LittleEndian): flags(u16) and protocol(u16)
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            packet.insert(0, 0);
            packet.insert(1, 0);
            packet.insert(2, 0);
            packet.insert(3, 2); // IPv4
        }

        packet
    }

    pub fn resp_pack(&self, payload: &[u8]) -> Vec<u8> {
        let header = self.resp_header();
        Self::pack(&header, payload)
    }

    pub fn name(&self) -> String {
        let protocol = self.protocol();
        let src_addr = self.payload.src_addr();
        let dst_addr = self.payload.dst_addr();
        format!("{:?}[{}]=>[{}]", protocol, src_addr, dst_addr)
    }

    pub fn update_seq(&mut self, len: u32) {
        // self.payload.update_seq(0);
    }
}

impl PseudoHeader {
    pub fn to_be_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.src_ip);
        bytes.extend_from_slice(&self.dst_ip);
        bytes.push(0);
        bytes.push(self.protocol);
        bytes.extend_from_slice(&self.length);
        // <[u8; 12]>::try_from(bytes).unwrap()
        bytes
    }
}

#[derive(Debug)]
pub enum Protocol {
    TCP,
    UDP,
    ICMP,
    UNKNOWN,
}

pub trait Packet {
    fn protocol(&self) -> Protocol;
    fn src_addr(&self) -> SocketAddr { SocketAddr::new([0, 0, 0, 0].into(), 0) }
    fn dst_addr(&self) -> SocketAddr { SocketAddr::new([0, 0, 0, 0].into(), 0) }
    fn payload(&self) -> &Vec<u8>;
    fn flags_type(&self) -> FlagsType { return FlagsType(0); }
    fn info(&self) -> String;
    fn pack(&self, options: &[u8], payload: &[u8]) -> Vec<u8>;
    fn update_seq(&mut self, seq: u32) {}
    // fn handle(&self, ip_packet: &[u8], f: &mut File, logging: &mut Logging, x: T) -> Result<usize>;
}

// fn new(pkt: &[u8], pseudo_header: PseudoHeader) -> Self;
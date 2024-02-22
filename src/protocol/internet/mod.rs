pub mod tcp;
pub mod udp;

/**
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
    // payload: [u8],
    pub payload: Vec<u8>,
}

pub struct Header {
    pub version_ihl: u8, pub dscp_ecn: u8, pub total_length: u16,
    pub identification: u16, pub flags_fragment_offset: u16,
    pub ttl: u8, pub protocol: u8, pub checksum: u16,
    pub src_ip: [u8; 4], pub dst_ip: [u8; 4],
    // options: [u8],
    pub options: Vec<u8>,
}

#[derive(Debug)]
pub enum Protocol{
    TCP,
    UDP,
    ICMP,
    UNKNOWN,
}

impl Datagram {
    pub fn new(bytes: &[u8]) -> Self {
        let ihl = bytes[0] & 0x0F;
        let options_len = (ihl - 5) as usize;
        // Self::verify_checksum(&bytes);

        Self {
            header: Header {
                version_ihl: bytes[0],
                dscp_ecn: bytes[1],
                total_length: (bytes[2] << 8 + bytes[3]) as u16,
                identification: (bytes[4] << 8 + bytes[5]) as u16,
                flags_fragment_offset: (bytes[6] << 8 + bytes[7]) as u16,
                ttl: bytes[8],
                protocol: bytes[9],
                checksum: (bytes[10] << 8 + bytes[11]) as u16,
                src_ip: [bytes[12], bytes[13], bytes[14], bytes[15]],
                dst_ip: [bytes[16], bytes[17], bytes[18], bytes[19]],
                options: bytes[20..(20 + options_len)].to_owned(),
            },
            payload: bytes[(20 + options_len)..].to_owned(),
        }
    }

    pub fn protocol(&self) -> Protocol{
        return match self.header.protocol {
            1 => Protocol::ICMP,
            6 => Protocol::TCP,
            17 => Protocol::UDP,
            _ => Protocol::UNKNOWN
        };
    }

    pub fn verify_checksum(header: &Vec<u8>) -> bool {
        let checksum = Self::calc_checksum(header);
        return if header[10] != 0 || header[11] != 0 {
            checksum == 0
        } else {
            false
        };
    }

    pub fn calc_checksum(header: &Vec<u8>) -> u16 {
        let mut binary_u16_segments = vec![];
        // Merge two u8 to u16
        for i in (0..header.len()).step_by(2) {
            let segment = (header[i] as u16) * 256 + (header[i + 1]) as u16;
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

        // Bitwise
        !checksum
    }
}

struct DatagramDetail {
    version: u8,
    ihl: u8,
    tos: u8,
    total_length: u16,
    identification: u16,
    flags: u8,
    fragment_offset: u32,
    ttl: u8,
    protocol: u8,
    header_checksum: u16,
    options: Vec<u8>,
}

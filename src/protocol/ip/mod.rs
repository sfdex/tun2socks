mod tcp;
mod udp;

pub struct Datagram {
    header: IpHeader,
    // payload: [u8],
    payload: Vec<u8>,
}

struct IpHeader {
    version_ihl: u8,
    dscp_ecn: u8,
    total_length: u16,
    identification: u16,
    flags_fragment_offset: u16,
    ttl: u8,
    protocol: u8,
    header_checksum: u16,
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    // options: [u8],
    options: Vec<u8>,
}

impl Datagram {
    pub fn new(bytes: Vec<u8>) -> Self {
        let ihl = bytes[0] & 0x0F;
        let options_len = (ihl - 5) as usize;
        Self::verify_checksum(&bytes);

        Self {
            header: IpHeader {
                version_ihl: bytes[0],
                dscp_ecn: bytes[1],
                total_length: (bytes[2] << 8 + bytes[3]) as u16,
                identification: (bytes[4] << 8 + bytes[5]) as u16,
                flags_fragment_offset: (bytes[6] << 8 + bytes[7]) as u16,
                ttl: bytes[8],
                protocol: bytes[9],
                header_checksum: (bytes[10] << 8 + bytes[11]) as u16,
                src_ip: [bytes[12], bytes[13], bytes[14], bytes[15]],
                dst_ip: [bytes[16], bytes[17], bytes[18], bytes[19]],
                options: bytes[20..(20 + options_len)].to_owned(),
            },
            payload: bytes[(20 + options_len)..].to_owned(),
        }
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

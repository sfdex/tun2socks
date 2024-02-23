use std::fs::File;
use std::net::IpAddr;
use crate::dispatcher::direct::dial_tcp;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol, tcp};
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::tcp::ControlType::*;
use crate::util::{bytes_to_u32, bytes_to_u32_no_prefix};

pub mod direct;

pub fn dispatch(data: Vec<u8>, stream: &mut File, logging: &mut Logging) {
    let datagram = Datagram::new(&data);
    let ip_header = &datagram.header;

    // Fragment
    let id = bytes_to_u32(&ip_header.identification);
    let mf = (ip_header.flags_fragment_offset[0] >> 5) & 1;
    let offset = bytes_to_u32_no_prefix(&ip_header.flags_fragment_offset, 3);

    logging.i(format!("{:?}: {:?} => {:?}, IHL({}), ID({id}, MF({mf}), OFFSET({offset})",
                      &datagram.protocol(),
                      IpAddr::from(ip_header.src_ip),
                      IpAddr::from(ip_header.dst_ip),
                      ip_header.version_ihl >> 4,
    ));

    match &datagram.protocol() {
        Protocol::TCP => {
            let tcp = Tcp::new(&datagram.payload);
            match tcp.control_type() {
                SYN => {
                    let result = datagram.write(stream, &tcp.pack(0b010010));
                }
                PUSH => {}
                _ => {}
            };
            let addr = IpAddr::from(ip_header.dst_ip);
            let port = bytes_to_u32(&tcp.header.dst_port) as u16;
            let result = dial_tcp(addr, port, &[1u8]);
        }
        Protocol::UDP => {
            logging.e("Unsupported udp protocol".to_string())
        }
        Protocol::ICMP => {
            logging.e("Unsupported ICMP protocol".to_string())
        }
        Protocol::UNKNOWN => {
            logging.e("Unsupported unknown protocol".to_string())
        }
    };
}
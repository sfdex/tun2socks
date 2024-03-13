use crate::logging::Logging;
use crate::protocol::internet::PseudoHeader;
use crate::protocol::internet::tcp::ControlType::{PUSH, SYN};
use crate::protocol::internet::tcp::Tcp;

pub struct Simulator;

impl Simulator {
    pub fn handle_tcp (tcp: &Tcp, id: u32, pseudo_header: &mut PseudoHeader, logging: &mut Logging) -> Vec<Vec<u8>>{
        return match tcp.control_type() {
            SYN => {
                let response = tcp.pack(id, 0b010010, vec![], pseudo_header);
                vec![response]
            }
            PUSH => {
                let msg = &tcp.payload;
                let mut data = "Hello ".as_bytes().to_vec();
                data.extend_from_slice(msg);
                let ack_response = tcp.pack(id, 0b10000, vec![], pseudo_header);
                let msg_response = tcp.pack(id, 0b11000, data, pseudo_header);
                vec![ack_response, msg_response]
            }
            _ => {
                vec![]
            }
        };
        // let addr = IpAddr::from(ip_header.dst_ip);
        // let port = bytes_to_u32(&tcp.header.dst_port) as u16;
        // let result = dial_tcp(addr, port, &[1u8]);
    }
}
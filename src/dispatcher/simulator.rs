use crate::protocol::internet::{Packet, Protocol};
use crate::protocol::internet::icmp::Icmp;
use crate::protocol::internet::tcp::*;
use crate::protocol::internet::udp::Udp;

pub struct Simulator;

impl Simulator {
    pub fn handle(protocol: &Protocol, packet: &Box<dyn Packet>) -> Vec<Vec<u8>> {
        return match protocol {
            Protocol::TCP => { Self::handle_tcp(packet) }
            Protocol::UDP => { Self::handle_udp(packet) }
            Protocol::ICMP => { Self::handle_icmp(packet) }
            Protocol::UNKNOWN => { vec![vec![]] }
        };
    }

    fn handle_tcp(tcp: &Box<dyn Packet>) -> Vec<Vec<u8>> {
        return match tcp.flags_type() {
            SYN | SEW => {
                let response = tcp.pack(&[*SYN_ACK], &vec![]);
                vec![response]
            }
            PSH_ACK => {
                let msg = tcp.payload();
                let mut data = "Hello ".as_bytes().to_vec();
                data.extend_from_slice(msg);
                let ack_response = tcp.pack(&[*ACK], &vec![]);
                let msg_response = tcp.pack(&[*PSH_ACK], &data);
                vec![ack_response, msg_response]
            }
            FIN => {
                vec![]
            }
            FIN_ACK => {
                let ack_response = tcp.pack(&[*ACK], &vec![]);
                let fin_ack_response = tcp.pack(&[*FIN_ACK], &vec![]);
                vec![ack_response, fin_ack_response]
            }
            RST => {
                vec![]
            }
            RST_ACK => {
                vec![]
            }
            _ => {
                vec![]
            }
        };
        // let addr = IpAddr::from(ip_header.dst_ip);
        // let port = bytes_to_u32(&tcp.header.dst_port) as u16;
        // let result = dial_tcp(addr, port, &[1u8]);
    }

    fn handle_udp(udp: &Box<dyn Packet>) -> Vec<Vec<u8>> {
        let mut msg = Vec::new();
        msg.extend_from_slice("Hello ".as_bytes());
        msg.extend_from_slice(&udp.payload());
        let response = udp.pack(&[], &msg);
        vec![response]
    }

    fn handle_icmp(icmp: &Box<dyn Packet>) -> Vec<Vec<u8>> {
        let response = icmp.pack(&[], &vec![]);
        vec![response]
    }
}
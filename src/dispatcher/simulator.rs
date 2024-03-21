use crate::protocol::internet::{Packet, Protocol};
use crate::protocol::internet::tcp::*;

pub struct Simulator;

type Pkt = Box<dyn Packet + Send + Sync>;
impl Simulator {
    pub fn handle(protocol: &Protocol, packet: &Pkt) -> Vec<Vec<u8>> {
        return match protocol {
            Protocol::TCP => { Self::handle_tcp(packet) }
            Protocol::UDP => { Self::handle_udp(packet) }
            Protocol::ICMP => { Self::handle_icmp(packet) }
            Protocol::UNKNOWN => { vec![vec![]] }
        };
    }

    pub fn handle_tcp(tcp: &Pkt) -> Vec<Vec<u8>> {
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
    }

    pub fn handle_udp(udp: &Pkt) -> Vec<Vec<u8>> {
        let mut msg = Vec::new();
        msg.extend_from_slice("Hello ".as_bytes());
        msg.extend_from_slice(&udp.payload());
        // let response = udp.pack(&[], &msg);
        vec![msg]
    }

    fn handle_icmp(icmp: &Pkt) -> Vec<Vec<u8>> {
        let response = icmp.pack(&[], &vec![]);
        vec![response]
    }
}
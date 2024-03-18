use crate::dispatcher::simulator::Simulator;
use crate::protocol::internet::{Datagram, Packet, Protocol, PseudoHeader};
use crate::protocol::internet::icmp::Icmp;
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::udp::Udp;
use crate::thread_pool::{Message, Reporter};
use crate::thread_pool::state::{State, TcpState, UdpState};

pub struct Handler {
    pub id: usize,
    pub name: String,
    pub reporter: Reporter,
    pub protocol: Protocol,
    pub datagram: Option<Datagram>,
    pub payload: Option<Box<dyn Packet>>,
}

impl Handler {
    pub fn new(id: usize, reporter: Reporter) -> Self {
        Self {
            id,
            name: String::new(),
            reporter,
            protocol: Protocol::UNKNOWN,
            datagram: None,
            payload: None,
        }
    }

    fn build_packet(&self) -> Box<dyn Packet> {
        let payload = &self.datagram;
        if let Some(p) = payload {
            let bytes = &p.payload;
            let pseudo_header = PseudoHeader {
                src_ip: p.header.src_ip,
                dst_ip: p.header.dst_ip,
                protocol: p.header.protocol,
                length: [0, 0],
            };
            match &self.protocol {
                Protocol::TCP => {
                    Box::new(Tcp::new(bytes, pseudo_header))
                }
                Protocol::UDP => {
                    Box::new(Udp::new(bytes, pseudo_header))
                }
                Protocol::ICMP => {
                    Box::new(Icmp::new(bytes))
                }
                // Protocol::UNKNOWN => {}
                _ => {
                    Box::new(Udp::new(bytes, pseudo_header))
                }
            }
        } else {
            Box::new(Icmp::new(&vec![]))
        }
    }

    fn pack(&self, payload: Message) -> Message {
        if let Some(datagram) = &self.datagram {
            datagram.pack(&payload)
        } else {
            Message::new()
        }
    }

    pub fn handle(&mut self, datagram: Datagram) {
        self.protocol = datagram.protocol();
        self.name = datagram.name();
        self.datagram = Some(datagram);
        self.payload = Some(self.build_packet());

        match self.protocol {
            Protocol::TCP => {
                self.handle_tcp();
            }
            Protocol::UDP => {
                // self.handle_udp();
            }
            Protocol::ICMP => {}
            Protocol::UNKNOWN => {}
        }
    }

    fn handle_tcp(&mut self) {
        if let Some(pkt) = &self.payload {
            println!("handle tcp: id = {}", self.id);
            let response = Simulator::handle_tcp(&pkt);
            for resp in response {
                self.reporter.lock().unwrap().send((usize::MAX, State::MESSAGE(self.pack(resp)))).unwrap();
                self.reporter.lock().unwrap().send((self.id, State::TCP { name: self.name.to_string(), state: TcpState::SynAckWait })).unwrap();
            }
        }
    }

    fn handle_udp(&mut self) {
        if let Some(pkt) = &self.payload {
            let response = Simulator::handle_udp(&pkt);
            for resp in response {
                self.reporter.lock().unwrap().send((usize::MAX, State::MESSAGE(resp))).unwrap();
                self.reporter.lock().unwrap().send((self.id, State::UDP { name: self.name.to_string(), state: UdpState::Communication })).unwrap();
            }
        }
    }
}
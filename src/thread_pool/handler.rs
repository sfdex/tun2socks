use std::net::{TcpStream, UdpSocket};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::usize;

use crate::protocol::internet::{Datagram, Packet, Protocol};
use crate::thread_pool::event::Event;
use crate::thread_pool::Reporter;

pub struct Handler {
    pub id: usize,
    pub reporter: Reporter,
    pub protocol: Protocol,
    pub datagram: Option<Arc<Datagram>>,
    pub payload: Option<Box<dyn Packet + Send + Sync>>,
    pub tcp: Option<TcpStream>,
    pub udp: Option<UdpSocket>,
    pub job: Option<JoinHandle<()>>,
}

impl Handler {
    pub fn new(id: usize, reporter: Reporter) -> Self {
        Self {
            id,
            reporter,
            protocol: Protocol::UNKNOWN,
            datagram: None,
            payload: None,
            job: None,
            tcp: None,
            udp: None,
        }
    }

    pub fn handle(&mut self, datagram: Arc<Datagram>) {
        self.protocol = datagram.protocol();
        self.datagram = Some(datagram);

        match self.protocol {
            Protocol::TCP => {
                self.handle_tcp();
            }
            Protocol::UDP => {
                self.handle_udp();
            }
            Protocol::ICMP => {}
            Protocol::UNKNOWN => {}
        }
    }

    pub fn stop(&mut self) {
        if let Some(tcp) = &self.tcp {
            tcp.shutdown(std::net::Shutdown::Both).unwrap();
        }

        self.tcp = None;
        self.udp = None;
        self.job = None;
    }

    pub fn report(&self, state: Event) {
        state.report(self.id, &self.reporter);
    }
}
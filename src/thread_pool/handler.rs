use std::net::{TcpStream, UdpSocket};
use std::sync::{Arc, mpsc, Mutex};
use std::thread::JoinHandle;
use std::usize;

use crate::protocol::internet::{Datagram, Packet, Protocol};
use crate::thread_pool::event::Event;
use crate::thread_pool::Reporter;

pub struct Handler {
    pub id: usize,
    pub name: String,
    pub reporter: Reporter,
    pub protocol: Protocol,
    pub datagram: Option<Arc<Datagram>>,
    pub payload: Option<Box<dyn Packet + Send>>,
    pub tcp: Option<TcpStream>,
    pub udp: Option<UdpSocket>,
    pub job: Option<JoinHandle<()>>,
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
            job: None,
            tcp: None,
            udp: None,
        }
    }

    pub fn handle(&mut self, datagram: Arc<Datagram>) {
        self.protocol = datagram.protocol();
        self.name = datagram.name();
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

    pub fn report(&self, state: Event) {
        state.report(self.id, &self.reporter);
    }
}
use std::net::{Shutdown, TcpStream, UdpSocket};
use std::thread::JoinHandle;
use std::usize;
use crate::log;

use crate::protocol::internet::{Payload, Protocol};
use crate::thread_pool::event::Event;
use crate::thread_pool::event::Event::LOG;
use crate::thread_pool::Reporter;

pub struct Handler {
    pub id: usize,
    pub reporter: Reporter,
    pub payload: Option<Payload>,
    pub tcp: Option<TcpStream>,
    pub udp: Option<UdpSocket>,
    pub job: Option<JoinHandle<()>>,
}

impl Handler {
    pub fn new(id: usize, reporter: Reporter) -> Self {
        Self {
            id,
            reporter,
            payload: None,
            job: None,
            tcp: None,
            udp: None,
        }
    }

    pub fn handle(&mut self, payload: Payload) {
        let protocol = payload.protocol();
        self.payload = Some(payload);

        match protocol {
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

    pub fn stop(self) {
        if let Some(tcp) = self.tcp {
            match tcp.shutdown(Shutdown::Both) {
                Ok(_) => {}
                Err(err) => {
                    println!("Shut down tcp error: {}", err);
                }
            }
            drop(tcp);
        } else if let Some(udp) = self.udp {
            drop(udp);
        }

        if let Some(job) = self.job {
            drop(job);
        }

        let r = &self.reporter;
        r.send((self.id, log!("Yes"))).unwrap_or(());
        drop(self.reporter);
    }
}
use std::{thread, usize};
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::sync::{Arc, mpsc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::log;
use crate::protocol::internet::{Datagram, Packet, Protocol};
use crate::protocol::internet::tcp::*;
use crate::thread_pool::{Message, Reporter};
use crate::thread_pool::Event::*;
use crate::thread_pool::event::{Event, TcpState};

pub struct Handler {
    pub id: usize,
    pub name: String,
    pub reporter: Reporter,
    pub protocol: Protocol,
    pub datagram: Option<Arc<Datagram>>,
    pub payload: Option<Box<dyn Packet + Send>>,
    tcp: Option<TcpStream>,
    udp: Option<UdpSocket>,
    job: Option<JoinHandle<()>>,
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

    fn pack(&self, payload: Message) -> Message {
        if let Some(datagram) = &self.datagram {
            datagram.resp_pack(&payload)
        } else {
            Message::new()
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

    fn handle_tcp(&mut self) {
        let id = self.id;
        let name = self.name.to_string();
        let datagram = if let Some(pkt) = &self.datagram {
            pkt
        } else {
            return;
        };

        let payload = &datagram.payload;
        let data = payload.payload();
        let dst_addr = payload.dst_addr();
        if let Some(stream) = &mut self.tcp {
            match stream.write_all(data) {
                Ok(_) => {
                    self.report(LOG("Send to remote success".into()));
                    self.report(MESSAGE(*ACK, vec![]));
                }
                Err(e) => {
                    self.report(log!("Send data error: bye bye, {:?}", e));
                    self.report(MESSAGE(*RST, vec![]));
                    self.report(IDLE);
                }
            }
            return;
        }

        self.report(log!("{id}->{name}: connect to {addr}", addr = dst_addr));
        let stream = match TcpStream::connect_timeout(&payload.dst_addr(), Duration::from_secs(5)) {
            Ok(stream) => {
                log!("{id}->{name}: success connect to server").report(id, &self.reporter);
                stream
            }
            Err(err) => {
                self.report(log!("{id}->{name}: failed connect: {e:#?}", e = err));
                self.report(MESSAGE(*RST, vec![]));
                self.report(IDLE);
                return;
            }
        };

        self.report(MESSAGE(*SYN_ACK, vec![]));
        self.report(TCP(self.name.to_string(), TcpState::SynAckWait));

        let reporter = Arc::clone(&self.reporter);

        let mut stream_cloned = stream.try_clone().unwrap();
        self.tcp = Some(stream);

        // Receive message
        let job = thread::spawn(move || {
            let mut buf = vec![0; 1500];
            loop {
                match stream_cloned.read(&mut buf) {
                    Ok(n) => {
                        if n == 0 {
                            log!("{id}->{name}: reach end").report(id, &reporter);
                            MESSAGE(*RST, vec![]).report(id, &reporter);
                            break;
                        }
                        log!("<<---{id}->{name}: recv {n} bytes\n\t{:?}", &buf[..n]).report(id, &reporter);
                        MESSAGE(0, buf[..n].to_vec()).report(id, &reporter);
                    }
                    Err(e) => {
                        log!("{id}->{name}: {:#?}", e).report(id, &reporter);
                        MESSAGE(*RST, vec![]).report(id, &reporter);
                        break;
                    }
                }
            }
            IDLE.report(id, &reporter);
        });

        self.job = Some(job);
    }

    fn report_event(reporter: &Arc<Mutex<mpsc::Sender<(usize, Event)>>>, id: usize, event: Event) {
        reporter.lock().unwrap().send((id, event)).unwrap()
    }

    fn handle_udp(&mut self) {
        let datagram = if let Some(datagram) = &self.datagram {
            datagram
        } else {
            return;
        };
        let payload = &datagram.payload;
        let data = payload.payload();
        let id = self.id;
        let name = self.name.to_string();
        let dst_addr = payload.dst_addr();

        let reporter = Arc::clone(&self.reporter);

        if let Some(udp) = &self.udp {
            match udp.send(data) {
                Ok(n) => {
                    self.report(log!("{id}->{name} sent {n} bytes"));
                }
                Err(err) => {
                    self.report(log!("{id}->{name} sent error: {:#?}", err));
                    self.report(IDLE);
                }
            }

            return;
        }

        let udp = match UdpSocket::bind("10.0.0.1:8989") {
            Ok(udp_socket) => {
                self.report(log!("{id}->{name} bind success"));
                udp_socket
            }
            Err(err) => {
                self.report(log!("{id}->{name} bind failed: {:#?}", err));
                return;
            }
        };

        match udp.connect(dst_addr) {
            Ok(_) => {
                self.report(log!("{id}->{name} connect to server success"))
            }
            Err(err) => {
                self.report(log!("{id}->{name}: udp connect to server error: {:#?}", err));
                self.report(IDLE);
            }
        }

        match udp.send(data) {
            Ok(n) => {
                self.report(log!("{id}->{name} sent {n} bytes"));
            }
            Err(err) => {
                self.report(log!("{id}->{name} sent error: {:#?}", err));
                self.report(IDLE);
                return;
            }
        }

        let udp_cloned = udp.try_clone().unwrap();
        self.udp = Some(udp);

        let job = thread::spawn(move || {
            loop {
                let mut buf = vec![0; 1500];
                log!("{id}->{name}: udp recv start").report(id, &reporter);

                match udp_cloned.recv(&mut buf) {
                    Ok(n) => {
                        // match udp.recv_from(&mut buf) {
                        //     Ok((n, addr)) => {
                        log!("{id}->{name}: udp recv {n} bytes, content: {}", String::from_utf8_lossy(&buf[..n])).report(id, &reporter);
                        MESSAGE(0, buf[..n].to_vec()).report(id, &reporter);
                    }
                    Err(err) => {
                        log!("{id}->{name}: udp recv error: {:#?}", err).report(id, &reporter);
                        IDLE.report(id, &reporter);
                        break;
                    }
                }
            }
            log!("{id}->{name}: udp recv end").report(id, &reporter);
        });

        self.job = Some(job);
    }

    fn report(&self, state: Event) {
        state.report(self.id, &self.reporter);
    }
}
use std::{thread, usize};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, mpsc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use crate::log;

use crate::protocol::internet::{Datagram, Packet, Protocol};
use crate::protocol::internet::tcp::*;
use crate::thread_pool::{Message, Reporter};
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
        // self.payload = Some(self.payload.unwrap());
        // self.payload = Some(self.build_packet());

        match self.protocol {
            Protocol::TCP => {
                // self.handle_tcp();
            }
            Protocol::UDP => {
                self.handle_udp();
            }
            Protocol::ICMP => {}
            Protocol::UNKNOWN => {}
        }
    }

    fn handle_tcp(&mut self) {
        println!("handle tcp: id = {}", self.id);
        let id = self.id;
        let name = self.name.to_string();
        if let Some(pkt) = &self.datagram {
            let payload = &pkt.payload;
            let data = payload.payload();
            let dst_addr = payload.dst_addr();
            if let Some(writer) = &mut self.tcp {
                match writer.write_all(data) {
                    Ok(_) => {
                        self.report(Event::LOG("Write to remote success".into()));
                        self.report(Event::MESSAGE(*ACK, vec![]));
                    }
                    Err(e) => {
                        self.report(log!("write data error: bye bye, {:?}", e));
                        self.report(Event::MESSAGE(*RST, vec![]));
                        self.report(Event::IDLE);
                    }
                }
                return;
            }

            self.report(log!("{id}->{name}: connect to {dst_addr}", dst_addr = dst_addr));
            let ret = TcpStream::connect_timeout(&payload.dst_addr(), std::time::Duration::from_secs(5));
            if let Err(e) = &ret {
                self.report(log!("{id}->{name}: connect to {dst_addr} error: {e:#?}", e = e));
                self.report(Event::MESSAGE(*RST, vec![]));
                self.report(Event::IDLE);
                return;
            }

            log!("{id}->{name}: connect to {dst_addr} success").report(id, &self.reporter);

            let tcp_state = Event::TCP(self.name.to_string(), TcpState::SynAckWait);
            self.report(tcp_state);

            self.report(Event::MESSAGE(*SYN_ACK, vec![]));

            let reporter = Arc::clone(&self.reporter);

            let stream = ret.unwrap();
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
                                break;
                            }
                            Event::MESSAGE(0, buf[..n].to_vec()).report(id, &reporter);
                        }
                        Err(e) => {
                            log!("{id}->{name}: {:#?}", e).report(id, &reporter);
                            Event::MESSAGE(*RST, vec![]).report(id, &reporter);
                            Event::IDLE.report(id, &reporter);
                            break;
                        }
                    }
                }
                reporter.lock().unwrap().send((id, Event::IDLE)).unwrap();
            });

            self.job = Some(job);
        }
    }

    fn report_event(reporter: &Arc<Mutex<mpsc::Sender<(usize, Event)>>>, id: usize, event: Event) {
        reporter.lock().unwrap().send((id, event)).unwrap()
    }

    fn handle_udp(&mut self) {
        if let Some(datagram) = &self.datagram {
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
                        self.report(Event::IDLE);
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
                    self.report(Event::IDLE);
                }
            }

            match udp.send(data) {
                Ok(n) => {
                    self.report(log!("{id}->{name} sent {n} bytes"));
                }
                Err(err) => {
                    self.report(log!("{id}->{name} sent error: {:#?}", err));
                    self.report(Event::IDLE);
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
                            Event::MESSAGE(0, buf[..n].to_vec()).report(id, &reporter);
                        }
                        Err(err) => {
                            log!("{id}->{name}: udp recv error: {:#?}", err).report(id, &reporter);
                            Event::IDLE.report(id, &reporter);
                            break;
                        }
                    }
                }
                log!("{id}->{name}: udp recv end").report(id, &reporter);
            });
            
            self.job = Some(job);
        }
    }

    fn report(&self, state: Event) {
        state.report(self.id, &self.reporter);
    }
}
use std::{thread, usize};
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::sync::{Arc, mpsc, Mutex};
use std::thread::JoinHandle;

use crate::protocol::internet::{Datagram, Packet, Protocol, tcp};
use crate::protocol::internet::tcp::*;
use crate::thread_pool::{Message, Reporter, Sender};
use crate::thread_pool::state::{Event, TcpState};

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
        if let Some(pkt) = &self.payload {
            let data = pkt.payload();
            if let Some(writer) = &mut self.tcp {
                match writer.write_all(data) {
                    Ok(_) => {
                        self.report_state(Event::LOG("Write to remote success".into()));
                        self.report_state(Event::MESSAGE(*ACK, vec![]));
                    }
                    Err(e) => {
                        self.report_state(Event::LOG(format!("write data error: bye bye, {:?}", e)));
                        self.report_state(Event::MESSAGE(*RST, vec![]));
                        self.report_state(Event::IDLE);
                    }
                }
                return;
            }

            self.report_state(Event::LOG(format!("handle tcp: id = {}", self.id)));
            let ret = TcpStream::connect_timeout(&pkt.dst_addr(), std::time::Duration::from_secs(5));
            if let Err(e) = &ret {
                self.report_state(Event::LOG(format!("write data error: bye bye, {:#?}", e)));
                self.report_state(Event::MESSAGE(*RST, vec![]));
                self.report_state(Event::IDLE);
                return;
            }

            let tcp_state = Event::TCP(self.name.to_string(), TcpState::SynAckWait);
            self.report_state(tcp_state);

            self.report_state(Event::MESSAGE(*SYN_ACK, vec![]));

            let reporter = Arc::clone(&self.reporter);
            let id = self.id;
            let name = self.name.to_string();

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
                                Self::report_event(&reporter, id, Event::LOG(format!("{id}->{name}: reach end")));
                                break;
                            }
                            Self::report_event(&reporter, id, Event::MESSAGE(0, buf[..n].to_vec()));
                        }
                        Err(e) => {
                            Self::report_event(&reporter, id, Event::LOG(format!("{id}->{name}: {:#?}", e)));
                            Self::report_event(&reporter, id, Event::MESSAGE(*RST, vec![]));
                            Self::report_event(&reporter, id, Event::IDLE);
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
            let pkt = &datagram.payload;
            let payload = pkt.payload();
            let reporter = Arc::clone(&self.reporter);
            let id = self.id;
            let name = self.name.to_string();

            if let Some(udp) = &self.udp {
                match udp.send(payload) {
                    Ok(n) => {
                        self.report_state(Event::LOG(format!("{id}->{name}: udp send {n} bytes")));
                    }
                    Err(err) => {
                        self.report_state(Event::LOG(format!("{id}->{name}: udp send error: {:#?}", err)));
                        self.report_state(Event::IDLE);
                    }
                }

                return;
            }

            let udp = UdpSocket::bind("10.0.0.1:8989").unwrap();
            match udp.connect(pkt.dst_addr()) {
                Ok(_) => {
                    self.report_state(Event::LOG(format!("{id}->{name}: udp connect success")))
                }
                Err(err) => {
                    self.report_state(Event::LOG(format!("{id}->{name}: udp connect error: {:#?}", err)));
                    self.report_state(Event::IDLE);
                }
            }

            match udp.send(payload) {
                Ok(n) => {
                    self.report_state(Event::LOG(format!("{id}->{name}: udp send {n} bytes")));
                }
                Err(err) => {
                    self.report_state(Event::LOG(format!("{id}->{name}: udp send error: {:#?}", err)));
                    self.report_state(Event::IDLE);
                    return;
                }
            }

            let udp_cloned = udp.try_clone().unwrap();
            self.udp = Some(udp);

            let job = thread::spawn(move || {
                loop {
                    let mut buf = vec![0; 1500];
                    match udp_cloned.recv_from(&mut buf) {
                        Ok((n, addr)) => {
                            Self::report_event(&reporter, id, Event::LOG(format!("{id}->{name}: udp recv {n} bytes from {addr}, content: {}", String::from_utf8_lossy(&buf[..n]))));
                            Self::report_event(&reporter, id, Event::MESSAGE(0, buf[..n].to_vec()));
                        }
                        Err(err) => {
                            Self::report_event(&reporter, id, Event::LOG(format!("{id}->{name}: udp recv error: {:#?}", err)));
                            Self::report_event(&reporter, id, Event::IDLE);
                            break;
                        }
                    }
                }
            });

            self.job = Some(job);
        }
    }

    fn report_state(&self, state: Event) {
        self.reporter.lock().unwrap().send((self.id, state)).unwrap();
    }
}
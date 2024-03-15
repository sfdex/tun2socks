use std::sync::{Arc, mpsc, Mutex};
use std::thread;
use crate::dispatcher::simulator::Simulator;
use crate::protocol::internet::{Datagram, Packet, Protocol, PseudoHeader};
use crate::protocol::internet::icmp::Icmp;
use crate::protocol::internet::Protocol::UNKNOWN;
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::udp::Udp;

struct ThreadPool {
    workers: Vec<Worker>,
}

type Message = Vec<u8>;
type Sender = Arc<Mutex<mpsc::Sender<Message>>>;
type Receiver = Arc<Mutex<mpsc::Receiver<Message>>>;

enum WorkerState {
    IDLE,
    RUNNING,
}


struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

struct Ckor {
    name: String,
    protocol: Protocol,
    datagram: Option<Datagram>,
    payload: Option<Box<dyn Packet>>,
}

impl Worker {
    pub fn new(id: usize, receiver: Receiver) -> Self {
        let thread = thread::spawn(move || {
            let mut ckor = Ckor {
                name: String::new(),
                protocol: UNKNOWN,
                datagram: None,
                payload: None,
            };
            
            loop {
                let msg = receiver.lock().unwrap().recv().unwrap();
                ckor.handle(&msg);
            }
        });
        println!("Worker {} is created", id);

        Self {
            id,
            thread,
        }
    }
}

impl Ckor {
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
    
    fn handle(&mut self, msg: &[u8]) {
        let datagram = Datagram::new(&msg);
        self.protocol = datagram.protocol();
        let payload = self.build_packet();
        let name = format!("{:?}({:?}{})", self.protocol, datagram.header.dst_ip, payload.flags_type());
        self.datagram = Some(datagram);
        self.payload = Some(payload);

        if let Some(pkt) = &self.payload {
            let response = Simulator::handle(&self.protocol, &pkt);
            for resp in response {
                
            }
        }
    }
}
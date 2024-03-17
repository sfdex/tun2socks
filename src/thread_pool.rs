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
    state_receiver: mpsc::Receiver<StateMessage>,
}

impl ThreadPool {
    fn new(size: usize) -> Self {
        let mut workers = Vec::with_capacity(size);
        let (reporter, state_receiver) = mpsc::channel();
        let reporter = Arc::new(Mutex::new(reporter));

        for i in 0..size {
            let (tx, rx) = mpsc::channel();
            let reporter = Arc::clone(&reporter);
            workers.push(Worker::new(i, reporter, tx, rx));
        }

        Self {
            workers,
            state_receiver,
        }
    }

    fn execute(&mut self, data: Message) {
        let datagram = Datagram::new(&data);
        let name = datagram.name();

        todo!("Find worker")
    }

    fn run(&mut self) {
        for worker_state in &self.state_receiver {
            let index = worker_state.0;
            let state = worker_state.1;

            if let State::MESSAGE(resp) = state {
                return;
            }

            self.workers[index].state = state;
        }
    }
}

struct Worker {
    id: usize,
    name: String,
    thread: Option<thread::JoinHandle<()>>,
    sender: Sender,
    state: State,
}

struct Ckor {
    name: String,
    protocol: Protocol,
    datagram: Option<Datagram>,
    payload: Option<Box<dyn Packet>>,
}

impl Worker {
    pub fn new(id: usize, reporter: Reporter, tx: Sender, rx: Receiver) -> Self {
        // pub fn set<F>(&mut self, id: usize, f: F, receiver: Receiver)
        //     where F: FnMut(Message) + Send + 'static
        let thread = thread::spawn(move || {
            let mut ckor = Ckor {
                name: String::new(),
                protocol: UNKNOWN,
                datagram: None,
                payload: None,
            };

            for msg in rx {
                ckor.handle(&msg, &reporter);
            }
        });

        Self {
            id,
            name: "".to_string(),
            thread: Some(thread),
            sender: tx,
            state: State::IDLE,
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

    fn handle(&mut self, msg: &[u8], sender: &Reporter) {
        let datagram = Datagram::new(&msg);
        self.protocol = datagram.protocol();
        let payload = self.build_packet();
        let name = format!("{:?}({:?}{})", self.protocol, datagram.header.dst_ip, payload.flags_type()).to_string();
        self.datagram = Some(datagram);
        self.payload = Some(payload);

        if let Some(pkt) = &self.payload {
            let response = Simulator::handle(&self.protocol, &pkt);
            for resp in response {
                let resp_state_msg = StateMessage(usize::MAX, State::MESSAGE(resp));
                sender.lock().unwrap().send(resp_state_msg).unwrap();
                let sack_wait_state_msg = StateMessage(usize::MAX, State::TCP { name: name.to_string(), state: TcpState::SynAckWait });
                sender.lock().unwrap().send(sack_wait_state_msg).unwrap();
            }
        }
    }
}


type Message = Vec<u8>;
type Reporter = Arc<Mutex<mpsc::Sender<StateMessage>>>;
type Sender = mpsc::Sender<Message>;
type Receiver = mpsc::Receiver<Message>;

struct StateMessage(usize, State);

enum State {
    MESSAGE(Message),
    TCP { name: String, state: TcpState },
    UDP { name: String, state: UdpState },
    ICMP { name: String, state: IcmpState },
    IDLE,
}

enum TcpState {
    SynAckWait,
    Communication,
    FinWait,
    RstWait,
    Destroy,
}

enum UdpState {
    Communication,
    Destroy,
}

enum IcmpState {
    Communication,
    Destroy,
}
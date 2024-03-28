use std::thread;
use crate::dispatcher::socks5;
use crate::protocol::internet::Datagram;
use crate::thread_pool::event::Event;
use crate::thread_pool::Sender;

pub struct Job {
    pub name: String,
    pub thread: thread::JoinHandle<()>,
    pub sender: Sender,
    pub state: Event,
    pub datagram: Datagram,
}

impl Job {
    pub fn new(name: &str, sender: Sender, datagram: Datagram) -> Self {
        let addr = datagram.payload.dst_addr();
        let mut socks5 = socks5::tcp_based::Client::new();
        let thread = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let _ = socks5.start();
                // do something
            }).unwrap();

        Self {
            name: name.to_string(),
            thread,
            sender,
            state: Event::IDLE,
            datagram,
        }
    }
}
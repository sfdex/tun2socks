use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

use crate::dispatcher::socks5;
use crate::protocol::internet::{Datagram, Payload};
use crate::thread_pool::{Reporter, Sender};
use crate::thread_pool::event::Event;

pub struct Job {
    pub name: String,
    pub thread: thread::JoinHandle<()>,
    pub sender: Sender,
    pub state: Event,
    pub datagram: Datagram,
}

impl Job {
    pub fn new(name: &str, reporter: Reporter, datagram: Datagram) -> Self {
        let payload = &datagram.payload;
        let mut socks5 = socks5::tcp_based::Client::new(
            SocketAddr::from_str("106.75.226.38:9900").unwrap(),
            &datagram.header.dst_ip.to_vec(),
            payload.dst_addr().port().to_be_bytes(),
            reporter,
        );

        let (sender, receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let result = socks5.start();
                if result.is_err() {
                    return;
                }

                for msg in receiver {
                    let payload: Payload = msg;
                    socks5.send(payload.payload()).unwrap_or(());
                }
                socks5.stop();
            }).unwrap();

        Self {
            name: name.to_string(),
            thread,
            sender,
            state: Event::IDLE,
            datagram,
        }
    }

    pub fn send(&self, payload: Payload) {
        self.sender.send(payload).unwrap_or(()); // Ignore error
    }
}
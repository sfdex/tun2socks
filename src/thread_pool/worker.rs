use std::sync::mpsc;
use std::thread;

use crate::protocol::internet::Datagram;
use crate::thread_pool::{Reporter, Sender};
use crate::thread_pool::event::Event;
use crate::thread_pool::handler::Handler;
use crate::tun::isRunning;

pub struct Worker {
    pub name: String,
    pub thread: thread::JoinHandle<()>,
    pub sender: Sender,
    pub state: Event,
    pub datagram: Option<Datagram>,
}

impl Worker {
    pub fn new(id: usize, reporter: Reporter) -> Self {
        let mut handler = Handler::new(id, reporter);
        let (tx, rx) = mpsc::channel();
        let thread = thread::spawn(move || {
            for payload in rx {
                if !isRunning() {
                    handler.stop();
                    break;
                }
                handler.handle(payload);
            }
        });

        Self {
            name: "".to_string(),
            thread,
            sender: tx,
            state: Event::IDLE,
            datagram: None,
        }
    }
}
use std::sync::mpsc;
use std::thread;

use crate::protocol::internet::Datagram;
use crate::thread_pool::{Reporter, Sender};
use crate::thread_pool::event::Event;
use crate::thread_pool::handler::Handler;

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
        let thread = thread::Builder::new()
            .name(format!("worker{id}"))
            .spawn(move || {
                for payload in rx {
                    handler.handle(payload);
                }
                handler.stop();
            }).unwrap();

        Self {
            name: "".to_string(),
            thread,
            sender: tx,
            state: Event::IDLE,
            datagram: None,
        }
    }
}
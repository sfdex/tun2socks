use std::sync::Arc;
use std::thread;
use crate::protocol::internet::Datagram;
use crate::thread_pool::{Receiver, Reporter, Sender};
use crate::thread_pool::handler::Handler;
use crate::thread_pool::event::Event;

pub struct Worker {
    pub name: String,
    thread: Option<thread::JoinHandle<()>>,
    pub sender: Sender,
    pub state: Event,
    pub datagram: Option<Arc<Datagram>>,
}

impl Worker {
    pub fn new(id: usize, reporter: Reporter, tx: Sender, rx: Receiver) -> Self {
        let thread = thread::spawn(move || {
            let mut handler = Handler::new(id, reporter);
            for msg in rx {
                handler.handle(msg);
            }
        });

        Self {
            name: "".to_string(),
            thread: Some(thread),
            sender: tx,
            state: Event::IDLE,
            datagram: None,
        }
    }
}
use std::sync::{Arc, Mutex};
use std::thread;

use crate::protocol::internet::Datagram;
use crate::thread_pool::{Receiver, Reporter, Sender};
use crate::thread_pool::event::Event;
use crate::thread_pool::handler::Handler;

pub struct Worker {
    pub name: String,
    pub thread: Option<thread::JoinHandle<()>>,
    pub sender: Sender,
    pub state: Event,
    pub handler: Arc<Mutex<Handler>>,
    pub datagram: Option<Arc<Datagram>>,
}

impl Worker {
    pub fn new(id: usize, reporter: Reporter, tx: Sender, rx: Receiver) -> Self {
        let handler = Handler::new(id, reporter);
        let handler = Arc::new(Mutex::new(handler));
        let handler_cloned = Arc::clone(&handler);
        let thread = thread::spawn(move || {
            for msg in rx {
                handler_cloned.lock().unwrap().handle(msg);
            }
        });

        Self {
            name: "".to_string(),
            thread: Some(thread),
            sender: tx,
            state: Event::IDLE,
            handler,
            datagram: None,
        }
    }

    pub fn stop(&mut self) {
        self.handler.lock().unwrap().stop();
        self.thread = None;
    }
}
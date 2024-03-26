use std::fs::File;
use std::io::Write;
use std::sync::{Arc, mpsc};

use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Payload};
use crate::thread_pool::event::Event;
use crate::thread_pool::worker::Worker;

mod worker;
pub mod event;
pub mod handler;

pub struct ThreadPool {}

impl ThreadPool {
    pub fn new(size: usize, reporter: Reporter) -> Self {
        for i in 0..size {
            let reporter = Arc::clone(&reporter);
            unsafe {
                WORKERS.push(Worker::new(i, reporter));
            }
        }

        Self {
            // writer,
            // state_receiver,
        }
    }

    pub fn execute(datagram: Datagram, logging: &mut Logging) {
        let name = datagram.name();
        unsafe {
            let mut index = 0;
            let mut matched_index = WORKERS.len();
            for (i, worker) in WORKERS.iter().enumerate() {
                if worker.name == name {
                    matched_index = i;
                    break;
                } else if worker.name == "" {
                    index = i;
                }
            }

            let payload = Arc::clone(&datagram.payload);

            let index = if matched_index != WORKERS.len() { matched_index } else { index };

            WORKERS[index].name = name.to_string();
            WORKERS[index].datagram = Some(datagram);

            match WORKERS[index].sender.send(payload) {
                Ok(_) => {
                }
                Err(err) => {
                    logging.d(format!("Failed send task({index}->{}), {:?}", name, err.to_string()));
                }
            }
        }
    }

    pub fn run(stream: &mut File, logging: &mut Logging, events: mpsc::Receiver<(usize, Event)>) {
        for worker_state in events {
            let index = worker_state.0;
            let event = worker_state.1;
            let name = unsafe { &WORKERS[index].name };

            match event {
                Event::IDLE => {
                    unsafe {
                        WORKERS[index].state = event;
                        WORKERS[index].name = String::new();
                        WORKERS[index].datagram = None;
                    }
                }
                Event::MESSAGE(flag, resp) => {
                    let worker = unsafe { &mut WORKERS[index] };
                    if let Some(datagram) = &mut worker.datagram {
                        let payload = datagram.payload.pack(&[flag], &resp);
                        let pkt = datagram.resp_pack(&payload);
                        logging.i(format!("<<--- Respond: len({})\n{:?}", pkt.len(), pkt));

                        // let new_dg = Datagram::new(&pkt);
                        // logging.i(new_dg.payload.info());

                        match stream.write_all(&pkt) {
                            Ok(()) => {
                                // datagram.update_seq(pkt.len() as u32);
                            }
                            Err(err) => {
                                logging.i(format!("<<--- Respond: Write error: {:?}", err));
                            }
                        }
                    }
                }
                Event::LOG(log) => {
                    logging.i(format!("{index}=>{}: {log}", name));
                }
                _ => {
                    unsafe {
                        WORKERS[index].state = event;
                    }
                }
            }
        }
    }

    // Stop all workers
    pub fn stop() {
        unsafe {
            loop {
                match WORKERS.pop() {
                    Some(worker) => {
                        drop(worker);
                    }
                    None => { break; }
                }
            }
            WORKERS.clear();
        }
    }
}

static mut WORKERS: Vec<Worker> = Vec::new();

type Message = Vec<u8>;
type Reporter = Arc<mpsc::Sender<(usize, Event)>>;
type Sender = mpsc::Sender<Payload>;
type Receiver = mpsc::Receiver<Payload>;

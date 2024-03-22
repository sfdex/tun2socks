use std::fs::File;
use std::io::Write;
use std::sync::{Arc, mpsc};

use crate::logging::Logging;
use crate::protocol::internet::Datagram;
use crate::thread_pool::event::Event;
use crate::thread_pool::worker::Worker;

mod worker;
pub mod event;
pub mod handler;

pub struct ThreadPool {}

impl ThreadPool {
    pub fn new(size: usize, reporter: Reporter) -> Self {
        for i in 0..size {
            let (tx, rx) = mpsc::channel();
            let reporter = Arc::clone(&reporter);
            unsafe {
                WORKERS.push(Worker::new(i, reporter, tx, rx));
            }
        }

        Self {
            // writer,
            // state_receiver,
        }
    }

    pub fn execute(datagram: Datagram, logging: &mut Logging) {
        let name = datagram.name();
        let d0 = Arc::new(datagram);
        let d1 = Arc::clone(&d0);
        unsafe {
            let mut index = 0;
            for (i, worker) in WORKERS.iter().enumerate() {
                if worker.name == name {
                    match worker.sender.send(d0) {
                        Ok(_) => {
                            logging.d(format!("Success send task({i}->{})", worker.name));
                        }
                        Err(err) => {
                            logging.d(format!("Failed send task({i}->{}), {:?}", worker.name, err.to_string()));
                        }
                    }
                    WORKERS[i].name = name;
                    WORKERS[i].datagram = Some(d1);
                    return;
                }

                if worker.name == "" {
                    index = i;
                }
            }

            WORKERS[index].sender.send(d0).unwrap();
            WORKERS[index].name = name;
            WORKERS[index].datagram = Some(d1);
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

                        let new_dg = Datagram::new(&pkt);
                        logging.i(new_dg.payload.info());

                        match stream.write_all(&pkt) {
                            Ok(()) => {
                                // datagram.update_seq(pkt.len() as u32);
                                logging.i("<<--- Respond: Write success\n".to_string());
                            }
                            Err(err) => {
                                logging.i(format!("<<--- Respond: Write error: {:?}", err));
                            }
                        }
                    }
                }
                Event::LOG(log) => {
                    println!("{index} LOG: {:?}", log);
                    logging.i(format!("{index}=>{}: {log}", name));
                }
                _ => {
                    unsafe {
                        WORKERS[index].state = event;
                    }
                }
            }
        }

        logging.i("<<--- tun2socks ended --->>".to_string());
    }

    // Stop all workers
    pub fn stop() {
        unsafe {
            for i in 0..10usize {
                WORKERS[i].stop();
            }
            WORKERS.clear();
        }
    }
}

static mut WORKERS: Vec<Worker> = Vec::new();

type Message = Vec<u8>;
type Reporter = Arc<mpsc::Sender<(usize, Event)>>;
type Sender = mpsc::Sender<Arc<Datagram>>;
type Receiver = mpsc::Receiver<Arc<Datagram>>;

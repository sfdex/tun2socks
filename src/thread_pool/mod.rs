use std::fs::File;
use std::io::Write;
use std::sync::{Arc, mpsc, Mutex};

use crate::protocol::internet::Datagram;
use crate::thread_pool::state::State;
use crate::thread_pool::worker::Worker;

mod state;
mod worker;
mod handler;

pub struct ThreadPool {
    // writer: File,
    // state_receiver: mpsc::Receiver<(usize, State)>,
}

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

    pub fn execute(datagram: Datagram) {
        let name = datagram.name();
        println!("new datagram: {}", name);
        let d0 = Arc::new(datagram);
        let d1 = Arc::clone(&d0);
        unsafe {
            let mut index = 0;
            for (i, worker) in WORKERS.iter().enumerate() {
                if worker.name == name {
                    println!("worker.name = {}, name = {}, =={}", worker.name, name, worker.name == name);
                    println!("Find it worker({i}), id: {}\n", &worker.name);

                    worker.sender.send(d0).unwrap();
                    WORKERS[i].name = name;
                    WORKERS[i].datagram = Some(d1);
                    return;
                }

                if worker.name == "" {
                    index = i;
                }
            }

            println!("worker({index}), name = {}\n", name);
            WORKERS[index].sender.send(d0).unwrap();
            WORKERS[index].name = name;
            WORKERS[index].datagram = Some(d1);
        }
    }

    pub fn run(stream: &mut File, events: mpsc::Receiver<(usize, State)>) {
        for worker_state in events {
            println!("Pool receive: index = {:?}", worker_state.0);

            let index = worker_state.0;
            let state = worker_state.1;

            match &state {
                State::IDLE => {
                    println!("{index} IDLE");
                    unsafe {
                        println!("Setting {index}");
                        WORKERS[index].state = state;
                        WORKERS[index].name = String::new();
                        WORKERS[index].datagram = None;
                        println!("Setting 2");
                    }
                }
                State::MESSAGE(flag, resp) => {
                    let mut worker = unsafe { &mut WORKERS[index] };
                    if let Some(datagram) = &mut worker.datagram {
                        let payload = datagram.payload.pack(&[*flag], &resp);
                        let pkt = datagram.resp_pack(&payload);
                        println!("run receive datagram: {:?}\n{:?}\n", pkt.len(), pkt);
                        match stream.write_all(&pkt) {
                            Ok(()) => {
                                // datagram.update_seq(pkt.len() as u32);
                                print!("Write success\n");
                            }
                            Err(err) => {
                                println!("Write error: {:?}", err);
                            }
                        }
                    }
                    return;
                }
                _ => {
                    unsafe {
                        WORKERS[index].state = state;
                    }
                }
            }
        }

        println!("run end");
    }
}

static mut WORKERS: Vec<Worker> = Vec::new();

type Message = Vec<u8>;
type Reporter = Arc<Mutex<mpsc::Sender<(usize, State)>>>;
type Sender = mpsc::Sender<Arc<Datagram>>;
type Receiver = mpsc::Receiver<Arc<Datagram>>;

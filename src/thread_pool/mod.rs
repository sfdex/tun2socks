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
        unsafe {
            let mut index = 0;
            for (i, worker) in WORKERS.iter().enumerate() {
                if worker.name == name {
                    println!("worker.name = {}, name = {}, =={}", worker.name, name, worker.name == name);
                    println!("Find it worker({i}), id: {}\n", &worker.name);
                    worker.sender.send(datagram).unwrap();
                    WORKERS[i].name = name;
                    return;
                }

                if worker.name == "" {
                    index = i;
                }
            }

            println!("worker({index}), name = {}\n", name);
            WORKERS[index].sender.send(datagram).unwrap();
            WORKERS[index].name = name;
        }
    }

    pub fn run(stream: &mut File, state_receiver: mpsc::Receiver<(usize, State)>) {
        for worker_state in state_receiver {
            println!("Pool receive: index = {:?}", worker_state.0);

            let index = worker_state.0;
            let state = worker_state.1;

            if let State::MESSAGE(resp) = &state {
                println!("run receive datagram: {:?}", resp.len());
                match stream.write_all(&resp) {
                    Ok(()) => {
                        print!("Write success\n");
                    }
                    Err(err) => {
                        println!("Write error: {:?}", err);
                    }
                }
            }

            if index < 10 {
                println!("Setting 0");
                unsafe {
                    println!("Setting {index}");
                    WORKERS[index].state = state;
                    println!("Setting 2");
                }
            }
        }

        println!("run end");
    }
}

static mut WORKERS: Vec<Worker> = Vec::new();

type Message = Vec<u8>;
type Reporter = Arc<Mutex<mpsc::Sender<(usize, State)>>>;
type Sender = mpsc::Sender<Datagram>;
type Receiver = mpsc::Receiver<Datagram>;

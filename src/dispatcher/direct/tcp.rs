use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::log;
use crate::protocol::internet::tcp::{ACK, PSH_ACK, RST, SYN_ACK};
use crate::thread_pool::event::Event::{IDLE, LOG, MESSAGE, TCP};
use crate::thread_pool::event::TcpState;
use crate::thread_pool::handler::Handler;

impl Handler {
    pub fn handle_tcp(&mut self) {
        let id = self.id;
        let payload = if let Some(pkt) = &self.payload {
            pkt
        } else {
            return;
        };

        self.report(log!("{}", payload.info()));

        let data = payload.payload();
        let dst_addr = payload.dst_addr();
        if let Some(stream) = &mut self.tcp {
            match stream.write_all(data) {
                Ok(_) => {
                    self.report(LOG("Send to remote success".into()));
                    self.report(MESSAGE(*ACK, vec![]));
                }
                Err(e) => {
                    self.report(log!("Send data error: bye bye, {:?}", e));
                    self.report(MESSAGE(*RST, vec![]));
                    self.report(IDLE);
                }
            }
            return;
        }

        // Connect
        self.report(log!("connect to {addr}", addr = dst_addr));
        let stream = match TcpStream::connect_timeout(&dst_addr, Duration::from_secs(5)) {
            Ok(stream) => {
                self.report(log!("success connect to server"));
                stream
            }
            Err(err) => {
                self.report(log!("failed connect: {e:#?}", e = err));
                self.report(MESSAGE(*RST, vec![]));
                self.report(IDLE);
                return;
            }
        };

        // Send S.
        self.report(MESSAGE(*SYN_ACK, vec![]));
        self.report(TCP(TcpState::SynAckWait));

        let reporter = Arc::clone(&self.reporter);

        let mut stream_cloned = stream.try_clone().unwrap();
        self.tcp = Some(stream);

        // Receive message
        let job = thread::spawn(move || {
            let mut buf = vec![0; 1500];
            log!("tcp loop start").report(id, &reporter);
            loop {
                match stream_cloned.read(&mut buf) {
                    Ok(n) => {
                        if n == 0 {
                            log!("reach end").report(id, &reporter);
                            MESSAGE(*RST, vec![]).report(id, &reporter);
                            break;
                        }
                        log!("<<---recv {n} bytes\n\t{:?}", &buf[..n]).report(id, &reporter);
                        MESSAGE(*PSH_ACK, buf[..n].to_vec()).report(id, &reporter);
                    }
                    Err(e) => {
                        log!("{:#?}", e).report(id, &reporter);
                        MESSAGE(*RST, vec![]).report(id, &reporter);
                        break;
                    }
                }
            }
            IDLE.report(id, &reporter);
        });

        self.job = Some(job);
    }
}
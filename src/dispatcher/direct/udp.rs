use std::net::UdpSocket;
use std::sync::Arc;
use std::thread;

use crate::log;
use crate::thread_pool::event::Event::{IDLE, MESSAGE};
use crate::thread_pool::handler::Handler;

impl Handler {
    pub fn handle_udp(&mut self) {
        let payload = if let Some(payload) = &self.payload {
            payload
        } else {
            return;
        };
        let data = payload.payload();

        if let Some(udp) = &self.udp {
            match udp.send(data) {
                Ok(n) => {
                    self.report(log!("sent {n} bytes"));
                }
                Err(err) => {
                    self.report(log!("sent error: {:#?}", err));
                    self.report(IDLE);
                }
            }

            return;
        }

        let udp = match UdpSocket::bind("0.0.0.0:0") {
            Ok(udp_socket) => {
                self.report(log!("bind success"));
                udp_socket
            }
            Err(err) => {
                self.report(log!("bind failed: {:#?}", err));
                return;
            }
        };

        let id = self.id;
        let dst_addr = payload.dst_addr();
        let reporter = Arc::clone(&self.reporter);

        match udp.connect(dst_addr) {
            Ok(_) => {
                self.report(log!("connect to server success"))
            }
            Err(err) => {
                self.report(log!("udp connect to server error: {:#?}", err));
                self.report(IDLE);
            }
        }

        match udp.send(data) {
            Ok(n) => {
                self.report(log!("sent {n} bytes"));
            }
            Err(err) => {
                self.report(log!("sent error: {:#?}", err));
                self.report(IDLE);
                return;
            }
        }

        let udp_cloned = udp.try_clone().unwrap();
        self.udp = Some(udp);

        // Receive message
        let mut buf = vec![0; 1500];
        let job = thread::spawn(move || {
            loop {
                log!("udp loop start").report(id, &reporter);

                match udp_cloned.recv(&mut buf) {
                    Ok(n) => {
                        // match udp.recv_from(&mut buf) {
                        //     Ok((n, addr)) => {
                        log!("udp recv {n} bytes, content: {}", String::from_utf8_lossy(&buf[..n])).report(id, &reporter);
                        MESSAGE(0, buf[..n].to_vec()).report(id, &reporter);
                    }
                    Err(err) => {
                        log!("udp recv error: {:#?}", err).report(id, &reporter);
                        IDLE.report(id, &reporter);
                        break;
                    }
                }
            }
            log!("udp recv end").report(id, &reporter);
        });

        self.job = Some(job);
    }
}
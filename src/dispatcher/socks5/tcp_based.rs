use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::protocol::internet::tcp::PSH_ACK;
use crate::protocol::socks5::*;
use crate::thread_pool::event::Event::MESSAGE;
use crate::thread_pool::Reporter;
use crate::util::bytes_to_u32;

pub struct Client {
    server_addr: SocketAddr,
    reporter: Reporter,
    dst_addr: Vec<u8>,
    dst_port: [u8; 2],
    methods: Vec<u8>,
    method: u8,
    stream: Option<TcpStream>,
    job: Option<JoinHandle<()>>,
}

impl Client {
    pub fn new(server_addr: SocketAddr, dst_addr: &[u8], dst_port: [u8; 2], reporter: Reporter) -> Self {
        Client {
            server_addr,
            reporter,
            dst_addr: dst_addr.to_vec(),
            dst_port,
            methods: vec![],
            method: 0,
            stream: None,
            job: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.negotiate()?;
        self.connect()?;
        self.recv()
    }

    fn negotiate(&mut self) -> Result<()> {
        let mut stream = match TcpStream::connect_timeout(&self.server_addr, Duration::from_secs(5)) {
            Ok(stream) => stream,
            Err(err) => {
                let err = format!("[NEGOTIATE] Failed to connect to socks5 server: {:?}", err);
                return Err(Error::new(ErrorKind::Other, err));
            }
        };

        let request = negotiation::Request::new(&self.methods);
        let bytes = request.as_bytes();

        match stream.write_all(&bytes) {
            Ok(_) => {}
            Err(err) => {
                let err = format!("[NEGOTIATE] Failed to write to socks5 server: {:?}", err);
                return Err(Error::new(ErrorKind::Other, err));
            }
        }

        let mut buf = [0; 1024];
        let reply = match stream.read(&mut buf) {
            Ok(n) => { negotiation::Reply::new(&buf[..n]) }
            Err(err) => {
                let err = format!("[NEGOTIATE] Failed to read from socks5 server: {:?}", err);
                return Err(Error::new(ErrorKind::Other, err));
            }
        };

        self.method = reply.method;
        self.stream = Some(stream);

        return Ok(());
    }

    fn connect(&mut self) -> Result<()> {
        let mut stream = if let Some(stream) = &self.stream {
            stream
        } else {
            return Err(Error::new(ErrorKind::Other, "[CONNECT] No stream"));
        };

        let request = request::TcpMessage::build_request(1, 1, (&self.dst_addr).to_vec(), self.dst_port);
        let bytes = request.as_bytes();

        match stream.write_all(&bytes) {
            Ok(_) => {}
            Err(err) => {
                let err = format!("[CONNECT] Failed to write to socks5 server: {:?}", err);
                return Err(Error::new(ErrorKind::Other, err));
            }
        }

        let mut buf = [0; 1024];
        let n = match stream.read(&mut buf) {
            Ok(n) => { n }
            Err(err) => {
                let err = format!("[CONNECT] Failed to read from socks5 server: {:?}", err);
                return Err(Error::new(ErrorKind::Other, err));
            }
        };

        let reply = request::TcpMessage::parse_reply(&buf[..n]);
        println!("Reply: {:?}", reply);
        if reply.opt == 0x00 {
            println!("[CONNECT] Success,  Server info: {:?}:{}", reply.addr, bytes_to_u32(&reply.port));
        } else {
            let err = "[CONNECT] Failed";
            return Err(Error::new(ErrorKind::Other, err));
        }

        Ok(())
    }

    pub fn send(&mut self, data: &[u8]) -> Result<()> {
        let mut stream = if let Some(stream) = &self.stream {
            stream
        } else {
            return Err(Error::new(ErrorKind::Other, "[Send] No stream"));
        };

        return match stream.write_all(data) {
            Ok(_) => { Ok(()) }
            Err(err) => {
                let err = format!("[Send] Failed to send to socks5 server: {:?}", err);
                Err(Error::new(ErrorKind::Other, err))
            }
        };
    }

    pub fn recv(&mut self) -> Result<()> {
        let stream = if let Some(stream) = &self.stream {
            stream
        } else {
            return Err(Error::new(ErrorKind::Other, "[Send] No stream"));
        };
        let mut stream_cloned = stream.try_clone().unwrap();
        let reporter = Arc::clone(&self.reporter);
        self.job = Some(thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                match stream_cloned.read(&mut buf) {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        MESSAGE(*PSH_ACK, buf[..n].to_vec()).report(0, &reporter);
                        println!("[COMM] Recv {} bytes", n);
                    }
                    Err(err) => {
                        println!("[COMM] Failed to recv from socks5 server: {:?}", err);
                        break;
                    }
                }
            }
        }));

        return Ok(());
    }

    pub fn stop(self) {
        if let Some(stream) = self.stream {
            stream.shutdown(Shutdown::Both).unwrap_or(());
            drop(stream);
        }
        if let Some(job) = self.job {
            drop(job);
        }
        drop(self.reporter);
    }
}
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use crate::logging::Logging;

pub fn test(logging: &mut Logging) {
    let tag = "SFDEX-TEST: ";

    logging.i(format!("{tag}"));

    let dst_addr = SocketAddr::from_str("1.2.3.4:5678").unwrap();
    let mut stream = match TcpStream::connect_timeout(&dst_addr, Duration::from_secs(5)) {
        Ok(stream) => {
            logging.i(format!("{tag}success connect to server"));
            stream
        }
        Err(err) => {
            logging.i(format!("{tag}failed connect: {e:#?}", e = err));
            return;
        }
    };

    let mut cloned_stream = stream.try_clone().unwrap();
    let mut logging2 = logging.clone();
    let job = thread::spawn(move || {
        let mut buf = vec![0; 1500];
        loop {
            match cloned_stream.read(&mut buf) {
                Ok(0) => {
                    logging2.i(format!("{tag}reach end"));
                    break;
                }
                Ok(n) => {
                    let bytes = &buf[..n];
                    logging2.i(format!("{tag}Recv: len({})\n{:?}", n, bytes));
                }
                Err(e) => {
                    logging2.i(format!("{tag}Read data error: bye bye, {:?}", e));
                    break;
                }
            }
        }
    });

    for i in 0..10 {
        thread::sleep(Duration::from_secs(1));
        let msg = format!("No.{}", i);
        match stream.write_all(msg.as_bytes()) {
            Ok(_) => {
                logging.i(format!("{tag}Send to remote success"));
            }
            Err(e) => {
                logging.i(format!("{tag}Send data error: bye bye, {:?}", e));
            }
        }
    }

    job.join().unwrap();
}
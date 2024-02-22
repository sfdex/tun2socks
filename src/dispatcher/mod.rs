use std::fs::File;
use std::net::IpAddr;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol};
use crate::protocol::internet::tcp::Tcp;

pub fn dispatch(data: Vec<u8>, stream: &mut File, logging: &mut Logging) {
    let datagram = Datagram::new(&data);
    let ip_header = &datagram.header;

    logging.i(format!("{:?}: {:?} => {:?}", &datagram.protocol(), IpAddr::from(ip_header.src_ip), IpAddr::from(ip_header.dst_ip)));

    match &datagram.protocol() {
        Protocol::TCP => {
            let tcp = Tcp::new(datagram.payload);
        }
        Protocol::UDP => {
            logging.e("Unsupported udp protocol".to_string())
        }
        Protocol::ICMP => {
            logging.e("Unsupported ICMP protocol".to_string())
        }
        Protocol::UNKNOWN => {
            logging.e("Unsupported unknown protocol".to_string())
        }
    }
}
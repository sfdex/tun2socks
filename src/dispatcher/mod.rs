use std::fs::File;
use std::io::Write;
use std::net::IpAddr;
use std::time::SystemTime;
use crate::dispatcher::direct::dial_tcp;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol, tcp};
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::tcp::ControlType::*;
use crate::util::{bytes_to_u32, bytes_to_u32_no_prefix};

pub mod direct;

pub fn dispatch(data: Vec<u8>, id: u32, stream: &mut File, logging: &mut Logging) {
    let datagram = Datagram::new(&data);
    let ip_header = &datagram.header;

    // Fragment
    let id = bytes_to_u32(&ip_header.identification);
    let mf = (ip_header.flags_fragment_offset[0] >> 5) & 1;
    let offset = bytes_to_u32_no_prefix(&ip_header.flags_fragment_offset, 3);

    logging.i(format!("{id}: {:?}: {:?} => {:?}, IHL({}), ID({id}), MF({mf}), OFFSET({offset})",
                      &datagram.protocol(),
                      IpAddr::from(ip_header.src_ip),
                      IpAddr::from(ip_header.dst_ip),
                      ip_header.version_ihl & 0x0F,
    ));

    logging.i(format!("payload length: {}", &datagram.payload.len()));

    match &datagram.protocol() {
        Protocol::TCP => {
            let tcp = Tcp::new(&datagram.payload, ip_header.src_ip, ip_header.dst_ip);
            logging.i(tcp.info());
            match tcp.control_type() {
                SYN => {
                    let ip_package = datagram.pack(&tcp.pack(id, 0b010010));
                    logging.i(format!("Respond: {:?}", &ip_package));

                    if let Err(err) = stream.write(&ip_package) {
                        logging.e(format!("Response to tun error: {}", err))
                    };
                }
                PUSH => {}
                _ => {}
            };
            // let addr = IpAddr::from(ip_header.dst_ip);
            // let port = bytes_to_u32(&tcp.header.dst_port) as u16;
            // let result = dial_tcp(addr, port, &[1u8]);
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
    };

    logging.d("TCP end\n\n".to_string());
}
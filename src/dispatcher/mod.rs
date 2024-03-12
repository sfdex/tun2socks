use std::fs::File;
use std::io::Write;
use std::net::IpAddr;
use std::time::SystemTime;
use crate::dispatcher::direct::dial_tcp;
use crate::logging;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol, PseudoHeader, tcp};
use crate::protocol::internet::icmp::{Echo, Icmp};
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::tcp::ControlType::*;
use crate::protocol::internet::udp::Udp;
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

    let mut pseudo_header = PseudoHeader {
        src_ip: ip_header.dst_ip,
        dst_ip: ip_header.src_ip,
        protocol: ip_header.protocol,
        length: [0, 0],
    };

    match &datagram.protocol() {
        Protocol::TCP => {
            let tcp = Tcp::new(&datagram.payload);
            logging.i(tcp.info());
            match tcp.control_type() {
                SYN => {
                    let tcp_syn_response = tcp.pack(id, 0b010010, vec![], &mut pseudo_header);
                    logging.i(format!("tcp_syn_response: {}", Tcp::new(&tcp_syn_response).info()));
                    let ip_packet = datagram.pack(&tcp_syn_response);
                    logging.i(format!("Respond to tcp({id}), len({}), packet: {:?}", &ip_packet.len(), &ip_packet));

                    match stream.write(&ip_packet) {
                        Ok(n) => {
                            logging.i(format!("Respond to tcp({id}), size({n})"));
                        }
                        Err(err) => {
                            logging.e(format!("Response to tcp({id}) error: {}", err))
                        }
                    }
                }
                PUSH => {}
                _ => {}
            };
            // let addr = IpAddr::from(ip_header.dst_ip);
            // let port = bytes_to_u32(&tcp.header.dst_port) as u16;
            // let result = dial_tcp(addr, port, &[1u8]);

            logging.d("TCP end\n\n".to_string());
        }
        Protocol::UDP => {
            let udp = Udp::new(&datagram.payload);
            logging.i(udp.info());

            let mut msg = Vec::new();
            msg.extend_from_slice("Hello ".as_bytes());
            msg.extend_from_slice(&udp.payload);

            let ip_packet = datagram.pack(&udp.pack(&mut pseudo_header, &msg));
            logging.i(format!("Respond to udp({id}), len({}), packet: {:?}", &ip_packet.len(), &ip_packet));

            match stream.write(&ip_packet) {
                Ok(n) => {
                    logging.i(format!("Respond to udp({id}), size({n})"));
                }
                Err(err) => {
                    logging.e(format!("Response to udp({id}) error: {:?}", err))
                }
            }

            logging.d("UDP end\n\n".to_string());
        }
        Protocol::ICMP => {
            let icmp: Icmp<Echo> = Icmp::new(&datagram.payload);
            let response = icmp.pack();
            let ip_packet = datagram.pack(&response);
            match stream.write(&ip_packet) {
                Ok(n) => {
                    logging.i(format!("Respond to icmp({id}), size({n})"));
                }
                Err(err) => {
                    logging.e(format!("Response to icmp({id}) error: {:?}", err))
                }
            }

            logging.d("ICMP end\n\n".to_string());
        }
        Protocol::UNKNOWN => {
            logging.e("Unsupported unknown protocol".to_string())
        }
    };
}
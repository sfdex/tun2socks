use std::fs::File;
use std::io::Write;
use std::net::IpAddr;
use crate::dispatcher::simulator::Simulator;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol, PseudoHeader};
use crate::protocol::internet::icmp::{Echo, Icmp};
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::udp::Udp;
use crate::util::{bytes_to_u32, bytes_to_u32_no_prefix};

pub mod direct;
pub mod simulator;

pub fn dispatch(data: Vec<u8>, stream: &mut File, logging: &mut Logging) {
    let datagram = Datagram::new(&data);
    let ip_header = &datagram.header;

    // Fragment
    let id = bytes_to_u32(&ip_header.identification);
    let mf = (ip_header.flags_fragment_offset[0] >> 5) & 1;
    let offset = bytes_to_u32_no_prefix(&ip_header.flags_fragment_offset, 3);

    logging.i(format!("--->> {id}: {:?}: {:?} => {:?}, IHL({}), ID({id}), MF({mf}), OFFSET({offset})",
                      &datagram.protocol(),
                      IpAddr::from(ip_header.src_ip),
                      IpAddr::from(ip_header.dst_ip),
                      ip_header.version_ihl & 0x0F,
    ));
    
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
            let response = Simulator::handle_tcp(&tcp, id, &mut pseudo_header, logging);
            if response.len() == 0 { 
                return;
            }

            for tcp_packet in response {
                logging.i(format!("<<--- Respond {}", Tcp::new(&tcp_packet).info()));
                let ip_packet = datagram.pack(&tcp_packet);
                logging.i(format!("<<--- Send: tcp({id}), len({}), packet: {:?}\n", &ip_packet.len(), &ip_packet));

                match stream.write(&ip_packet) {
                    Ok(n) => {
                        if n != ip_packet.len() {
                            logging.i(format!("<<--- Send: tcp({id}) error, write size({n}\n, len({}))", ip_packet.len()));
                        }
                    }
                    Err(err) => {
                        logging.e(format!("<<--- Send: tcp({id}) error: {}\n", err))
                    }
                }
            }
        }
        Protocol::UDP => {
            let udp = Udp::new(&datagram.payload);
            logging.i(udp.info());

            let mut msg = Vec::new();
            msg.extend_from_slice("Hello ".as_bytes());
            msg.extend_from_slice(&udp.payload);

            let ip_packet = datagram.pack(&udp.pack(&mut pseudo_header, &msg));
            logging.i(format!("<<--- Send: udp({id}), len({}), packet: {:?}\n", &ip_packet.len(), &ip_packet));

            match stream.write(&ip_packet) {
                Ok(n) => {
                    if n != ip_packet.len() {
                        logging.i(format!("<<--- Send: udp({id}) error, write size({n}, len({}))\n", ip_packet.len()));
                    }
                }
                Err(err) => {
                    logging.e(format!("<<--- Send: udp({id}) error: {:?}\n", err))
                }
            }
        }
        Protocol::ICMP => {
            let icmp: Icmp<Echo> = Icmp::new(&datagram.payload);
            let response = icmp.pack();
            let ip_packet = datagram.pack(&response);
            match stream.write(&ip_packet) {
                Ok(n) => {
                    if n != ip_packet.len() {
                        logging.i(format!("<<--- Send: icmp({id}) error, write size({n}, len({}))\n", ip_packet.len()));
                    }
                }
                Err(err) => {
                    logging.e(format!("<<--- Send: icmp({id}) error: {:?}\n", err))
                }
            }
        }
        Protocol::UNKNOWN => {
            logging.e("Unsupported unknown protocol\n".to_string())
        }
    };
}
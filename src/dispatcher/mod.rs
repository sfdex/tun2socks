use std::fs::File;
use std::io::Write;
use std::net::IpAddr;
use crate::dispatcher::simulator::Simulator;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol, Packet, PseudoHeader};
use crate::protocol::internet::icmp::Icmp;
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
    
    let pseudo_header = PseudoHeader {
        src_ip: ip_header.dst_ip,
        dst_ip: ip_header.src_ip,
        protocol: ip_header.protocol,
        length: [0, 0],
    };

    let protocol = &datagram.protocol();

    let packet = build_packet(protocol, &datagram.payload, pseudo_header);
    logging.i(packet.info());
    
    if let Protocol::UNKNOWN = protocol {
        logging.e("Unsupported unknown protocol\n".to_string());
        return;
    }

    let response = Simulator::handle(protocol, &packet);

    if response.len() == 0 {
        return;
    }

    for msg in response {
        logging.i(format!("<<--- Respond {}", build_packet(protocol, &msg, pseudo_header).info()));
        let ip_packet = datagram.pack(&msg);
        logging.i(format!("<<--- Send: {:?}({id}), len({}), packet: {:?}\n", protocol, &ip_packet.len(), &ip_packet));

        match stream.write(&ip_packet) {
            Ok(n) => {
                if n != ip_packet.len() {
                    logging.i(format!("<<--- Send: {:?}({id}) error, write size({n}\n, len({}))", protocol, ip_packet.len()));
                }
            }
            Err(err) => {
                logging.e(format!("<<--- Send: {:?}({id}) error: {}\n", protocol, err))
            }
        }
    }
}

fn build_packet(protocol: &Protocol, data: &[u8], pseudo_header: PseudoHeader) -> Box<dyn Packet> {
    return match protocol {
        Protocol::TCP => {
            Box::new(Tcp::new(data, pseudo_header))
        }
        Protocol::UDP => {
            Box::new(Udp::new(data, pseudo_header))
        }
        Protocol::ICMP => {
            Box::new(Icmp::new(data))
        }
        // Protocol::UNKNOWN => {}
        _=> {
            Box::new(Udp::new(data, pseudo_header))
        }
    }
}

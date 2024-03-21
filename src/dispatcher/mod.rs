use std::fs::File;
use std::io::Write;
use std::net::IpAddr;
use std::thread;
use crate::dispatcher::simulator::Simulator;
use crate::logging::Logging;
use crate::protocol::internet::{Datagram, Protocol, Packet, PseudoHeader};
use crate::protocol::internet::icmp::Icmp;
use crate::protocol::internet::tcp::Tcp;
use crate::protocol::internet::udp::Udp;
use crate::util::{bytes_to_u32, bytes_to_u32_no_prefix};

pub mod simulator;
pub mod direct;

pub fn handle_datagram(datagram: &[u8], stream: &mut File, logging: &mut Logging) {
    logging.i(format!("--->> Recv: len({}), {:?}", (&datagram).len(), &datagram));

    // msg[0] & 4 == 4 #ipv4
    // msg[0] & 6 == 6 #ipv6
    let version = (datagram[0] >> 4) & 0b1111;
    match version {
        4 => {
            // stream.read(&mut buf2) // Read remaining bytes error: OS(11), Operation would block.
            let data = datagram.to_vec();
            let mut copy_stream = stream.try_clone().unwrap();
            let mut copy_logging = logging.clone();

            let s = thread::spawn(move || {
                dispatch(data, &mut copy_stream, &mut copy_logging);
            });

            s.join().expect("TODO: panic message");
        }
        6 => {
            logging.w("Unsupported version ipv6".to_string());
        }
        _ => {
            logging.e(format!("Error version {}", version));
        }
    }
}

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

    // let packet = build_packet(protocol, &datagram.payload, pseudo_header);
    let packet = build_packet(protocol, &[], pseudo_header);
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
        let ip_packet = datagram.resp_pack(&msg);
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

fn build_packet(protocol: &Protocol, data: &[u8], pseudo_header: PseudoHeader) -> Box<dyn Packet + Send + Sync> {
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
        _ => {
            Box::new(Udp::new(data, pseudo_header))
        }
    };
}

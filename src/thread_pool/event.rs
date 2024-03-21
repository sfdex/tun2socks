use crate::thread_pool::{Message, Reporter};

#[derive(Debug)]
pub enum Event {
    MESSAGE(u8, Message),
    TCP(TcpState),
    UDP(UdpState),
    ICMP(IcmpState),
    LOG(String),
    IDLE,
}

#[derive(Debug)]
pub enum TcpState {
    SynAckWait,
    Communication,
    FinWait,
    RstWait,
    Destroy,
}

#[derive(Debug)]
pub enum UdpState {
    Communication,
    Destroy,
}

#[derive(Debug)]
pub enum IcmpState {
    Communication,
    Destroy,
}

impl Event {
    pub fn report(self, id: usize, reporter: &Reporter) {
        println!("## reporting event: {:?}", self);
        match reporter.send((id, self)) {
            Ok(_) => {}
            Err(err) => {
                println!("!# reporting event failed: {}", err.to_string())
            }
        }
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        crate::thread_pool::event::Event::LOG(res)
    }}
}
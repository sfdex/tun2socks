use crate::thread_pool::{Message, Reporter};

pub enum Event {
    MESSAGE(u8, Message),
    TCP(String, TcpState),
    UDP(String, UdpState),
    ICMP(String, IcmpState),
    LOG(String),
    IDLE,
}

pub enum TcpState {
    SynAckWait,
    Communication,
    FinWait,
    RstWait,
    Destroy,
}

pub enum UdpState {
    Communication,
    Destroy,
}

pub enum IcmpState {
    Communication,
    Destroy,
}

impl Event {
    pub fn report(self, id: usize, reporter: &Reporter) {
        reporter.lock().unwrap().send((id, self)).unwrap();
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let res = std::fmt::format(format_args!($($arg)*));
        Event::LOG(res)
    }}
}
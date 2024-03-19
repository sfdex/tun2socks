use crate::thread_pool::Message;

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
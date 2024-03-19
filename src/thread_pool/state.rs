use crate::thread_pool::Message;

pub struct StateMessage(pub usize, pub State);

pub enum State {
    MESSAGE(u8, Message),
    TCP(String, TcpState),
    UDP(String, UdpState),
    ICMP(String, IcmpState),
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
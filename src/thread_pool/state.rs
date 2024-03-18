use crate::thread_pool::Message;

pub struct StateMessage(pub usize, pub State);

pub enum State {
    MESSAGE(Message),
    TCP { name: String, state: TcpState },
    UDP { name: String, state: UdpState },
    ICMP { name: String, state: IcmpState },
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
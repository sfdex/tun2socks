/*
The SOCKS request/reply is formed as follows:

        +----+-----+-------+------+----------+----------+
        |VER | OPT |  RSV  | ATYP |   ADDR   |   PORT   |
        +----+-----+-------+------+----------+----------+
        | 1  |  1  | X'00' |  1   | Variable |    2     |
        +----+-----+-------+------+----------+----------+

     Where:
     # Request message:

          o  VER        protocol version: X'05'
          o  OPT(CMD)   Command field:
             o  CONNECT X'01'
             o  BIND X'02'
             o  UDP ASSOCIATE X'03'
          o  RSV    RESERVED
          o  ATYP   address type of following address
             o  IP V4 address: X'01'
             o  DOMAINNAME: X'03'
             o  IP V6 address: X'04'
          o  ADDR   desired destination address
          o  PORT   desired destination port in network octet order


     # Reply message:
     
          o  VER        protocol version: X'05'
          o  OPT(REP)   Reply field:
             o  X'00' succeeded
             o  X'01' general SOCKS server failure
             o  X'02' connection not allowed by ruleset
             o  X'03' Network unreachable
             o  X'04' Host unreachable
             o  X'05' Connection refused
             o  X'06' TTL expired
             o  X'07' Command not supported
             o  X'08' Address type not supported
             o  X'09' to X'FF' unassigned
          o  RSV    RESERVED
          o  ATYP   address type of following address
             o  IP V4 address: X'01'
             o  DOMAINNAME: X'03'
             o  IP V6 address: X'04'
          o  ADDR   server bound address
          o  PORT   server bound port in network octet order
 */
#[derive(Debug)]
pub struct TcpMessage {
    pub ver: u8,
    pub opt: u8,
    rsv: u8,
    atyp: u8,
    pub addr: Vec<u8>,
    pub port: [u8; 2],
}

impl TcpMessage {
    pub fn build_request(opt: u8, atyp: u8, addr: Vec<u8>, port: [u8; 2]) -> Self {
        TcpMessage {
            ver: 5,
            opt,
            rsv: 0,
            atyp,
            addr,
            port,
        }
    }

    pub fn parse_reply(data: &[u8]) -> Self {
        let len = data.len();
        let ver = data[0];
        let opt = data[1];
        let rsv = data[2];
        let atyp = data[3];
        let addr = data[4..len - 2].to_vec();
        let port = [data[len - 2], data[len - 1]];
        TcpMessage {
            ver,
            opt,
            rsv,
            atyp,
            addr,
            port,
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.ver);
        bytes.push(self.opt);
        bytes.push(self.rsv);
        bytes.push(self.atyp);
        bytes.extend(&self.addr);
        bytes.extend(&self.port);
        bytes
    }
}

/*
Each UDP datagram carries a UDP request header with it:

      +----+------+------+----------+----------+----------+
      |RSV | FRAG | ATYP |   ADDR   |   PORT   |   DATA   |
      +----+------+------+----------+----------+----------+
      | 2  |  1   |  1   | Variable |    2     | Variable |
      +----+------+------+----------+----------+----------+

The fields in the UDP request header are:
          o  RSV  Reserved X'0000'
          o  FRAG    Current fragment number
          o  ATYP    address type of following addresses:
             o  IP V4 address: X'01'
             o  DOMAINNAME: X'03'
             o  IP V6 address: X'04'
          o  ADDR    desired destination address
          o  PORT    desired destination port
          o  DATA    user data

 */
pub struct UdpMessage {
    rsv: [u8; 2],
    frag: u8,
    atyp: u8,
    addr: Vec<u8>,
    port: [u8; 2],
    data: Vec<u8>,
}

impl UdpMessage {
    pub fn new(atyp: u8, addr: &[u8], port: [u8;2], data: &[u8]) -> Self {
        Self {
            rsv: [0, 0],
            frag: 0,
            atyp,
            addr: addr.to_vec(),
            port,
            data: data.to_vec(),
        }
    }
}
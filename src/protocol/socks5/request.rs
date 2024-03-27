/**
# Request message:

o  VER    protocol version: X'05'
o  OPT
   o  CONNECT X'01'
   o  BIND X'02'
   o  UDP ASSOCIATE X'03'
o  RSV    RESERVED
o  ATYP   address type of following address
   o  IP V4 address: X'01'
   o  DOMAINNAME: X'03'
   o  IP V6 address: X'04'
o  ADDR       desired destination address
o  PORT       desired destination port in network octet
   order

# Reply message:
o  VER    protocol version: X'05'
o  OPT    Reply field:
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
o  ADDR       server bound address
o  PORT       server bound port in network octet order
 */
pub struct Message {
    pub ver: u8,
    pub opt: u8,
    rsv: u8,
    atyp: u8,
    addr: Vec<u8>,
    port: [u8; 2],
}

impl Message {
    pub fn build_request(opt: u8, atyp: u8, addr: Vec<u8>, port: [u8; 2]) -> Self {
        Message {
            ver: 5,
            opt,
            rsv: 0,
            atyp,
            addr,
            port,
        }
    }

    fn parse_reply(data: &[u8]) -> Self {
        let len = data.len();
        let ver = data[0];
        let opt = data[1];
        let rsv = data[2];
        let atyp = data[3];
        let addr = data[4..len - 2].to_vec();
        let port = [data[len - 2], data[len - 1]];
        Message {
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
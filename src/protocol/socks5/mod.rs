struct Connect {
    ver: u8,
    nmethods: u8,
    methods: Vec<u8>,
}

/**
The values currently defined for METHOD are:
    o  X'00' NO AUTHENTICATION REQUIRED
    o  X'01' GSSAPI
    o  X'02' USERNAME/PASSWORD
    o  X'03' to X'7F' IANA ASSIGNED
    o  X'80' to X'FE' RESERVED FOR PRIVATE METHODS
    o  X'FF' NO ACCEPTABLE METHODS
 */

struct ConnectResponse {
    ver: u8,
    method: u8,
}


/**
o  VER    protocol version: X'05'
o  CMD
   o  CONNECT X'01'
   o  BIND X'02'
   o  UDP ASSOCIATE X'03'
o  RSV    RESERVED
o  ATYP   address type of following address
   o  IP V4 address: X'01'
   o  DOMAINNAME: X'03'
   o  IP V6 address: X'04'
o  DST.ADDR       desired destination address
o  DST.PORT desired destination port in network octet
   order
*/
struct Request {
    ver: u8,
    cmd: u8,
    rsv: u8,
    atyp: u8,
    dst_addr: Vec<u8>,
    dst_port: u16,
}

/**
          o  VER    protocol version: X'05'
          o  REP    Reply field:
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
          o  BND.ADDR       server bound address
          o  BND.PORT       server bound port in network octet order
*/
struct Response{
    ver:u8,
    rep:u8,
    rsv:u8,
    atyp:u8,
    bnd_addr:Vec<u8>,
    bnd_port:u16,
}

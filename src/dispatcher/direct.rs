use std::net::{IpAddr, SocketAddr, TcpStream, UdpSocket};
use std::io::{Error, ErrorKind, Read, Result, Write};
// use log::log;

pub fn dial_tcp(addr: IpAddr, port: u16, data: &[u8]) -> Result<Vec<u8>> {
    let mut stream = TcpStream::connect(SocketAddr::new(addr, port))?;
    stream.write_all(data)?;
    let mut buf = vec![0; 1024];
    let n = stream.read(&mut buf)?;
    if n > 0 {
        return Ok(buf[..n].to_vec());
    };
    return Err(Error::new(ErrorKind::UnexpectedEof, "Tcp remote response"));
}

pub fn dial_udp(addr: IpAddr, port: u16, data: &[u8]) -> Result<Vec<u8>> {
    let udp_socket = UdpSocket::bind("127.0.0.1:889")?;
    udp_socket.connect(SocketAddr::new(addr, port))?;
    let sent_bytes = udp_socket.send(data)?;
    let mut buf = vec![0; 1024];
    let recv_bytes = udp_socket.recv(&mut buf)?;
    if recv_bytes > 0 {
        return Ok(buf[..recv_bytes].to_vec());
    };
    return Err(Error::new(ErrorKind::UnexpectedEof, "Udp remote response"));
}
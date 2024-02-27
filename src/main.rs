// use tun2socks::dns::dns_resolve;

use std::fs::File;
use tun2socks_rust::logging::Logging;
use tun2socks_rust::protocol::internet::Datagram;
use tun2socks_rust::tun;

fn test_datagram_tcp() {
    let datagram = [69, 0, 0, 60, 74, 107, 64, 0, 64, 6, 34, 152, 10, 0, 2, 16, 192, 168, 1, 1, 133, 156, 3, 85, 238, 109, 198, 113, 0, 0, 0, 0, 160, 2, 255, 255, 42, 67, 0, 0, 2, 4, 4, 196, 4, 2, 8, 10, 198, 91, 76, 200, 0, 0, 0, 0, 1, 3, 3, 6];
    let mut stream = File::create("build/stream.txt").unwrap();
    let mut logging = Logging::new("build/logging.txt");
    tun::handle_datagram(&datagram, 1, &mut stream, &mut logging);
    // assert_eq!(2 + 2, 4);
}


fn main() {
    // dns_resolve();
    test_datagram_tcp()
}

use tun2socks::dns::dns_resolve;
use tun2socks::tun2socks;

fn main() {
    println!("Hello, world!");
    tun2socks();
    dns_resolve();
}

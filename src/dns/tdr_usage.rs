/*use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::Resolver;

pub fn dns_resolve() {
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    // let domain_name = "lab.surfacephone.top";
    let domain_name = "qq.com";
    match resolver.lookup_ip(domain_name) {
        Ok(response) => {
            for ip in response.iter() {
                println!("Found address: {}", ip);
            }
        }
        Err(err) => eprintln!("Cannot resolve domain: {}", err)
    }
}*/
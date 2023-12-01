use std::net::UdpSocket;

pub fn dns_resolve() {
    // 1. Build DNS query message
    let query = build_dns_query_message("www.google.com");

    // 2. Send UDP request
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to address");
    socket
        .send_to(&query, "8.8.8.8:53")
        .expect("Couldn't send data");

    // 3. Receive UDP response
    let mut buf = [0u8; 512];
    let (number_of_bytes, src_addr) = socket.recv_from(&mut buf).expect("Didn't receive data");
    println!("number_of_bytes: {}", number_of_bytes);
    println!("src_addr: {}", src_addr);

    let received_data = &buf[..number_of_bytes];

    // 4. Parse response
    let response = parse_dns_response(received_data);
}

fn build_dns_query_message(domain_name: &str) -> Vec<u8> {
    let mut message: Vec<u8> = Vec::new();
    message.extend_from_slice(&[0, 3]); // ID
    message.extend_from_slice(&[1, 0]); // FLAGS
    message.extend_from_slice(&[0, 1]); // NQ
    message.extend_from_slice(&[0, 0]); // NANRR
    message.extend_from_slice(&[0, 0]); // NAURR
    message.extend_from_slice(&[0, 0]); // NADRR

    // Domain name
    for (usize, str) in domain_name.split(".").into_iter().enumerate() {
        message.push(str.len() as u8);
        message.extend_from_slice(str.as_bytes());
        println!("len: {}", str.len());
        println!("str: {}", str);
    }
    message.push(0); // End of this domain name

    message.extend_from_slice(&[0, 1]); // Query is Type A query (Host Address)
    message.extend_from_slice(&[0, 1]); // Query is class IN (Internet Address)

    message
}

fn parse_dns_response(response: &[u8]) -> [u8; 4] {
    println!("response: {:?}", response);
    return [0, 0, 0, 0];
}

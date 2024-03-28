/**
If the connection request succeeds, the client enters a negotiation for the authentication
method to be used, authenticates with the chosen method, then sends a relay request.

The client connects to the server, and sends a version identifier/method selection message
 */
pub struct Request {
    ver: u8,
    nmethods: u8,
    methods: Vec<u8>,
}

impl Request {
    pub fn new(methods: &[u8]) -> Self {
        Request {
            ver: 5,
            nmethods: methods.len() as u8,
            methods: methods.to_vec(),
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.ver);
        bytes.push(self.nmethods);
        bytes.extend(&self.methods);
        bytes
    }
}

/**
The SOCKS server evaluates the request, and either establishes the appropriate connection or denies it.

The server selects from one of the methods given in METHODS, and sends a METHOD selection message

The values currently defined for METHOD are:
    o  X'00' NO AUTHENTICATION REQUIRED
    o  X'01' GSSAPI
    o  X'02' USERNAME/PASSWORD
    o  X'03' to X'7F' IANA ASSIGNED
    o  X'80' to X'FE' RESERVED FOR PRIVATE METHODS
    o  X'FF' NO ACCEPTABLE METHODS
 */
pub struct Reply {
    pub ver: u8,
    pub method: u8,
}

impl Reply {
    pub fn new(data: &[u8]) -> Self {
        Reply {
            ver: data[0],
            method: data[1],
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        vec![self.ver, self.method]
    }
}
pub fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut result = 0u32;
    let mut mv = bytes.len() - 1;
    for byte in bytes {
        result = result + (*byte << mv * 8) as u32;
        mv = mv - 1;
    };
    result
}

pub fn bytes_to_u32_no_prefix(bytes: &[u8], n_prefix: u32) -> u32 {
    let mut result = 0u32;
    let mut mv = bytes.len() - 1;
    for byte in bytes {
        if mv == bytes.len() - 1 {
            let highest = byte & (2u8.pow(8 - n_prefix) - 1);
            result = (highest as u32) << mv * 8;
        } else {
            result = result + ((*byte as u32) << mv * 8);
        }
        mv = mv - 1;
    };
    result
}

pub fn u16_to_bytes(number: u16) -> [u8; 2] {
    let highest = (number / 256) as u8;
    let lowest = (number % 256) as u8;
    [highest, lowest]
}

pub fn u32_to_bytes(number: u32) -> [u8; 4] {
    // let first = (number / (1u32 << 24)) as u8;
    // let second = (number / (1u32 << 16) % 256) as u8;
    // let third = (number / (1u32 << 8) % 256) as u8;
    // let fourth = (number % (1u32 << 8)) as u8;

    // another implementation
    let first = ((number >> 24) & 0xFF) as u8;
    let second = ((number >> 16) & 0xFF) as u8;
    let third = ((number >> 8) & 0xFF) as u8;
    let fourth = (number & 0xFF) as u8;

    [first, second, third, fourth]
}
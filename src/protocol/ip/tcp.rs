struct TcpHeader{
    src_port:u16,
    dst_port:u16,
    seq_no:u32,
    ack_no:u32,
    data_offset:u8,
    flags:u8,
    window_size:u16,
    check_sum:u16,
    urgent_pointer:u16,
    options:Vec<u8>,
}
use std::net::Ipv4Addr;

pub trait Unpack: Sized {
    fn unpack(bytes: &mut Vec<u8>) -> Self;
}

impl Unpack for u8 {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        bytes.drain(..1).next().unwrap()
    }
}

impl Unpack for u32 {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let mut buf: [u8; 4] = [0; 4];
        let drain: Vec<u8> = bytes.drain(..4).collect();
        buf.copy_from_slice(&drain);
        return u32::from_le_bytes(buf);
    }
}

impl Unpack for bool {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let drain: Vec<u8> = bytes.drain(..1).collect();
        return drain[0] == 1;
    }
}

impl Unpack for Ipv4Addr {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        Ipv4Addr::from(<u32>::unpack(bytes))
    }
}

impl Unpack for String {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let length = <u32>::unpack(bytes) as usize;
        let mut buffer: Vec<u8> = vec![0; length];
        let drain: Vec<u8> = bytes.drain(..length).collect();
        buffer.copy_from_slice(&drain);
        return String::from_utf8_lossy(&mut buffer).to_string();
    }
}
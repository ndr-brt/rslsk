use flate2::{Decompress, FlushDecompress};

use crate::message::unpack::Unpack;

pub struct PeerInit {
    pub username: String,
    pub connection_type: String, // TODO: this could be an enum
    pub token: u32
}

impl Unpack for PeerInit {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        PeerInit {
            username: <String>::unpack(bytes),
            connection_type: <String>::unpack(bytes),
            token: <u32>::unpack(bytes)
        }
    }
}

pub struct FileSearchResponse {
    pub username: String,
    pub token: u32,
    pub count: u32
}

impl Unpack for FileSearchResponse {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let mut uncompressed = Vec::with_capacity(bytes.len() * 4096);
        Decompress::new(true).decompress_vec(bytes.as_slice(), &mut uncompressed, FlushDecompress::Sync).unwrap();

        let username = <String>::unpack(&mut uncompressed);
        let token = <u32>::unpack(&mut uncompressed);
        let count = <u32>::unpack(&mut uncompressed);

        FileSearchResponse { username, token, count }
    }
}
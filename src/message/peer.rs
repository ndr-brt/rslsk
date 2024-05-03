use crate::message::unpack::Unpack;

pub struct PeerInit {
    pub username: String,
    pub connection_type: String, // TODO: this could be an enum
    pub token: u32
}

pub struct FileSearchResponse {

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
use crate::message::pack::Pack;

pub struct TransferRequest {
    direction: u32,
    token: u32,
    filename: String
}

impl Pack for TransferRequest {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(40u32.pack());
        bytes.extend(0u32.pack());
        bytes.extend(3u32.pack());
        bytes.extend(self.filename.pack());
        return bytes;
    }
}
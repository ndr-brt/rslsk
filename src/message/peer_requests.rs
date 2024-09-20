use crate::message::pack::Pack;

pub struct QueueUpload {
    pub(crate) filename: String
}

impl Pack for QueueUpload {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(43u32.pack());
        bytes.extend(self.filename.pack());
        return bytes;
    }
}
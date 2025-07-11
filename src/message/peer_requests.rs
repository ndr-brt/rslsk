use std::fmt::{Display, Formatter};

use crate::message::pack::Pack;

#[derive(Debug)]
pub enum PeerRequests {
    QueueUpload(QueueUpload),
    UserInfoResponse(UserInfoResponse)
}

impl Pack for PeerRequests {
    fn pack(&self) -> Vec<u8> {
        return match self {
            PeerRequests::QueueUpload(message) => message.pack(),
            PeerRequests::UserInfoResponse(message) => message.pack(),
        };

    }
}

#[derive(Debug)]
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

impl Display for QueueUpload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct UserInfoResponse {
    pub(crate) description: String,
    pub(crate) has_picture: bool,
    pub(crate) total_upload: u32,
    pub(crate) queue_size: u32,
    pub(crate) slots_free: bool
}

impl Pack for UserInfoResponse {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(16u32.pack());
        bytes.extend(self.description.pack());
        bytes.extend(self.has_picture.pack());
        bytes.extend(self.total_upload.pack());
        bytes.extend(self.queue_size.pack());
        bytes.extend(self.slots_free.pack());
        return bytes;
    }
}

impl Display for UserInfoResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
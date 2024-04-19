use crate::message::pack::Pack;

pub struct LoginRequest {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl Pack for LoginRequest {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(1u32.pack());
        bytes.extend(self.username.pack());
        bytes.extend(self.password.pack());
        bytes.extend(160u32.pack());
        return bytes;
    }
}
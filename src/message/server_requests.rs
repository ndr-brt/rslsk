use crate::message::pack::Pack;

pub enum ServerRequests {
    LoginRequest(LoginRequest)
}

impl Pack for ServerRequests {
    fn pack(&self) -> Vec<u8> {
        return match self {
            ServerRequests::LoginRequest(message) => message.pack()
        };

    }
}

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
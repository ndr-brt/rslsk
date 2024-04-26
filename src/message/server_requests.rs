use crate::message::pack::Pack;

#[derive(Debug, PartialEq)]
pub enum ServerRequests {
    LoginRequest(LoginRequest),
    FileSearch(FileSearch)
}

impl Pack for ServerRequests {
    fn pack(&self) -> Vec<u8> {
        return match self {
            ServerRequests::LoginRequest(message) => message.pack(),
            ServerRequests::FileSearch(message) => message.pack()
        };

    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct FileSearch {
    pub(crate) token: u32,
    pub(crate) query: String
}

impl Pack for FileSearch {
    fn pack(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(26u32.pack());
        bytes.extend(self.token.pack());
        bytes.extend(self.query.pack());
        return bytes;
    }
}
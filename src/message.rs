use buf_redux::Buffer;
use crate::message::Message::LoginRequest;

pub(crate) enum Message {
    LoginRequest(&'static str, &'static str)
}

impl Message {
    pub fn as_buffer(&self) -> Buffer {
        match self {
            LoginRequest(username, password) => {
                let message_type: u32 = 1;
                let username_bytes = username.as_bytes();
                let password_bytes = password.as_bytes();
                let unknown_field: u32 = 160;
                let another_unknown_field: u32 = 0x11;
                let cred = format!("{}{}", username, password);
                println!("credentials: {}", cred);
                let computed = md5::compute(format!("{}{}", username, password));
                let computed_string = format!("{:x}", computed);
                println!("hex credentials: {}", computed_string.as_str());

                let mut message = Buffer::new();
                message.push_bytes(&message_type.to_le_bytes());
                let username_len: u32 = username_bytes.len() as u32;
                message.push_bytes(&username_len.to_le_bytes());
                message.push_bytes(username_bytes);
                let password_len: u32 = password_bytes.len() as u32;
                message.push_bytes(&password_len.to_le_bytes());
                message.push_bytes(password_bytes);
                message.push_bytes(&unknown_field.to_le_bytes());
                let hex_credentials_len: u32 = computed_string.as_str().as_bytes().len() as u32;
                message.push_bytes(&hex_credentials_len.to_le_bytes());
                message.push_bytes(&computed_string.as_str().as_bytes());
                message.push_bytes(&another_unknown_field.to_le_bytes());

                let mut complete = Buffer::new();
                let message_len: u32 = message.len() as u32;
                complete.push_bytes(&message_len.to_le_bytes());
                complete.push_bytes(message.buf());

                complete
            }
        }
    }
}
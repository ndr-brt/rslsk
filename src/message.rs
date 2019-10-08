use buf_redux::Buffer;

struct P2PMessage {
    buffer: Buffer
}

impl P2PMessage {
    fn new() -> Self {
        P2PMessage { buffer: Buffer::new() }
    }

    fn append_u32(mut self, value: u32) -> Self {
        self.buffer.push_bytes(&value.to_le_bytes());
        self
    }

    fn append_string(mut self, value: &str) -> Self {
        let length = value.len() as u32;
        self.buffer.push_bytes(&length.to_le_bytes());
        self.buffer.push_bytes(value.as_bytes());
        self
    }

    fn to_buffer(&self) -> Buffer {
        let mut complete = Buffer::new();
        let buffer_len: u32 = self.buffer.len() as u32;
        complete.push_bytes(&buffer_len.to_le_bytes());
        complete.push_bytes(self.buffer.buf());

        complete
    }
}


pub(crate) trait Message {
    fn as_buffer(&self) -> Buffer;
}
impl dyn Message {
    pub fn login_request(username: &'static str, password: &'static str) -> LoginRequest {
        LoginRequest { username, password }
    }
}

pub struct LoginRequest {
    username: &'static str,
    password: &'static str,
}

impl Message for LoginRequest {
    fn as_buffer(&self) -> Buffer {
        let cred = format!("{}{}", self.username, self.password);
        println!("credentials: {}", cred);
        let computed = md5::compute(format!("{}{}", self.username, self.password));
        let computed_string = format!("{:x}", computed);
        println!("hex credentials: {}", computed_string.as_str());

        P2PMessage::new()
            .append_u32(1)
            .append_string(self.username)
            .append_string(self.password)
            .append_u32(160)
            .append_string(computed_string.as_str())
            .append_u32(17)
            .to_buffer()
    }
}
use buffer_redux::Buffer;
use super::slsk_buffer::SlskBuffer;


pub(crate) trait Message: Send {
    fn as_buffer(&self) -> Buffer;
}
impl dyn Message {
    pub fn login_request(username: &'static str, password: &'static str) -> Box<LoginRequest> {
        Box::new(LoginRequest { username, password })
    }
}

pub struct LoginRequest {
    username: &'static str,
    password: &'static str,
}

impl Message for LoginRequest {
    fn as_buffer(&self) -> Buffer {
        let md5 = md5::compute(format!("{}{}", self.username, self.password));

        SlskBuffer::new()
            .append_u32(1)
            .append_string(self.username)
            .append_string(self.password)
            .append_u32(160)
            .append_string(format!("{:x}", md5).as_str())
            .append_u32(17)
            .to_buffer()
    }
}
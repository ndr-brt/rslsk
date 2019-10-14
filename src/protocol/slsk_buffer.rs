use buf_redux::Buffer;

pub struct SlskBuffer {
    buffer: Buffer
}

impl SlskBuffer {
    pub fn new() -> Self {
        SlskBuffer { buffer: Buffer::new() }
    }

    pub fn append_u32(mut self, value: u32) -> Self {
        self.buffer.push_bytes(&value.to_le_bytes());
        self
    }

    pub fn append_string(mut self, value: &str) -> Self {
        let length = value.len() as u32;
        self.buffer.push_bytes(&length.to_le_bytes());
        self.buffer.push_bytes(value.as_bytes());
        self
    }

    pub fn raw_buffer(&self) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.push_bytes(self.buffer.buf());
        buffer
    }

    pub fn to_buffer(&self) -> Buffer {
        let mut complete = Buffer::new();
        let buffer_len: u32 = self.buffer.len() as u32;
        complete.push_bytes(&buffer_len.to_le_bytes());
        complete.push_bytes(self.buffer.buf());

        complete
    }
}
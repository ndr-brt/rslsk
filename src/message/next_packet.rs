use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;

pub(crate) trait NextPacket {
    async fn next_packet(&mut self) -> Result<Vec<u8>, String>;
}

impl NextPacket for OwnedReadHalf {
    async fn next_packet(&mut self) -> Result<Vec<u8>, String> {
        let mut length_buffer: [u8; 4] = [0, 0, 0, 0];
        match self.read_exact(&mut length_buffer).await {
            Ok(_len) => (),
            Err(_err) => {
                return Err(format!("cannot read packet length: {}", _err));
            },
        }

        let length = u32::from_le_bytes(length_buffer);
        let mut bytes: Vec<u8> = vec![0; length as usize];
        let _ = self.read_exact(&mut bytes).await;
        Ok(bytes)
    }
}
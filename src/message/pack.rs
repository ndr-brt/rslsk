pub trait Pack: Sized {
    fn pack(&self) -> Vec<u8>;
}

impl Pack for u32 {
    fn pack(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Pack for u8 {
    fn pack(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl Pack for bool {
    fn pack(&self) -> Vec<u8> {
        if self.eq(&true) {
            vec![1]
        } else {
            vec![0]
        }
    }
}

impl Pack for String {
    fn pack(&self) -> Vec<u8> {
        let length = self.len() as u32;

        let mut vec = vec!();
        vec.extend(length.pack());
        vec.extend(self.as_bytes());
        vec
    }
}

impl Pack for Vec<u8> {
    fn pack(&self) -> Vec<u8> {
        let length: u32 = self.len().try_into().unwrap_or(u32::MAX);
        let mut bytes: Vec<u8> = vec![];
        bytes.extend(length.pack());
        let count: u32 = 1;
        for i in self {
            bytes.extend(i.pack());
            if count == u32::MAX {
                break;
            }
        }
        bytes
    }
}
use flate2::{Decompress, FlushDecompress};

use crate::message::unpack::Unpack;

pub struct PeerInit {
    pub username: String,
    pub connection_type: String, // TODO: this could be an enum
    pub token: u32
}

impl Unpack for PeerInit {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        PeerInit {
            username: <String>::unpack(bytes),
            connection_type: <String>::unpack(bytes),
            token: <u32>::unpack(bytes)
        }
    }
}

pub struct FileSearchResponse {
    pub username: String,
    pub token: u32,
    pub results: Vec<ResultItem>,
    pub slot_free: bool,
    pub avg_speed: u32,
    pub queue_length: u32,
    pub private_results: Vec<ResultItem>
}

impl Unpack for FileSearchResponse {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let mut uncompressed = Vec::with_capacity(bytes.len() * 4096);
        Decompress::new(true)
            .decompress_vec(bytes.as_slice(), &mut uncompressed, FlushDecompress::Sync)
            .unwrap();
        let username = <String>::unpack(&mut uncompressed);
        let token = <u32>::unpack(&mut uncompressed);
        let result_count = <u32>::unpack(&mut uncompressed);

        let mut results: Vec<ResultItem> = vec![];
        for _ in 0..result_count {
            results.push(<ResultItem>::unpack(&mut uncompressed));
        }

        let slot_free = <bool>::unpack(&mut uncompressed);
        let avg_speed = <u32>::unpack(&mut uncompressed);
        let queue_length = <u32>::unpack(&mut uncompressed);

        // TODO: private results still not supported
        // if uncompressed.len() > 0 {
        //     let _unknown = <u32>::unpack(&mut uncompressed);
        //     let mut private_results: Vec<ResultItem> = vec![];
        //     let private_result_count = <u32>::unpack(&mut uncompressed);
        //
        //     for _ in [..private_result_count] {
        //         private_results.push(<ResultItem>::unpack(&mut uncompressed));
        //     }
        //
        //     FileSearchResponse { username, token, results, slot_free, avg_speed, queue_length, private_results }
        // } else {
        // }

        FileSearchResponse { username, token, results, slot_free, avg_speed, queue_length, private_results: vec![] }

    }
}

pub struct ResultItem {
    pub filename: String,
    pub file_size: u64,
    pub file_extension: String,
    pub attributes: Vec<FileAttribute>
}

impl Unpack for ResultItem {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let _code = <u8>::unpack(bytes);
        let filename = <String>::unpack(bytes);
        let file_size = <u64>::unpack(bytes);
        let file_extension = <String>::unpack(bytes);
        let number_of_attributes = <u32>::unpack(bytes);
        let mut attributes: Vec<FileAttribute> = vec![];
        for _ in 0..number_of_attributes {
            attributes.push(<FileAttribute>::unpack(bytes));
        }
        ResultItem { filename, file_size, file_extension, attributes }
    }
}

pub struct FileAttribute {
    pub code: u32,
    pub value: u32
}

impl Unpack for FileAttribute {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let code = <u32>::unpack(bytes);
        let value = <u32>::unpack(bytes);
        FileAttribute { code, value }
    }
}

pub struct UserInfoRequest {

}

impl Unpack for UserInfoRequest {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        UserInfoRequest {

        }
    }

}

#[derive(Debug)]
pub struct UploadFailed {
    filename: String
}

impl Unpack for UploadFailed {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        UploadFailed {
            filename: <String>::unpack(bytes),
        }
    }
}

use crate::message::unpack::Unpack;

pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    ip: Option<u32>,
    hash: Option<String>,
    is_supporter: Option<bool>
}

pub struct RoomList {
    pub number_of_rooms: u32
}

impl Unpack for LoginResponse {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let success = <bool>::unpack(bytes);
        let message = <String>::unpack(bytes);

        if success {
            LoginResponse {
                success,
                message,
                ip: Some(<u32>::unpack(bytes)),
                hash: Some(<String>::unpack(bytes)),
                is_supporter: Some(<bool>::unpack(bytes)),
            }
        } else {
            LoginResponse {
                success,
                message,
                ip: None,
                hash: None,
                is_supporter: None,
            }
        }
    }
}

impl Unpack for RoomList {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let number_of_rooms = <u32>::unpack(bytes);

        RoomList {
            number_of_rooms
        }
    }
}
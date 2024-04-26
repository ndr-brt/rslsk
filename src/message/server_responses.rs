use std::net::Ipv4Addr;
use crate::message::unpack::Unpack;
#[derive(Clone, Debug)]
pub enum ServerResponses {
    LoginResponse(LoginResponse)
}

#[derive(Clone, Debug)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub(crate) ip: Option<Ipv4Addr>,
    pub(crate) hash: Option<String>,
    pub(crate) is_supporter: Option<bool>
}

pub struct RoomList {
    pub number_of_rooms: u32
}

pub struct PrivilegedUsers {
    pub number_of_users: u32,
    pub users: Vec<String>
}

pub struct ParentMinSpeed {
    pub speed: u32
}

pub struct ParentSpeedRatio {
    pub ratio: u32
}

pub struct WishlistInterval {
    pub interval: u32
}

pub struct ExcludedSearchPhrases {
    pub count: u32,
    pub phrases: Vec<String>
}

impl Unpack for LoginResponse {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let success = <bool>::unpack(bytes);
        let message = <String>::unpack(bytes);

        if success {
            LoginResponse {
                success,
                message,
                ip: Some(<Ipv4Addr>::unpack(bytes)),
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

        RoomList { number_of_rooms }
    }
}

impl Unpack for PrivilegedUsers {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let number_of_users = <u32>::unpack(bytes);

        let users: Vec<String> = [0..number_of_users].iter().map(|_index| <String>::unpack(bytes)).collect();

        PrivilegedUsers { number_of_users, users }
    }
}

impl Unpack for ParentMinSpeed {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let speed = <u32>::unpack(bytes);

        ParentMinSpeed { speed }
    }
}

impl Unpack for ParentSpeedRatio {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let ratio = <u32>::unpack(bytes);

        ParentSpeedRatio { ratio }
    }
}

impl Unpack for WishlistInterval {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let interval = <u32>::unpack(bytes);

        WishlistInterval { interval }
    }
}

impl Unpack for ExcludedSearchPhrases {
    fn unpack(bytes: &mut Vec<u8>) -> Self {
        let count = <u32>::unpack(bytes);

        let phrases: Vec<String> = [0..count].iter().map(|_index| <String>::unpack(bytes)).collect();

        ExcludedSearchPhrases { count, phrases }
    }
}
#[derive(Debug, PartialEq)]
pub enum Event {
    LoginSucceeded { message: String },
    LoginFailed { message: String }
}
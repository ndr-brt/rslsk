#[derive(Debug, PartialEq)]
pub enum Event {
    LoginSucceeded { message: String }
}
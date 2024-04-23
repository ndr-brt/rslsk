use tokio::sync::oneshot::Sender;

pub enum Command {
    Login { username: String, password: String, tx: Sender<crate::Event> }
}
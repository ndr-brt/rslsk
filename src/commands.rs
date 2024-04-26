use tokio::sync::oneshot;

pub enum Command {
    Login { username: String, password: String, tx: oneshot::Sender<crate::Event> },
    Search { query: String, tx: oneshot::Sender<crate::Event> }
}
use tokio::sync::oneshot;
use crate::events::SearchResultItem;

pub enum Command {
    Login { username: String, password: String, tx: oneshot::Sender<crate::Event> },
    Search { query: String, tx: oneshot::Sender<crate::Event> },
    Download { item: SearchResultItem, destination: String, tx: oneshot::Sender<crate::Event> }
}
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    LoginSucceeded { message: String },
    LoginFailed { message: String },
    SearchResultReceived { recv: mpsc::Receiver<SearchResultItem> },
    DownloadQueued { message: String },
    DownloadFailed { message: String }
}

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub username: String,
    pub filename: String
}
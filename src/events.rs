use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    LoginSucceeded { message: String },
    LoginFailed { message: String },
    SearchResultReceived { recv: mpsc::Receiver<SearchResultItem> },
    DownloadFailed { message: String }
}

#[derive(Debug)]
pub struct SearchResultItem {
    pub username: String,
    pub filename: String
}
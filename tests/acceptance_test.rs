use std::fmt::Debug;
use std::fs::Metadata;
use std::path::Path;
use std::time::Duration;
use tokio::fs;
use tokio::time::{sleep, timeout};
use rslsk::events::SearchResultItem;

use rslsk::Slsk;

const USERNAME: &str = "ginetto";
const PASSWORD: &str = "ginetto";

#[tokio::test]
async fn login() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from(USERNAME), String::from(PASSWORD))
        .await.unwrap();

    let result = slsk.login().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn search() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from(USERNAME), String::from(PASSWORD))
        .await.unwrap();

    slsk.login().await.unwrap();

    let result = slsk.search(String::from("leatherface")).await;

    assert!(result.is_ok());

    let mut receiver = result.unwrap();
    let item = timeout(Duration::from_secs(10), receiver.recv()).await.unwrap().unwrap();
    assert!(item.filename.to_lowercase().contains("leatherface"));
}

#[tokio::test]
async fn download() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from(USERNAME), String::from(PASSWORD))
        .await.unwrap();

    slsk.login().await.unwrap();

    let result = slsk.search(String::from("leatherface")).await;

    assert!(result.is_ok());

    let mut receiver = result.unwrap();
    let item = timeout(Duration::from_secs(10), receiver.recv()).await.unwrap().unwrap();

    let filename = item.clone().filename;
    let downloaded_file = format!("/tmp/{}", filename);
    slsk.download(item, downloaded_file.clone()).await.unwrap();

    match wait_for_file_creation(downloaded_file, Duration::from_secs(10)).await {
        Ok(metadata) => {
            assert!(metadata.is_file())
        }
        Err(err) => panic!("Failed to detect file creation: {}", err),
    }


}


async fn wait_for_file_creation<P: AsRef<Path>>(path: P, timeout: Duration) -> Result<Metadata, &'static str> {
    let start = tokio::time::Instant::now();
    while start.elapsed() < timeout {
        let result = fs::metadata(&path).await;
        if result.is_ok() {
            return Ok(result.unwrap());
        }
        sleep(Duration::from_secs(1)).await;
    }
    Err("File creation timed out")
}
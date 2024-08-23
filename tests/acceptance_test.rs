use std::time::Duration;
use tokio::fs;
use tokio::time::timeout;

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

// #[tokio::test]
// async fn download() {
//     let slsk = Slsk::connect("server.slsknet.org", 2242, String::from(USERNAME), String::from(PASSWORD))
//         .await.unwrap();
//
//     slsk.login().await.unwrap();
//
//     let result = slsk.search(String::from("leatherface")).await;
//
//     assert!(result.is_ok());
//
//     let mut receiver = result.unwrap();
//     let item = timeout(Duration::from_secs(10), receiver.recv()).await.unwrap().unwrap();
//
//     let filename = item.clone().filename;
//     let downloaded_file = format!("/tmp/{}", filename);
//     slsk.download(item, downloaded_file.clone()).await.unwrap();
//
//     let file_metadata = fs::metadata(downloaded_file).await.unwrap();
//     assert!(file_metadata.is_file());
// }
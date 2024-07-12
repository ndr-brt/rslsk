use std::time::Duration;

use tokio::time::timeout;

use rslsk::Slsk;

#[tokio::test]
async fn login() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from("ginogino"), String::from("ginogino"))
        .await.unwrap();

    let result = slsk.login().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn search() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from("ginogino"), String::from("ginogino"))
        .await.unwrap();

    slsk.login().await.unwrap();

    let result = slsk.search(String::from("leatherface")).await;

    assert!(result.is_ok());

    let mut receiver = result.unwrap();
    let item = timeout(Duration::from_secs(10), receiver.recv()).await.unwrap().unwrap();
    assert!(item.filename.to_lowercase().contains("leatherface"));
}
//
// #[tokio::test]
// async fn download() {
//     let slsk = Slsk::connect("server.slsknet.org", 2242, String::from("ginogino"), String::from("ginogino"))
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
//     slsk.download(item, String::from("/tmp")).await.unwrap();
// }
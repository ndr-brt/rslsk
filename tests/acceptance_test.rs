use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;

use tokio::time::sleep;

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

    let search_token = slsk.search(String::from("leatherface")).await.expect("valid search token");

    let results = await_for(
        || slsk.clone().get_search_results(search_token),
        |r| r.len() > 0,
        Duration::from_secs(10)
    ).await;

    match results {
        Ok(items) => {
            assert_ne!(items.len(), 0);
            println!("LEN {}", items.len().clone());
            assert!(items.iter().any(|item| item.filename.contains("leatherface")));
        },
        Err(err) => panic!("{}", err)
    }
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
//     match wait_for_file_creation(downloaded_file, Duration::from_secs(10)).await {
//         Ok(metadata) => {
//             assert!(metadata.is_file())
//         }
//         Err(err) => panic!("Failed to detect file creation: {}", err),
//     }
//
//
// }
//
//
// async fn wait_for_file_creation<P: AsRef<Path>>(path: P, timeout: Duration) -> Result<Metadata, &'static str> {
//     let start = tokio::time::Instant::now();
//     while start.elapsed() < timeout {
//         let result = fs::metadata(&path).await;
//         if result.is_ok() {
//             return Ok(result.unwrap());
//         }
//         sleep(Duration::from_secs(1)).await;
//     }
//     Err("File creation timed out")
// }

async fn await_for<A, Fut, O, F>(action: A, polling: F, timeout: Duration) -> Result<O, String>
where
    A: Fn() -> Fut,
    Fut: Future<Output = O>,
    F: for<'a> Fn(&'a O) -> bool,
{
    let start = tokio::time::Instant::now();
    while start.elapsed() < timeout {
        let a = action().await;
        if polling(&a) {
            return Ok(a);
        }
        sleep(Duration::from_secs(1)).await;
    }

    Err("Timed out".into())
}
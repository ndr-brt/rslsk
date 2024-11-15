use std::future::Future;
use std::time::Duration;

use tokio::fs;
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
        |r| r.iter().any(|item| item.filename.contains("leatherface")),
        Duration::from_secs(10)
    ).await;

    match results {
        Ok(items) => {
            assert_ne!(items.len(), 0);
            assert!(items.iter().any(|item| item.filename.contains("leatherface")));
        },
        Err(err) => panic!("{}", err)
    }
}

#[tokio::test]
async fn download() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from(USERNAME), String::from(PASSWORD))
        .await.unwrap();

    slsk.login().await.unwrap();

    let search_token = slsk.search(String::from("leatherface")).await.expect("valid search token");
    let search_results = await_for(
        || slsk.clone().get_search_results(search_token),
        |r| r.len() > 0,
        Duration::from_secs(10)
    ).await.expect("search result items");

    let item = search_results.first().expect("item");

    let filename = item.clone().filename;
    let downloaded_file = format!("/tmp/{}", filename.split("\\").last().unwrap());
    slsk.download(item.clone(), downloaded_file.clone()).await.expect("Download to have been queued");

    let metadata = await_for(
        || fs::metadata(&downloaded_file),
        |result| result.is_ok(),
        Duration::from_secs(10)
    ).await.unwrap_or_else(|_| panic!("Failed to find file {}", downloaded_file.clone()));

    match metadata {
        Ok(metadata) => {
            assert!(metadata.is_file())
        }
        Err(err) => panic!("Failed to detect file creation: {}", err),
    }
}

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
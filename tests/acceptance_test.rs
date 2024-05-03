use std::time::Duration;
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
    tokio::time::sleep(Duration::from_secs(10)).await;

    // let item = result.unwrap().recv().await.unwrap();
    // assert!(item.filename.contains("leatherface"));
}
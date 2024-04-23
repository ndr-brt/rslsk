use rslsk::Slsk;

#[tokio::test]
async fn login() {
    let slsk = Slsk::connect("server.slsknet.org", 2242, String::from("ginogino"), String::from("ginogino"))
        .await.unwrap();

    let result = slsk.login().await;

    assert!(result.is_ok());
}
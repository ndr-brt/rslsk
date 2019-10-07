use rslsk::Slsk;

#[test]
fn login() {
    let slsk = Slsk::new("server.slsknet.org", 2242, "ginogino", "ginogino");
    let result = slsk.login();

    assert!(result.is_ok());
}
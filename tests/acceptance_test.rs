use rslsk::Slsk;

#[test]
fn login() {
    match Slsk::connect("server.slsknet.org", 2242, "ginogino", "ginogino") {
        Ok(slsk) => {
            let result = slsk.login();

            assert!(result.is_ok());
        },
        Err(_e) => unreachable!()
    }

}
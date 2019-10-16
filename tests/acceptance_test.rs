use rslsk::Slsk;

#[test]
#[ignore]
fn login() {
    match Slsk::connect("server.slsknet.org", 2242, "ginogino", "ginogino") {
        Ok(mut slsk) => {
            match slsk.login() {
                Ok(login_responded) => {
                    assert_eq!(login_responded.success, true);
                },
                Err(e) => unreachable!(e)
            }
        },
        Err(e) => unreachable!(e)
    }

}
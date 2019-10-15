use rslsk::Slsk;

#[test]
#[ignore]
fn login() {
    match Slsk::connect("server.slsknet.org", 2242, "ginogino", "ginogino") {
        Ok(slsk) => {

            match slsk.login() {
                Ok(login_responded) => {
                    assert_eq!(login_responded.success, true);
                },
                Err(_e) => unreachable!()
            }

        },
        Err(e) => unreachable!()
    }

}
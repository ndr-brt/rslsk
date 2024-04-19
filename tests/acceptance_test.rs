use std::thread;
use std::time::Duration;
use rslsk::Slsk;

#[test]
fn login() {
    match Slsk::connect("server.slsknet.org", 2242, String::from("ginogino"), String::from("ginogino")) {
        Ok(slsk) => {
            let result = slsk.login();

            let duration = Duration::from_secs(8);
            thread::sleep(duration);

            assert!(result.is_ok());

            // wait for login response, verify success is true


        },
        Err(_e) => unreachable!()
    }

}
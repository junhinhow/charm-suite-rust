//! Detection from the real process environment.
//!
//! This is the only test in its binary, so changing variables does not race
//! with other tests. Every variable read by `Profile::detect` is set.

use charm_a11y::Profile;

#[test]
fn detect_uses_process_environment() {
    std::env::set_var("NO_COLOR", "");
    std::env::set_var("CLICOLOR", "");
    std::env::set_var("TERM", "xterm-256color");
    std::env::set_var("ASCII", "");
    std::env::set_var("A11Y_ASCII", "");
    std::env::set_var("A11Y_HIGH_CONTRAST", "");
    std::env::set_var("A11Y_REDUCED_MOTION", "");
    std::env::set_var("A11Y_SCREEN_READER", "");
    std::env::set_var("COLUMNS", "90");

    let want = Profile {
        unicode: true,
        width: 90,
        ..Profile::default()
    };
    assert_eq!(Profile::detect(), want);
}

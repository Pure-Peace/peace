#[allow(unused_imports)]
use super::*;

#[test]
fn test_login_reply() {
    assert_eq!(
        login_reply(LoginFailed::InvalidCredentials),
        vec![5, 0, 0, 4, 0, 0, 0, 255, 255, 255, 255]
    )
}

#[test]
fn test_login_notfication() {
    assert_eq!(
        notification("hello"),
        vec![24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111]
    )
}

#[test]
fn test_send_message() {
    assert_eq!(
        send_message("PurePeace", 1001, "hello", "osu"),
        vec![
            7, 0, 0, 27, 0, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 11, 5, 104, 101,
            108, 108, 111, 11, 3, 111, 115, 117, 233, 3, 0, 0
        ]
    )
}

#[test]
fn test_change_username() {
    assert_eq!(
        change_username("PurePeace", "peppy"),
        vec![
            9, 0, 0, 20, 0, 0, 0, 11, 18, 80, 117, 114, 101, 80, 101, 97, 99, 101, 62, 62, 62, 62,
            112, 101, 112, 112, 121
        ]
    )
}

#[test]
fn test_rtx() {
    assert_eq!(
        rtx("Peace"),
        vec![105, 0, 0, 7, 0, 0, 0, 11, 5, 80, 101, 97, 99, 101]
    )
}

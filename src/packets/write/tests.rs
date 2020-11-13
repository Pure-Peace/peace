#[allow(unused_imports)]
use super::*;



#[test]
fn test_login_reply() {
    assert_eq!(
        login_reply(LoginReply::InvalidCredentials),
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

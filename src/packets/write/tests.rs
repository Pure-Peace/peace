#![allow(unused_imports)]
use super::*;
use crate::constants::{LoginFailed, LoginSuccess};

use actix_rt;

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

#[actix_rt::test]
async fn test_send_message() {
    assert_eq!(
        send_message("PurePeace", 1001, "hello", "osu").await,
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

#[test]
fn test_login() {
    let resp = PacketBuilder::new();
    let resp = resp
        .add(login_reply(LoginSuccess::Verified(1009)))
        .add(protocol_version(19))
        .add(notification("Welcome to Peace!"))
        .add(main_menu_icon(
            "https://i.kafuu.pro/welcome.png",
            "https://www.baidu.com",
        ))
        .add(silence_end(0))
        .add(channel_info_end());
    assert_eq!(
        resp.write_out(),
        vec![
            5, 0, 0, 4, 0, 0, 0, 241, 3, 0, 0, 75, 0, 0, 4, 0, 0, 0, 19, 0, 0, 0, 24, 0, 0, 19, 0,
            0, 0, 11, 17, 87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 80, 101, 97, 99, 101,
            33, 76, 0, 0, 55, 0, 0, 0, 11, 53, 104, 116, 116, 112, 115, 58, 47, 47, 105, 46, 107,
            97, 102, 117, 117, 46, 112, 114, 111, 47, 119, 101, 108, 99, 111, 109, 101, 46, 112,
            110, 103, 124, 104, 116, 116, 112, 115, 58, 47, 47, 119, 119, 119, 46, 98, 97, 105,
            100, 117, 46, 99, 111, 109, 92, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 89, 0, 0, 0, 0, 0, 0
        ]
    )
}

#[actix_rt::test]
async fn test_write_i32_list() {
    //let list = utils::write_int_list(&vec![1001, 1002, 1003]).await;
    let list = user_presence_bundle(&vec![1001, 1002, 1003]).await;
    println!("{:?}", list);
}

#![allow(unused_imports)]
use crate::{LoginFailed, LoginSuccess};

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

#[test]
fn test_login() {
    let resp = PacketBuilder::new();
    let resp = resp
        .add(login_reply(LoginSuccess::Verified(1009)))
        .add(protocol_version(19))
        .add(notification("Welcome to Peace!"))
        .add(main_menu_icon(
            "https://i.kafuu.pro/welcome.png|https://www.baidu.com",
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

#[test]
fn test_write_i32_list() {
    //let list = utils::write_int_list(&vec![1001, 1002, 1003]);
    let list = user_presence_bundle(&vec![1001, 1002, 1003]);
    println!("{:?}", list);
}

#[test]
fn test_write_u32_i32() {
    let int_u32 = super::utils::write(536870912 as u32);
    let int_i32 = super::utils::write(536870912);

    println!("{:?} {:?}", int_u32, int_i32);
}

#[test]
fn test_user_presence() {
    let data = user_presence(5, "PurePeace", 8, 48, 1, 1.0, 1.0, 666);
    println!("{}", data.len());
    assert_eq!(
        data,
        [
            83, 0, 0, 30, 0, 0, 0, 5, 0, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32,
            48, 1, 0, 0, 128, 63, 0, 0, 128, 63, 154, 2, 0, 0
        ]
    )
}

#[test]
fn test_user_stats() {
    let data = user_stats(
        5,
        1,
        "idle",
        "asdqwezxcasdqwezxcasdqwezxcasdqw",
        0,
        0,
        1,
        10000000,
        0.998,
        10000,
        100000000,
        100,
        10000,
    );
    println!("{}", data.len());
    assert_eq!(
        data,
        [
            11, 0, 0, 84, 0, 0, 0, 5, 0, 0, 0, 1, 11, 4, 105, 100, 108, 101, 11, 32, 97, 115, 100,
            113, 119, 101, 122, 120, 99, 97, 115, 100, 113, 119, 101, 122, 120, 99, 97, 115, 100,
            113, 119, 101, 122, 120, 99, 97, 115, 100, 113, 119, 0, 0, 0, 0, 0, 1, 0, 0, 0, 128,
            150, 152, 0, 0, 0, 0, 0, 40, 131, 35, 60, 16, 39, 0, 0, 0, 225, 245, 5, 0, 0, 0, 0,
            100, 0, 0, 0, 16, 39
        ]
    )
}

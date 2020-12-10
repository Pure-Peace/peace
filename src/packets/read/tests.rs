#![allow(unused_imports)]
use super::*;

#[test]
fn test_read_header() {
    println!(
        "p1: {:?}\np2: {:?}",
        PacketReader::read_header(vec![4, 0, 0, 0, 0, 0, 0]),
        PacketReader::read_header(vec![24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111])
    );
}

#[test]
fn test_header() {
    let mut p1 = PacketReader::from_vec(vec![4, 0, 0, 0, 0, 0, 0]);
    let mut p2 = PacketReader::from_vec(vec![24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111]);

    print!("p1: {:?}; ", p1.next());
    println!("idx: {:?}", p1.index);
    print!("p2: {:?}; ", p2.next());
    println!("idx: {:?}", p2.index);
}

#[test]
fn test_mutiple_headers() {
    // Mutiple packet headers read
    let mut p3 = PacketReader::from_vec(vec![
        24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111, 4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 7, 0,
        0, 0, 11, 5, 104, 101, 108, 108, 111,
    ]);
    println!("p3 0: {:?}", p3.next());
    println!("p3 1: {:?}", p3.next());
    println!("p3 2: {:?}", p3.next());
    println!("p3 3 (outside): {:?}", p3.next());
    println!(
        "finish: {}; packet count: {}; payload count: {}",
        p3.finish, p3.packet_count, p3.payload_count
    );
}

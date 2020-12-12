#![allow(unused_imports)]
use super::*;
use actix_rt;

#[actix_rt::test]
async fn test_read_header() {
    println!(
        "p1: {:?}\np2: {:?}",
        PacketReader::read_header(vec![4, 0, 0, 0, 0, 0, 0]).await,
        PacketReader::read_header(vec![24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111]).await
    );
}

#[actix_rt::test]
async fn test_header() {
    let mut p1 = PacketReader::from_vec(vec![4, 0, 0, 0, 0, 0, 0]);
    let mut p2 = PacketReader::from_vec(vec![24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111]);

    print!("p1: {:?}; ", p1.next().await);
    println!("idx: {:?}", p1.index);
    print!("p2: {:?}; ", p2.next().await);
    println!("idx: {:?}", p2.index);
}

#[actix_rt::test]
async fn test_mutiple_headers() {
    // Mutiple packet headers read
    let mut p3 = PacketReader::from_vec(vec![
        24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111, 4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 7, 0,
        0, 0, 11, 5, 104, 101, 108, 108, 111,
    ]);
    println!("p3 0: {:?}", p3.next().await);
    println!("p3 1: {:?}", p3.next().await);
    println!("p3 2: {:?}", p3.next().await);
    println!("p3 3 (outside): {:?}", p3.next().await);
    println!(
        "finish: {}; packet count: {}; payload count: {}",
        p3.finish, p3.packet_count, p3.payload_count
    );
}

#[test]
fn test_read_uleb128() {
    assert_eq!(read_uleb128(&[0xE5, 0x8E, 0x26]), (624485, 3));
}

#[actix_rt::test]
async fn test_read_payload() {
    // It's a notification packet, content: Hello, World!💖
    let packet = vec![
        24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33,
        240, 159, 146, 150,
    ];
    let mut reader = PacketReader::from_vec(packet);
    let (id, payload) = reader.next().await.unwrap();

    let mut payload_reader = PayloadReader::new(payload.unwrap());
    let str_data = payload_reader.read_string().await;

    println!("{:?}: {}", id, str_data);
}

#[actix_rt::test]
async fn test_read_mutiple_packet_and_payloads() {
    let mut reader = PacketReader::from_vec(vec![
        4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108, 111, 44, 32, 87,
        111, 114, 108, 100, 33, 240, 159, 146, 150, 4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0, 11,
        16, 229, 147, 136, 229, 147, 136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145, 104, 0,
        0, 0, 0, 0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175, 187, 229, 143, 150, 229, 174, 140,
        228, 186, 134, 239, 188, 129, 239, 188, 129, 226, 156, 168,
    ]);
    while let Some((packet_id, payload)) = reader.next().await {
        print!("{:?}: ", packet_id);
        match payload {
            None => println!("Non-payload"),
            Some(payload) => {
                let mut payload_reader = PayloadReader::new(payload);
                println!("{}", payload_reader.read_string().await);
            }
        }
    }
}

#[actix_rt::test]
async fn test_read_integer() {
    let packet = vec![103, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0];
    let mut reader = PacketReader::from_vec(packet);
    let (id, payload) = reader.next().await.unwrap();

    let mut payload_reader = PayloadReader::new(payload.unwrap());
    let int_data = payload_reader.read_integer::<u32>().await;

    println!("{:?}: {}", id, int_data);
}

#[actix_rt::test]
async fn test_read_message() {
    let packet = vec![
        1, 0, 0, 20, 0, 0, 0, 11, 0, 11, 6, 228, 189, 160, 229, 165, 189, 11, 4, 35, 111, 115, 117,
        0, 0, 0, 0,
    ];
    let mut reader = PacketReader::from_vec(packet);
    let (id, payload) = reader.next().await.unwrap();

    let mut payload_reader = PayloadReader::new(payload.unwrap());
    let message = payload_reader.read_message().await;

    println!("{:?}: {:?}", id, message);
}

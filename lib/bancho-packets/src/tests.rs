mod packets_reading {
    use crate::{read_uleb128, BanchoMessage, PacketReader, PayloadReader};

    #[test]
    fn test_read_header() {
        println!(
            "p1: {:?}\np2: {:?}",
            PacketReader::read_header(&[4, 0, 0, 0, 0, 0, 0]),
            PacketReader::read_header(&[24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111])
        );
    }

    #[test]
    fn test_header() {
        let mut p1 = PacketReader::new(&[4, 0, 0, 0, 0, 0, 0]);
        let mut p2 = PacketReader::new(&[24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111]);

        print!("p1: {:?}; ", p1.next());
        println!("idx: {:?}", p1.index());
        print!("p2: {:?}; ", p2.next());
        println!("idx: {:?}", p2.index());
    }

    #[test]
    fn test_mutiple_headers() {
        // Mutiple packet headers read
        let mut p3 = PacketReader::new(&[
            24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111, 4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 7,
            0, 0, 0, 11, 5, 104, 101, 108, 108, 111,
        ]);
        println!("p3 0: {:?}", p3.next());
        println!("p3 1: {:?}", p3.next());
        println!("p3 2: {:?}", p3.next());
        println!("p3 3 (outside): {:?}", p3.next());
        println!("finish: {}; ", p3.is_finished());
    }

    #[test]
    fn test_read_uleb128() {
        assert_eq!(read_uleb128(&[0xE5, 0x8E, 0x26]), Some((624485, 3)));
    }

    #[test]
    fn test_read_payload() {
        // It's a notification packet, content: Hello, World!💖
        let packet = &[
            24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100,
            33, 240, 159, 146, 150,
        ];
        let mut reader = PacketReader::new(packet);
        let packet = reader.next().unwrap();

        let mut payload_reader = PayloadReader::new(packet.payload.unwrap());
        let str_data = payload_reader.read::<String>();

        println!("{:?}: {:?}", packet.id, str_data);
    }

    #[test]
    fn test_read_mutiple_packet_and_payloads() {
        let mut reader = PacketReader::new(&[
            4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108, 111, 44, 32, 87,
            111, 114, 108, 100, 33, 240, 159, 146, 150, 4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0,
            11, 16, 229, 147, 136, 229, 147, 136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145,
            104, 0, 0, 0, 0, 0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175, 187, 229, 143, 150,
            229, 174, 140, 228, 186, 134, 239, 188, 129, 239, 188, 129, 226, 156, 168,
        ]);
        while let Some(packet) = reader.next() {
            print!("{:?}: ", packet.id);
            match packet.payload {
                None => println!("Non-payload"),
                Some(payload) => {
                    let mut payload_reader = PayloadReader::new(payload);
                    println!("{:?}", payload_reader.read::<String>());
                }
            }
        }
    }

    #[test]
    fn test_read_integer() {
        let mut reader = PacketReader::new(&[103, 0, 0, 4, 0, 0, 0, 1, 0, 0, 0]);
        let packet = reader.next().unwrap();

        let mut payload_reader = PayloadReader::new(packet.payload.unwrap());
        let int_data = payload_reader.read::<u32>();

        println!("{:?}: {:?}", packet.id, int_data);
    }

    #[test]
    fn test_read_message() {
        let mut reader = PacketReader::new(&[
            1, 0, 0, 20, 0, 0, 0, 11, 0, 11, 6, 228, 189, 160, 229, 165, 189, 11, 4, 35, 111, 115,
            117, 0, 0, 0, 0,
        ]);
        let packet = reader.next().unwrap();

        let mut payload_reader = PayloadReader::new(packet.payload.unwrap());
        let message = payload_reader.read::<BanchoMessage>();

        println!("{:?}: {:?}", packet.id, message);
    }

    #[test]
    fn test_super_mutiple_packets() {
        let packet = &[
            24, 0, 0, 32, 0, 0, 0, 11, 30, 230, 172, 162, 232, 191, 142, 230, 130, 168, 239, 188,
            140, 233, 171, 152, 232, 180, 181, 231, 154, 132, 230, 146, 146, 230, 179, 188, 231,
            137, 185, 105, 0, 0, 7, 0, 0, 0, 11, 5, 80, 101, 97, 99, 101, 24, 0, 0, 44, 0, 0, 0,
            11, 42, 45, 32, 79, 110, 108, 105, 110, 101, 32, 85, 115, 101, 114, 115, 58, 32, 50,
            10, 45, 32, 87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 111, 115, 117, 33, 75,
            97, 102, 117, 117, 126, 126, 92, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 4, 0, 0, 0,
            232, 3, 0, 0, 75, 0, 0, 4, 0, 0, 0, 19, 0, 0, 0, 71, 0, 0, 4, 0, 0, 0, 39, 0, 0, 0, 83,
            0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0,
            16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 11, 0, 0, 46, 0, 0, 0, 232, 3, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 202, 7, 224, 54, 0, 0, 0, 0, 100, 112, 123, 63, 41, 0, 0, 0,
            135, 96, 87, 56, 0, 0, 0, 0, 1, 0, 0, 0, 7, 1, 89, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 64, 0,
            0, 6, 0, 0, 0, 11, 4, 35, 111, 115, 117, 64, 0, 0, 11, 0, 0, 0, 11, 9, 35, 97, 110,
            110, 111, 117, 110, 99, 101, 64, 0, 0, 8, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110,
            65, 0, 0, 27, 0, 0, 0, 11, 4, 35, 111, 115, 117, 11, 17, 75, 97, 102, 117, 117, 32,
            103, 108, 111, 98, 97, 108, 32, 99, 104, 97, 116, 2, 0, 65, 0, 0, 31, 0, 0, 0, 11, 9,
            35, 97, 110, 110, 111, 117, 110, 99, 101, 11, 16, 65, 110, 110, 111, 117, 110, 99, 101,
            32, 99, 104, 97, 110, 110, 101, 108, 2, 0, 65, 0, 0, 27, 0, 0, 0, 11, 6, 35, 99, 104,
            105, 110, 97, 11, 15, 67, 104, 105, 110, 97, 32, 99, 111, 109, 109, 117, 110, 105, 116,
            121, 1, 0, 65, 0, 0, 31, 0, 0, 0, 11, 8, 35, 101, 110, 103, 108, 105, 115, 104, 11, 17,
            69, 110, 103, 108, 105, 115, 104, 32, 99, 111, 109, 109, 117, 110, 105, 116, 121, 1, 0,
            65, 0, 0, 26, 0, 0, 0, 11, 6, 35, 97, 100, 109, 105, 110, 11, 14, 65, 114, 101, 32,
            121, 111, 117, 32, 97, 100, 109, 105, 110, 63, 2, 0, 65, 0, 0, 71, 0, 0, 0, 11, 6, 35,
            108, 111, 98, 98, 121, 11, 59, 84, 104, 105, 115, 32, 105, 115, 32, 116, 104, 101, 32,
            108, 111, 98, 98, 121, 32, 119, 104, 101, 114, 101, 32, 121, 111, 117, 32, 102, 105,
            110, 100, 32, 103, 97, 109, 101, 115, 32, 116, 111, 32, 112, 108, 97, 121, 32, 119,
            105, 116, 104, 32, 111, 116, 104, 101, 114, 115, 33, 1, 0, 65, 0, 0, 69, 0, 0, 0, 11,
            7, 35, 114, 97, 110, 107, 101, 100, 11, 56, 82, 97, 110, 107, 32, 114, 101, 113, 117,
            101, 115, 116, 115, 32, 109, 97, 112, 115, 32, 119, 105, 108, 108, 32, 98, 101, 32,
            112, 111, 115, 116, 101, 100, 32, 104, 101, 114, 101, 33, 32, 40, 73, 102, 32, 105,
            116, 115, 32, 114, 97, 110, 107, 101, 100, 46, 41, 1, 0, 72, 0, 0, 6, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 76, 0, 0, 51, 0, 0, 0, 11, 49, 104, 116, 116, 112, 115, 58, 47, 47, 105, 46,
            107, 97, 102, 117, 117, 46, 112, 114, 111, 47, 119, 101, 108, 99, 111, 109, 101, 46,
            112, 110, 103, 124, 104, 116, 116, 112, 115, 58, 47, 47, 107, 97, 102, 117, 117, 46,
            112, 114, 111, 83, 0, 0, 29, 0, 0, 0, 231, 3, 0, 0, 11, 8, 67, 104, 105, 110, 111, 66,
            111, 116, 24, 48, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3,
            0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 83, 0, 0, 30, 0, 0, 0, 232, 3, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101,
            97, 99, 101, 32, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
        ];
        let mut reader = PacketReader::new(packet);
        while let Some(packet) = reader.next() {
            println!("{:?}: {:?}", packet.id, packet.payload.unwrap_or(&[]));
        }
    }

    #[test]
    fn test_read_i32_list() {
        let payload = vec![4, 0, 233, 3, 0, 0, 234, 3, 0, 0, 235, 3, 0, 0, 236, 3, 0, 0];
        let mut payload_reader = PayloadReader::new(&payload);
        // read i32 list with i16 length
        let int_list = payload_reader.read::<Vec<i32>>();

        println!("{:?}", int_list);
        assert_eq!(int_list, Some(vec![1001, 1002, 1003, 1004]))
    }
}

mod packets_writing {
    use crate::*;

    #[test]
    fn test_login_reply() {
        assert_eq!(
            server_packet::login_reply(LoginFailed::InvalidCredentials),
            vec![5, 0, 0, 4, 0, 0, 0, 255, 255, 255, 255]
        )
    }

    #[test]
    fn test_login_notfication() {
        assert_eq!(
            server_packet::notification("hello"),
            vec![24, 0, 0, 7, 0, 0, 0, 11, 5, 104, 101, 108, 108, 111]
        )
    }

    #[test]
    fn test_send_message() {
        assert_eq!(
            server_packet::send_message("PurePeace", 1001, "hello", "osu"),
            vec![
                7, 0, 0, 27, 0, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101, 11, 5, 104,
                101, 108, 108, 111, 11, 3, 111, 115, 117, 233, 3, 0, 0
            ]
        )
    }

    #[test]
    fn test_change_username() {
        assert_eq!(
            server_packet::change_username("PurePeace", "peppy"),
            vec![
                9, 0, 0, 20, 0, 0, 0, 11, 18, 80, 117, 114, 101, 80, 101, 97, 99, 101, 62, 62, 62,
                62, 112, 101, 112, 112, 121
            ]
        )
    }

    #[test]
    fn test_rtx() {
        assert_eq!(
            server_packet::rtx("Peace"),
            vec![105, 0, 0, 7, 0, 0, 0, 11, 5, 80, 101, 97, 99, 101]
        )
    }

    #[test]
    fn test_login() {
        let resp = PacketBuilder::new();
        let resp = resp
            .add(server_packet::login_reply(LoginSuccess::Verified(1009)))
            .add(server_packet::protocol_version(19))
            .add(server_packet::notification("Welcome to Peace!"))
            .add(server_packet::main_menu_icon(
                "https://i.kafuu.pro/welcome.png",
                "https://www.baidu.com",
            ))
            .add(server_packet::silence_end(0))
            .add(server_packet::channel_info_end());
        assert_eq!(
            resp.write_out(),
            vec![
                5, 0, 0, 4, 0, 0, 0, 241, 3, 0, 0, 75, 0, 0, 4, 0, 0, 0, 19, 0, 0, 0, 24, 0, 0, 19,
                0, 0, 0, 11, 17, 87, 101, 108, 99, 111, 109, 101, 32, 116, 111, 32, 80, 101, 97,
                99, 101, 33, 76, 0, 0, 55, 0, 0, 0, 11, 53, 104, 116, 116, 112, 115, 58, 47, 47,
                105, 46, 107, 97, 102, 117, 117, 46, 112, 114, 111, 47, 119, 101, 108, 99, 111,
                109, 101, 46, 112, 110, 103, 124, 104, 116, 116, 112, 115, 58, 47, 47, 119, 119,
                119, 46, 98, 97, 105, 100, 117, 46, 99, 111, 109, 92, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0,
                89, 0, 0, 0, 0, 0, 0
            ]
        )
    }

    #[test]
    fn test_write_i32_list() {
        //let list = utils::write_int_list(&vec![1001, 1002, 1003]);
        assert_eq!(
            server_packet::user_presence_bundle(&vec![1001, 1002, 1003]),
            vec![96, 0, 0, 14, 0, 0, 0, 3, 0, 233, 3, 0, 0, 234, 3, 0, 0, 235, 3, 0, 0]
        )
    }

    #[test]
    fn test_write_u32_i32() {
        let int_u32 = utils::osu_write(536870912 as u32);
        let int_i32 = utils::osu_write(536870912);

        println!("{:?} {:?}", int_u32, int_i32);
    }

    #[test]
    fn test_user_presence() {
        let data = server_packet::user_presence(5, "PurePeace", 8, 48, 1, 1.0, 1.0, 666);
        println!("{}", data.len());
        assert_eq!(
            data,
            [
                83, 0, 0, 30, 0, 0, 0, 5, 0, 0, 0, 11, 9, 80, 117, 114, 101, 80, 101, 97, 99, 101,
                32, 48, 1, 0, 0, 128, 63, 0, 0, 128, 63, 154, 2, 0, 0
            ]
        )
    }

    #[test]
    fn test_user_stats() {
        let data = server_packet::user_stats(
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
                11, 0, 0, 84, 0, 0, 0, 5, 0, 0, 0, 1, 11, 4, 105, 100, 108, 101, 11, 32, 97, 115,
                100, 113, 119, 101, 122, 120, 99, 97, 115, 100, 113, 119, 101, 122, 120, 99, 97,
                115, 100, 113, 119, 101, 122, 120, 99, 97, 115, 100, 113, 119, 0, 0, 0, 0, 0, 1, 0,
                0, 0, 128, 150, 152, 0, 0, 0, 0, 0, 40, 131, 35, 60, 16, 39, 0, 0, 0, 225, 245, 5,
                0, 0, 0, 0, 100, 0, 0, 0, 16, 39
            ]
        )
    }

    #[test]
    fn test_client_packets() {
        let (t1, t2, t3, t4, t5, t6) = (1, "test", "test", 2, 3, 4);
        let data = client_packet::user_change_action(t1, t2, t3, t4, t5, t6);

        let mut reader = PacketReader::new(&data);
        let packet = reader.next().unwrap();
        let payload = packet.payload.unwrap();

        let mut reader = PayloadReader::new(payload);

        let r1 = reader.read::<u8>().unwrap();
        let r2 = reader.read::<String>().unwrap();
        let r3 = reader.read::<String>().unwrap();
        let r4 = reader.read::<u32>().unwrap();
        let r5 = reader.read::<u8>().unwrap();
        let r6 = reader.read::<i32>().unwrap();

        assert_eq!(
            (t1, t2, t3, t4, t5, t6),
            (r1, r2.as_str(), r3.as_str(), r4, r5, r6)
        )
    }
}

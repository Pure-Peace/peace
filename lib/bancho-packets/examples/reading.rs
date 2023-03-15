use bancho_packets::{PacketReader, PayloadReader};

fn main() {
    // Data packets from osu!
    let data = &[
        4, 0, 0, 0, 0, 0, 0, 24, 0, 0, 19, 0, 0, 0, 11, 17, 72, 101, 108, 108,
        111, 44, 32, 87, 111, 114, 108, 100, 33, 240, 159, 146, 150, 4, 0, 0,
        0, 0, 0, 0, 24, 0, 0, 18, 0, 0, 0, 11, 16, 229, 147, 136, 229, 147,
        136, 227, 128, 144, 240, 159, 152, 131, 227, 128, 145, 104, 0, 0, 0, 0,
        0, 0, 24, 0, 0, 23, 0, 0, 0, 11, 21, 232, 175, 187, 229, 143, 150, 229,
        174, 140, 228, 186, 134, 239, 188, 129, 239, 188, 129, 226, 156, 168,
    ];

    // Create reader
    let reader = PacketReader::new(data);

    // Read packets
    for packet in reader {
        print!("{:?}: ", packet.id);
        match packet.payload {
            Some(payload) => {
                // Read payload
                let mut payload_reader = PayloadReader::new(payload);
                println!("{:?}", payload_reader.read::<String>());
            },
            None => println!("Non-payload"),
        }
    }
}

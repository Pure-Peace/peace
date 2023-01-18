# bancho-packets-derive

**Available derives**

- `ReadPacket`: This derive macro will implement the `BanchoPacketRead` trait for the struct.
- `WritePacket`: This derive macro will implement the `BanchoPacketRead` trait for the struct.
- `PacketLength`: This derive macro will implement the `BanchoPacketLength` trait for the struct.

example

```rust
use bancho_packets::{ReadPacket, PacketReader, PayloadReader};

#[derive(Debug, Clone, ReadPacket)]
/// [`BanchoMessage`] is the message structure of the bancho client.
pub struct BanchoMessage {
    pub sender: String,
    pub content: String,
    pub target: String,
    pub sender_id: i32,
}

// Now we can use [`PayloadReader`] to read the [`BanchoMessage`] from bytes.
let mut reader = PacketReader::new(&[
    1, 0, 0, 20, 0, 0, 0, 11, 0, 11, 6, 228, 189, 160, 229, 165, 189,
    11, 4, 35, 111, 115, 117, 0, 0, 0, 0,
]);
let packet = reader.next().unwrap();

let mut payload_reader = PayloadReader::new(packet.payload.unwrap());
let message = payload_reader.read::<BanchoMessage>();

println!("{:?}: {:?}", packet.id, message);
```

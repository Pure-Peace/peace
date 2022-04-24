use peace_protocol::{prelude::*, InstanceFactory};
use peace_protocol::{ChatChannel, Server, SessionStorage};

#[tokio::main]
async fn main() {
    write_space!().insert(
        "",
        Server {
            sessions: SessionStorage {},
            channels: ChatChannel {},
        },
    );

    read_space!().get("");
    get_space!().read().await;
    get_space!().write().await;

    SpaceStore::get();
    SpaceStore::write().await;
    SpaceStore::read().await;

    SpaceStore::get1();
    SpaceStore::write1().await;
    SpaceStore::read1().await;
}

use peace_protocol::{server::Server, space::prelude::*};

#[tokio::main]
async fn main() {
    write_space!().regist(Server::new("peace")).unwrap();
}

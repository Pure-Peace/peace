use peace_protocol::{server::Server, space::{prelude::*}};

#[tokio::main]
async fn main() {
    Space::write().regist(Server::new("peace")).unwrap();
    println!("{}", Space::read().contains_key("peace"));
}

use async_std::sync::RwLock;

#[derive(Debug)]
pub struct Player {
    pub name: String,
    pub money: i32,
    pub age: i32,
}

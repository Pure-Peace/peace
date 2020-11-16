use fast_async_mutex::mutex::Mutex;
use std::sync::Arc;


pub type TestType = Arc<Mutex<i32>>;

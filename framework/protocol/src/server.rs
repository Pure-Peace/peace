pub struct Server {
    name: &'static str,
    pub sessions: Sessions,
    pub channels: Channels,
}

impl Server {
    pub fn new(name: &'static str) -> Self {
        Server {
            name,
            sessions: Sessions {},
            channels: Channels {},
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

pub struct Channels {}

pub struct Sessions {}

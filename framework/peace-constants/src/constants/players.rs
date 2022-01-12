use enum_primitive_derive::Primitive;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
pub enum PresenceFilter {
    // A class to represent the update scope the client wishes to receive
    None = 0,
    All = 1,
    Friends = 2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Primitive)]
#[repr(u8)]
pub enum Action {
    // A class to represent the client's current state
    Idle = 0,
    Afk = 1,
    Playing = 2,
    Editing = 3,
    Modding = 4,
    Multiplayer = 5,
    Watching = 6,
    Unknown = 7,
    Testing = 8,
    Submitting = 9,
    Paused = 10,
    Lobby = 11,
    Multiplaying = 12,
    Direct = 13,
}

impl Action {
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

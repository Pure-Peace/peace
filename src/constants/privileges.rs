#![allow(dead_code)]
pub enum Privileges {
    Normal      = 1 << 0,
    Verified    = 1 << 1, // has logged in to the server in-game.

    // has bypass to low-ceiling anticheat measures (trusted).
    Whitelisted = 1 << 2,

    // donation tiers, receives some extra benefits.
    Supporter   = 1 << 4,
    Premium     = 1 << 5,

    // notable users, receives some extra benefits.
    Alumni      = 1 << 7,

    // staff permissions, able to manage server state.
    Tournament  = 1 << 10, // able to manage match state without host.
    Nominator   = 1 << 11, // able to manage maps ranked status.
    Mod         = 1 << 12, // able to manage users (level 1).
    Admin       = 1 << 13, // able to manage users (level 2).
    Dangerous   = 1 << 14, // able to manage full server state.

    Donator = 1 << 4 | 1 << 5,
    Staff = 1 << 12 | 1 << 13 | 1 << 14
}

impl Privileges {
    pub fn enough(self, privileges: i32) -> bool {
        (privileges & self as i32) > 0
    }

    pub fn not_enough(self, privileges: i32) -> bool {
        (privileges & self as i32) == 0
    }
}

pub enum BanchoPrivileges {
    Player     = 1 << 0,
    Moderator  = 1 << 1,
    Supporter  = 1 << 2,
    Owner      = 1 << 3,
    Developer  = 1 << 4,
    Tournament = 1 << 5, // NOTE: not used in communications with osu! client
}
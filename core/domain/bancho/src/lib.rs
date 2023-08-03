use bitmask_enum::bitmask;
use enum_primitive_derive::Primitive;
use peace_pb::bancho_state::CheckUserTokenRequest;
use peace_unique_id::Ulid;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::EnumString;
use tonic::IntoRequest;

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Hash, Serialize, Deserialize)]
pub enum GameMode {
    #[default]
    Standard            = 0,
    Taiko               = 1,
    Fruits              = 2,
    Mania               = 3,

    StandardRelax       = 4,
    TaikoRelax          = 5,
    FruitsRelax         = 6,
    StandardAutopilot   = 8,

    StandardScoreV2     = 12,
}

impl GameMode {
    #[inline]
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[rustfmt::skip]
#[derive(Default)]
#[bitmask(i32)]
pub enum BanchoPrivileges {
    #[default]
    Normal          = 1 << 0,
    Moderator       = 1 << 1,
    Supporter       = 1 << 2,
    Administrator   = 1 << 3,
    Developer       = 1 << 4,
    Tournament      = 1 << 5,
}

impl serde::Serialize for BanchoPrivileges {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i32(self.bits())
    }
}

impl<'de> serde::Deserialize<'de> for BanchoPrivileges {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        i32::deserialize(deserializer).map(Self::from)
    }
}

#[rustfmt::skip]
#[derive(Default)]
#[bitmask(u32)]
pub enum Mods {
    #[default]
    NoMod         = 0,
    NoFail        = 1 << 0,
    Easy          = 1 << 1,
    TouchScreen   = 1 << 2,
    Hidden        = 1 << 3,
    HardRock      = 1 << 4,
    SuddenDeath   = 1 << 5,
    DoubleTime    = 1 << 6,
    Relax         = 1 << 7,
    HalfTime      = 1 << 8,
    NightCore     = 1 << 9,
    FlashLight    = 1 << 10,
    Auto          = 1 << 11,
    SpunOut       = 1 << 12,
    AutoPilot     = 1 << 13,
    Perfect       = 1 << 14,
    Key4          = 1 << 15,
    Key5          = 1 << 16,
    Key6          = 1 << 17,
    Key7          = 1 << 18,
    Key8          = 1 << 19,
    FadeIn        = 1 << 20,
    Random        = 1 << 21,
    Cinema        = 1 << 22,
    Target        = 1 << 23,
    Key9          = 1 << 24,
    KeyCoop       = 1 << 25,
    Key1          = 1 << 26,
    Key3          = 1 << 27,
    Key2          = 1 << 28,
    ScoreV2       = 1 << 29,
    Mirror        = 1 << 30,

    KeyMods = Self::Key1.bits
        | Self::Key2.bits
        | Self::Key3.bits
        | Self::Key4.bits
        | Self::Key5.bits
        | Self::Key6.bits
        | Self::Key7.bits
        | Self::Key8.bits
        | Self::Key9.bits,

    ScoreIncrease = Self::Hidden.bits
        | Self::HardRock.bits
        | Self::FadeIn.bits
        | Self::DoubleTime.bits
        | Self::FlashLight.bits,

    SpeedChanging =
        Self::DoubleTime.bits | Self::NightCore.bits | Self::HalfTime.bits,

    StandardOnly = Self::AutoPilot.bits | Self::SpunOut.bits | Self::Target.bits,
    ManiaOnly = Self::Mirror.bits
        | Self::Random.bits
        | Self::FadeIn.bits
        | Self::KeyMods.bits,
}

impl serde::Serialize for Mods {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl<'de> serde::Deserialize<'de> for Mods {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u32::deserialize(deserializer).map(Self::from)
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Serialize, Deserialize)]
pub enum UserOnlineStatus {
    #[default]
    Idle          = 0,
    Afk           = 1,
    Playing       = 2,
    Editing       = 3,
    Modding       = 4,
    Multiplayer   = 5,
    Watching      = 6,
    Unknown       = 7,
    Testing       = 8,
    Submitting    = 9,
    Paused        = 10,
    Lobby         = 11,
    Multiplaying  = 12,
    Direct        = 13,
}

impl UserOnlineStatus {
    #[inline]
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

#[rustfmt::skip]
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Primitive, Serialize, Deserialize)]
pub enum PresenceFilter {
    #[default]
    None    = 0,
    All     = 1,
    Friends = 2,
}

impl PresenceFilter {
    #[inline]
    pub fn val(&self) -> i32 {
        *self as i32
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, EnumString, Serialize, Deserialize)]
pub enum BanchoCountryCode {
    OC = 1,  AO = 11, BD = 21, BR = 31, CF = 41, CU = 51, DZ = 61, FK = 71,
    EU = 2,  AQ = 12, BE = 22, BS = 32, CG = 42, CV = 52, EC = 62, FM = 72,
    AD = 3,  AR = 13, BF = 23, BT = 33, CH = 43, CX = 53, EE = 63, FO = 73,
    AE = 4,  AS = 14, BG = 24, BV = 34, CI = 44, CY = 54, EG = 64, FR = 74,
    AF = 5,  AT = 15, BH = 25, BW = 35, CK = 45, CZ = 55, EH = 65, FX = 75,
    AG = 6,  AU = 16, BI = 26, BY = 36, CL = 46, DE = 56, ER = 66, GA = 76,
    AI = 7,  AW = 17, BJ = 27, BZ = 37, CM = 47, DJ = 57, ES = 67, GB = 77,
    AL = 8,  AZ = 18, BM = 28, CA = 38, CN = 48, DK = 58, ET = 68, GD = 78,
    AM = 9,  BA = 19, BN = 29, CC = 39, CO = 49, DM = 59, FI = 69, GE = 79,
    AN = 10, BB = 20, BO = 30, CD = 40, CR = 50, DO = 60, FJ = 70, GF = 80,

    GH = 81, GU = 91,  IE = 101, JP = 111, KY = 121, LU = 131, MM = 141, MW = 151,
    GI = 82, GW = 92,  IL = 102, KE = 112, KZ = 122, LV = 132, MN = 142, MX = 152,
    GL = 83, GY = 93,  IN = 103, KG = 113, LA = 123, LY = 133, MO = 143, MY = 153,
    GM = 84, HK = 94,  IO = 104, KH = 114, LB = 124, MA = 134, MP = 144, MZ = 154,
    GN = 85, HM = 95,  IQ = 105, KI = 115, LC = 125, MC = 135, MQ = 145, NA = 155,
    GP = 86, HN = 96,  IR = 106, KM = 116, LI = 126, MD = 136, MR = 146, NC = 156,
    GQ = 87, HR = 97,  IS = 107, KN = 117, LK = 127, MG = 137, MS = 147, NE = 157,
    GR = 88, HT = 98,  IT = 108, KP = 118, LR = 128, MH = 138, MT = 148, NF = 158,
    GS = 89, HU = 99,  JM = 109, KR = 119, LS = 129, MK = 139, MU = 149, NG = 159,
    GT = 90, ID = 100, JO = 110, KW = 120, LT = 130, ML = 140, MV = 150, NI = 160,

    NL = 161, PG = 171, PY = 181, SE = 191, SR = 201, TJ = 211, TZ = 221, VG = 231,
    NO = 162, PH = 172, QA = 182, SG = 192, ST = 202, TK = 212, UA = 222, VI = 232,
    NP = 163, PK = 173, RE = 183, SH = 193, SV = 203, TM = 213, UG = 223, VN = 233,
    NR = 164, PL = 174, RO = 184, SI = 194, SY = 204, TN = 214, UM = 224, VU = 234,
    NU = 165, PM = 175, RU = 185, SJ = 195, SZ = 205, TO = 215, US = 225, WF = 235,
    NZ = 166, PN = 176, RW = 186, SK = 196, TC = 206, TL = 216, UY = 226, WS = 236,
    OM = 167, PR = 177, SA = 187, SL = 197, TD = 207, TR = 217, UZ = 227, YE = 237,
    PA = 168, PS = 178, SB = 188, SM = 198, TF = 208, TT = 218, VA = 228, YT = 238,
    PE = 169, PT = 179, SC = 189, SN = 199, TG = 209, TV = 219, VC = 229, RS = 239,
    PF = 170, PW = 180, SD = 190, SO = 200, TH = 210, TW = 220, VE = 230, ZA = 240,

    ZM = 241, BL = 251,
    ME = 242, MF = 252,
    ZW = 243,
    XX = 244,
    A2 = 245,
    O1 = 246,
    AX = 247,
    GG = 248,
    IM = 249,
    JE = 250,
}

impl BanchoCountryCode {
    pub fn get_code(s: &str) -> u8 {
        BanchoCountryCode::from_str(s).map(|c| c as u8).unwrap_or_default()
    }
}

#[derive(thiserror::Error, Debug, Serialize, Deserialize)]
pub enum ParseBanchoClientTokenError {
    #[error("Invalid token format")]
    InvalidFormat,
    #[error("Invalid user id")]
    InvalidUserId,
    #[error("Invalid session id")]
    InvalidSessionId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BanchoClientToken {
    pub user_id: i32,
    pub session_id: Ulid,
    pub signature: String,
}

impl BanchoClientToken {
    #[inline]
    pub fn content(&self) -> String {
        format!("{}.{}", self.user_id, self.session_id)
    }

    #[inline]
    pub fn encode_content(user_id: i32, session_id: &str) -> String {
        format!("{user_id}.{session_id}")
    }

    #[inline]
    pub fn encode(user_id: i32, session_id: &str, signature: &str) -> String {
        format!("{user_id}.{session_id}.{signature}")
    }
}

impl FromStr for BanchoClientToken {
    type Err = ParseBanchoClientTokenError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split('.').collect::<Vec<&str>>();
        if split.len() != 3 {
            return Err(ParseBanchoClientTokenError::InvalidFormat);
        }

        let user_id = split[0]
            .parse::<i32>()
            .map_err(|_| ParseBanchoClientTokenError::InvalidUserId)?;

        let session_id = Ulid::from_str(split[1])
            .map_err(|_| ParseBanchoClientTokenError::InvalidSessionId)?;

        let signature = split[2].to_string();

        Ok(Self { user_id, session_id, signature })
    }
}

impl IntoRequest<CheckUserTokenRequest> for BanchoClientToken {
    fn into_request(self) -> tonic::Request<CheckUserTokenRequest> {
        tonic::Request::new(CheckUserTokenRequest {
            user_id: self.user_id,
            session_id: self.session_id.to_string(),
            signature: self.signature,
        })
    }
}

impl std::fmt::Display for BanchoClientToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.user_id, self.session_id, self.signature)
    }
}

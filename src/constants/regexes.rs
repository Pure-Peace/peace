use regex::Regex;

lazy_static::lazy_static! {
    pub static ref USERNAME_REGEX: Regex = Regex::new(r"^[0-9a-zA-Z_ \[\]-]{2,16}$").unwrap();
    pub static ref EMAIL_REGEX: Regex = Regex::new(r"^[^@\s]{1,200}@[^@\s\.]{1,30}\.[^@\.\s]{2,24}$").unwrap();
    pub static ref OSU_VERSION_REGEX: Regex = Regex::new(r"(?x)(?P<year>\d{4})(?P<month>\d{2})(?P<day>\d{2})").unwrap();
    pub static ref OSU_FILENAME_FROM_QUERY: Regex = Regex::new(r"(\b&f=)(.*\.osu?)(\b&)").unwrap();
    pub static ref OSU_FILENAME: Regex = Regex::new(r"^(?P<artist>.+) - (?P<title>.+) (?:\((?P<creator>.+)\))?(?: \[(?P<version>.+)\])?\.osu$").unwrap();
}

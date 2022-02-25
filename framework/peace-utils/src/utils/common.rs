use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct ContentDisposition<'a> {
    pub name: Option<&'a str>,
    pub filename: Option<&'a str>,
}

impl<'a> ContentDisposition<'a> {
    #[inline]
    pub fn parse(s: &'a str) -> Option<Self> {
        let s_last = s.strip_prefix("form-data;")?;
        let mut name = None;
        let mut filename = None;
        for i in s_last.split(";") {
            if let Some((k, v)) = i.trim().split_once("=") {
                if name.is_some() && filename.is_some() {
                    break;
                }
                if k == "name" {
                    name = Some(v.trim_matches('"'));
                } else if k == "filename" {
                    filename = Some(v.trim_matches('"'));
                }
            }
        }
        if name.is_none() && filename.is_none() {
            return None;
        }
        Some(Self { name, filename })
    }

    #[inline]
    pub fn get_name(s: &'a str) -> Option<&'a str> {
        Self::get_key(s, "name")
    }

    #[inline]
    pub fn get_file_name(s: &'a str) -> Option<&'a str> {
        Self::get_key(s, "filename")
    }

    #[inline]
    pub fn get_key(s: &'a str, key: &str) -> Option<&'a str> {
        let s_last = s.strip_prefix("form-data;")?;
        for i in s_last.split(";") {
            if let Some((k, v)) = i.trim().split_once("=") {
                if k == key {
                    return Some(v.trim_matches('"'));
                }
            }
        }
        None
    }
}

#[inline]
pub fn get_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

#[inline]
pub fn build_s(len: usize) -> String {
    let mut s = String::new();
    for i in 1..len + 1 {
        s += (if i == len {
            format!("${}", i)
        } else {
            format!("${},", i)
        })
        .as_str();
    }
    s
}

#[inline]
pub fn safe_file_name(mut s: String) -> String {
    for i in r#":\*></?"|"#.chars() {
        s = s.replace(i, "");
    }
    s
}

#[inline]
pub fn safe_string(mut s: String) -> String {
    for i in r#":\*></?"|.,()[]{}!@#$%^&-_=+~`"#.chars() {
        s = s.replace(i, "");
    }
    s
}

#[inline]
pub fn progress_bar(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/white} ]{pos:>7}/{len:7} ({eta})").progress_chars("#>-"));
    bar
}

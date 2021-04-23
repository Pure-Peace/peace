use indicatif::{ProgressBar, ProgressStyle};

#[inline(always)]
pub fn get_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

#[inline(always)]
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

#[inline(always)]
pub fn safe_file_name(mut s: String) -> String {
    for i in r#":\*></?"|"#.chars() {
        s = s.replace(i, "");
    }
    s
}

#[inline(always)]
pub fn safe_string(mut s: String) -> String {
    for i in r#":\*></?"|.,()[]{}!@#$%^&-_=+~`"#.chars() {
        s = s.replace(i, "");
    }
    s
}

#[inline(always)]
pub fn progress_bar(total: u64) -> ProgressBar {
    let bar = ProgressBar::new(total);
    bar.set_style(ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/white} ]{pos:>7}/{len:7} ({eta})").progress_chars("#>-"));
    bar
}

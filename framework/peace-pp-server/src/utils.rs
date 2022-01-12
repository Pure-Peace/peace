use ntex::web::types::Data;
use tokio::fs::File as AsyncFile;
use colored::Colorize;
use peace_performance::Beatmap as PPbeatmap;
use std::cmp::min;
use std::time::Instant;
use std::{fs, io};

use crate::objects::{Caches, PPbeatmapCache};

#[inline(always)]
pub fn check_is_osu_file(entry: &Result<fs::DirEntry, io::Error>) -> u8 {
    if entry.is_err() {
        return 3;
    };
    let entry = entry.as_ref().unwrap();
    if entry.path().is_dir() {
        return 2;
    };
    let file_name = match entry.file_name().into_string() {
        Ok(n) => n,
        Err(_) => {
            return 3;
        }
    };
    if !file_name.ends_with(".osu") {
        return 3;
    };
    1
}

#[inline(always)]
pub fn listing_osu_files(osu_files_dir: &String) -> (Vec<Option<fs::DirEntry>>, usize) {
    println!(
        "{}",
        format!("\n> Listing .osu dir '{}' now...", osu_files_dir).bright_yellow()
    );
    let entries: Vec<Option<fs::DirEntry>> = fs::read_dir(osu_files_dir.clone())
        .unwrap()
        .map(|r| match check_is_osu_file(&r) {
            1 => Some(r.unwrap()),
            _ => None,
        })
        .filter(|r| r.is_some())
        .collect();
    let total = entries.len();
    println!(
        "\n{}",
        format!("> Done, .osu file count: {}", total).bright_yellow()
    );
    (entries, total)
}

#[inline(always)]
pub async fn preload_osu_files(osu_files_dir: &String, max_load: i32, caches: &Data<Caches>) {
    let (entries, total) = listing_osu_files(osu_files_dir);
    if total > 9000 && max_load > 9000 {
        println!("{}", "WARNING: Your will preload > 9000 beatmaps, loading them into memory may cause insufficient memory or even system crashes.".red())
    };
    println!("\n  Preloading .osu files into Memory...");
    let bar = peace_utils::common::progress_bar(min(max_load, total as i32) as u64);
    let mut success = 0;
    let start = Instant::now();
    let mut pp_beatmap_cache = caches.pp_beatmap_cache.write().await;
    for entry in entries {
        bar.inc(1);
        if let Some(entry) = entry {
            {
                if let Ok(file_name) = entry.file_name().into_string() {
                    let md5 = file_name.replace(".osu", "");
                    {
                        if let Ok(file) = AsyncFile::open(entry.path()).await {
                            match PPbeatmap::parse(file).await {
                                Ok(b) => {
                                    pp_beatmap_cache.insert(md5.to_string(), PPbeatmapCache::new(b))
                                }
                                Err(_e) => continue,
                            };
                        };
                    }
                }
            }
            success += 1;
            if success >= max_load {
                break;
            }
        }
    }
    bar.finish();
    println!(
        "\n{}\n",
        format!(
            "> Beatmaps has preloaded, \n> Success: {}, Total: {}, MaxLoad: {}; \n> time spent: {:?}",
            success,
            total,
            max_load,
            start.elapsed()
        )
        .bright_yellow()
    )
}

#[inline(always)]
pub fn recalculate_osu_file_md5(osu_files_dir: &String) {
    let mut renamed = 0;
    let mut done = 0;
    let mut error = 0;
    let (entries, total) = listing_osu_files(osu_files_dir);
    println!("\n  Recalculating MD5 file names...");
    let bar = peace_utils::common::progress_bar(total as u64);
    let start = Instant::now();
    for entry in entries {
        bar.inc(1);
        if let Some(entry) = entry {
            let md5 = match peace_utils::async_file::calc_file_md5(entry.path()) {
                Ok(md5) => md5,
                Err(_) => {
                    error += 1;
                    continue;
                }
            };
            if fs::rename(entry.path(), format!("{}/{}.osu", osu_files_dir, md5)).is_err() {
                error += 1;
            } else {
                renamed += 1;
            }
            done += 1;
        }
    }
    bar.finish();
    println!(
        "{}\n",
        format!(
            "> Done, \n> Success / Done / Total: {} / {} / {}; \n> Errors: {}; \n> time spent: {:?}",
            renamed,
            done,
            total,
            error,
            start.elapsed()
        )
        .bright_yellow()
    )
}

#[inline(always)]
pub fn checking_osu_dir(osu_files_dir: &String, recalculate_md5: bool) {
    if osu_files_dir == "" {
        println!(
            "{}",
            "> [Error] Please add .osu files dir in pp-server-config!!!\n"
                .bold()
                .red()
        );
    } else if recalculate_md5 {
        recalculate_osu_file_md5(osu_files_dir);
    };
}

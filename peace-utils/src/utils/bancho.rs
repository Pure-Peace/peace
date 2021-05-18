use peace_database::Database;

/// Get beatmap ratings from database
#[inline(always)]
pub async fn get_beatmap_rating(beatmap_md5: &String, database: &Database) -> Option<f32> {
    match database
        .pg
        .query_first(
            r#"SELECT AVG("rating")::float4 FROM "beatmaps"."ratings" WHERE "map_md5" = $1"#,
            &[beatmap_md5],
        )
        .await
    {
        Ok(value) => Some(value.get(0)),
        Err(err) => {
            error!(
                "failed to get avg rating from beatmap {}, err: {:?}",
                beatmap_md5, err
            );
            None
        }
    }
}

#[inline(always)]
pub async fn save_replay(
    score_file: bytes::Bytes,
    score_id: i64,
    data_dir: &str,
    mode: &peace_constants::GameMode,
) -> std::io::Result<()> {
    let dir = format!(
        "{}/replays/{}/{}",
        data_dir,
        mode.mode_name(),
        mode.sub_mod()
    );
    let file_path = format!("{}/{}.osr", dir, score_id);
    let _ = std::fs::create_dir_all(dir);
    tokio::fs::write(file_path, score_file).await
}

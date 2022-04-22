use crate::objects::Bancho;

use {
    maxminddb::Reader,
    memmap::Mmap,
    ntex::server::Server,
    ntex::web::{
        get,
        types::{Data, Path},
        HttpResponse,
    },
    peace_database::Database,
    std::time::Instant,
    tokio::sync::mpsc::UnboundedSender,
};

/// GET "/test_pg"
#[get("/test_pg")]
pub async fn test_pg(database: Data<Database>) -> HttpResponse {
    let start = Instant::now();
    let contents = database
        .pg
        .query_first_simple(r#"SELECT 'PurePeace' as "name";"#)
        .await
        .unwrap();
    let end = start.elapsed();
    let name: String = contents.get("name");
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(format!("{}\n{:.2?}", name, end))
}

/// GET "/test_redis"
#[get("/test_redis")]
pub async fn test_redis(database: Data<Database>) -> HttpResponse {
    let start = Instant::now();
    let _ = database.redis.set("test", &["PurePeace", "NX"]).await;
    let contents: String = database.redis.get("test").await.unwrap();
    let end = start.elapsed();
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(format!("{}\n{:.2?}", contents, end))
}

/// GET "/test_async_lock"
#[get("/test_async_lock")]
pub async fn test_async_lock(bancho: Data<Bancho>) -> HttpResponse {
    let start = Instant::now();
    let player_sessions = read_lock!(bancho.player_sessions);
    let _map = &player_sessions.token_map;
    let end = start.elapsed();
    HttpResponse::Ok()
        .set_header("Content-Type", "text/html; charset=UTF-8")
        .body(&format!("{:.2?}", end))
}

/// GET "/test_player_read"
#[get("/test_player_read/{token}")]
pub async fn test_player_read(token: Path<String>, bancho: Data<Bancho>) -> HttpResponse {
    let start = Instant::now();
    let player_info = match read_lock!(bancho.player_sessions)
        .get_player_data(&token.into_inner())
        .await
    {
        Some(player_data) => format!("{:?}", player_data),
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}

/// GET "/test_player_money_add"
#[get("/test_player_money_add/{token}")]
pub async fn test_player_money_add(token: Path<String>, bancho: Data<Bancho>) -> HttpResponse {
    let start = Instant::now();
    let player_sessions = read_lock!(bancho.player_sessions);
    let player_info = match player_sessions.token_map.get(&token.into_inner()) {
        Some(player) => {
            // (*player).money += 1;
            //tokio::task::sleep(std::time::Duration::from_secs(1)).await;
            format!("{:?}", *player)
        }
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{} {:.2?}", player_info, end))
}

/// GET "/test_player_money_reduce"
#[get("/test_player_money_reduce/{token}")]
pub async fn test_player_money_reduce(token: Path<String>, bancho: Data<Bancho>) -> HttpResponse {
    let start = Instant::now();
    let player_sessions = read_lock!(bancho.player_sessions);
    let player_info = match player_sessions.token_map.get(&token.into_inner()) {
        Some(player) => {
            // (*player).money -= 1;
            format!("{:?}", *player)
        }
        None => "non this player".to_string(),
    };
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{:?} {:.2?}", player_info, end))
}

/// GET "/test_player_money_reduce_special"
#[get("/test_player_money_reduce_special/{token}")]
pub async fn test_player_money_reduce_special(
    token: Path<String>,
    bancho: Data<Bancho>,
) -> HttpResponse {
    let start = Instant::now();
    let player_info = read_lock!(bancho.player_sessions)
        .handle_player_get(
            &token.into_inner(),
            |_player| Some(()), /* (*player).money -= 1 */
        )
        .await;
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("{:?} {:.2?}", player_info, end))
}

/// GET "/player_sessions_all"
#[get("/player_sessions_all")]
pub async fn player_sessions_all(bancho: Data<Bancho>) -> HttpResponse {
    HttpResponse::Ok().body(read_lock!(bancho.player_sessions).map_to_string().await)
}

/// GET "/player_maps_info"
#[get("/player_maps_info")]
pub async fn player_maps_info(bancho: Data<Bancho>) -> HttpResponse {
    let start = Instant::now();
    let maps = read_lock!(bancho.player_sessions);
    HttpResponse::Ok().body(format!(
        "token_map: {}, id_map: {}, name_map: {}; time: {:.2?}",
        &maps.token_map.len(),
        &maps.id_map.len(),
        &maps.name_map.len(),
        start.elapsed()
    ))
}

/// GET "/player_channels_all"
#[get("/player_channels_all")]
pub async fn player_channels_all(bancho: Data<Bancho>) -> HttpResponse {
    HttpResponse::Ok().body(format!("{:?}", read_lock!(bancho.channel_list).values()))
}

/// GET "/player_sessions_kick"
#[get("/player_sessions_kick/{token}")]
pub async fn player_sessions_kick(token: Path<String>, bancho: Data<Bancho>) -> HttpResponse {
    let token = token.into_inner();
    HttpResponse::Ok().body(
        match write_lock!(bancho.player_sessions)
            .logout(&token, Some(&bancho.channel_list))
            .await
        {
            Some(player) => format!("{}\n{:?}", token, player),
            None => "non this player".to_string(),
        },
    )
}

/// GET "/player_sessions_kick_uid"
#[get("/player_sessions_kick_uid/{user_id}")]
pub async fn player_sessions_kick_uid(user_id: Path<i32>, bancho: Data<Bancho>) -> HttpResponse {
    HttpResponse::Ok().body(
        match write_lock!(bancho.player_sessions)
            .logout_with_id(user_id.into_inner(), Some(&bancho.channel_list))
            .await
        {
            Some(player) => format!("{:?}", player),
            None => "non this player".to_string(),
        },
    )
}

/// GET "/test_geo_ip"
#[get("/test_geo_ip/{ip_address}")]
pub async fn test_geo_ip(
    ip_address: Path<String>,
    geo_db: Data<Option<Reader<Mmap>>>,
) -> HttpResponse {
    match peace_utils::geoip::geo_ip_info(&ip_address.to_string(), &geo_db.get_ref()).await {
        Ok(json_success) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(json_success),
        Err(json_error) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .body(json_error),
    }
}

/// GET "/bancho_config_get"
#[get("/bancho_config_get")]
pub async fn bancho_config_get(bancho: Data<Bancho>) -> HttpResponse {
    HttpResponse::Ok().body(format!("{:?}", read_lock!(bancho.config)))
}

/// GET "/bancho_config_update"
#[get("/bancho_config_update")]
pub async fn bancho_config_update(database: Data<Database>, bancho: Data<Bancho>) -> HttpResponse {
    let result = write_lock!(bancho.config).update(&database).await;
    HttpResponse::Ok().body(format!("{}", result))
}

/// GET "/osu_api_test"
#[get("/osu_api_test")]
pub async fn osu_api_test(bancho: Data<Bancho>) -> HttpResponse {
    let results = write_lock!(bancho.osu_api).test_all().await;
    HttpResponse::Ok()
        .content_type("application/json")
        .body(results)
}

/// GET "/osu_api_all"
#[get("/osu_api_all")]
pub async fn osu_api_all(bancho: Data<Bancho>) -> HttpResponse {
    HttpResponse::Ok().body(format!("{:?}", read_lock!(bancho.osu_api)))
}

/// GET "/osu_api_reload"
#[get("/osu_api_reload")]
pub async fn osu_api_reload(bancho: Data<Bancho>) -> HttpResponse {
    let start = Instant::now();
    let api_keys = read_lock!(bancho.config).data.server.osu_api_keys.clone();
    write_lock!(bancho.osu_api).reload_clients(api_keys).await;
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("done in: {:?}", end))
}

/// GET "/server_stop"
#[get("/server_stop")]
pub async fn server_stop(sender: Data<UnboundedSender<Option<Server>>>) -> HttpResponse {
    let start = Instant::now();
    let _ = sender.send(None);
    let end = start.elapsed();
    HttpResponse::Ok().body(format!("done in: {:?}", end))
}

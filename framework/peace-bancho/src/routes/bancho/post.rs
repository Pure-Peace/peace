use super::depends::*;
use crate::handlers;

pub async fn handler(
    req: HttpRequest,
    body: Bytes,
    bancho: Data<Bancho>,
    database: Data<Database>,
    caches: Data<Caches>,
    counter: Data<IntCounterVec>,
    geo_db: Data<Option<Reader<Mmap>>>,
) -> HttpResponse {
    // Prom counter
    counter
        .with_label_values(&["/bancho", "post", "start"])
        .inc();

    // Get real request ip
    let request_ip = match peace_utils::web::get_realip(&req).await {
        Ok(ip) => ip,
        Err(_) => {
            return HttpResponse::BadRequest().body("bad requests");
        }
    };

    // Blocked ip
    if read_lock!(bancho.config)
        .data
        .server
        .ip_blacklist
        .contains(&request_ip)
    {
        return HttpResponse::Ok().body("bad requests");
    };

    let mut resp = PacketBuilder::new();
    let bancho_start = std::time::Instant::now();
    let headers = req.headers();

    // Get osu ver
    let osu_version = peace_utils::web::get_osuver(&req).await;

    // If not login
    if !headers.contains_key("osu-token") {
        return super::login::handler(
            resp,
            req,
            body,
            request_ip,
            osu_version,
            bancho,
            database,
            caches,
            counter,
            geo_db,
        )
        .await;
    }

    // Get token from headers
    let token = match headers.get("osu-token").unwrap().to_str() {
        Ok(token) => token.to_string(),
        Err(err) => {
            error!("Failed to get osu-token, error: {:?}", err);
            return HttpResponse::Ok().body(
                resp.add(bancho_packets::login_reply(LoginFailed::ServerError))
                    .write_out(),
            );
        }
    };

    // Get player
    let player_sessions_r = read_lock!(bancho.player_sessions);
    let (player_data, weak_player) = match player_sessions_r.token_map.get(&token) {
        Some(player) => (
            PlayerData::from(&*read_lock!(player)),
            Arc::downgrade(player),
        ),
        None => {
            return HttpResponse::Ok()
                .content_type("text/html; charset=UTF-8")
                .body(
                    resp.add(bancho_packets::notification("Welcome back!"))
                        .add(bancho_packets::bancho_restart(0))
                        .write_out(),
                );
        }
    };
    // Drop the lock first
    drop(player_sessions_r);

    // Read & handle client packets
    let mut reader = PacketReader::new(&body);
    while let Some(packet) = reader.next() {
        // osu_ping need not handle
        if packet.id == PacketId::OSU_PING {
            continue;
        };

        handlers::bancho::packets::read_handle(
            &packet.id,
            &request_ip,
            &token,
            &player_data,
            &weak_player,
            &bancho,
            &database,
            packet.payload,
        )
        .await;
    }

    // Push player's packets to the response
    if let Some(player) = weak_player.upgrade() {
        let mut player = write_lock!(player);
        // Update player's active time
        player.update_active();

        // Update player's ip data (includes geo ip data)
        if request_ip != player.ip {
            player.update_ip(request_ip, geo_db.as_ref());
        }

        // Dequeue player's packet into resp
        while let Some(packet_data) = player.dequeue().await {
            resp.add_ref(packet_data);
        }
    }

    let bancho_end = bancho_start.elapsed();
    debug!(
        "bancho handle end, time spend: {:?}; payload size: {}",
        bancho_end, reader.payload_len()
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(resp.write_out())
}

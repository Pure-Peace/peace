use crate::{
    packets::{self, PacketReader},
    utils,
};

use super::depends::*;

pub async fn handler(
    req: HttpRequest,
    body: Bytes,
    player_sessions: Data<RwLock<PlayerSessions>>,
    database: Data<Database>,
    channel_list: Data<RwLock<ChannelList>>,
    counter: Data<IntCounterVec>,
) -> HttpResponse {
    // Prom counter
    counter
        .with_label_values(&["/bancho", "post", "start"])
        .inc();

    let bancho_start = std::time::Instant::now();

    let mut resp = packets::PacketBuilder::new();

    // Get headers
    let headers = req.headers();

    // Get real request ip
    let request_ip = utils::get_realip(&req)
        .await
        .expect("Cannot get request ip");

    // Get osu ver
    let osu_version = utils::get_osuver(&req).await;

    // If not login
    if !headers.contains_key("osu-token") {
        return super::login::handler(
            resp,
            req,
            body,
            request_ip,
            osu_version,
            database,
            player_sessions,
            channel_list,
            counter,
        )
        .await;
    }

    // Get token from headers
    let token = match headers.get("osu-token").unwrap().to_str() {
        Ok(token) => token.to_string(),
        Err(err) => {
            error!("Failed to get osu-token, error: {:?}", err);
            return HttpResponse::Ok().body(
                resp.add(packets::login_reply(LoginFailed::ServerError))
                    .write_out(),
            );
        }
    };

    // Get player
    let player_sessions_r = player_sessions.read().await;
    let player_data = match player_sessions_r.map.write().await.get_mut(&token) {
        Some(player) => {
            // Update player's active time
            player.update_active();
            PlayerData::from(player)
        }
        None => {
            return HttpResponse::Ok()
                .content_type("text/html; charset=UTF-8")
                .body(
                    resp.add(packets::login_reply(LoginFailed::ServerError))
                        .add(packets::notification("Please login again!"))
                        .write_out(),
                )
        }
    };
    // Drop the lock first
    drop(player_sessions_r);

    // Read & handle client packets
    let mut reader = PacketReader::from_bytes(body);
    while let Some((packet_id, payload)) = reader.next().await {
        packet_id
            .handle(
                &request_ip,
                &token,
                &player_data,
                &player_sessions,
                &database,
                &channel_list,
                payload,
            )
            .await;
    }

    // Push player's packets to the response
    let player_sessions_r = player_sessions.read().await;
    match player_sessions_r.map.write().await.get_mut(&token) {
        Some(player) => {
            // Dequeue player's packet into resp
            while let Some(packet_data) = player.dequeue().await {
                resp.add_ref(packet_data);
            }
            // Update player's active time
            player.update_active();
        }
        // Player has been logout
        None => {}
    };
    drop(player_sessions_r);

    let bancho_end = bancho_start.elapsed();
    debug!("bancho handle end, time spend: {:?}", bancho_end);

    HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(resp.write_out())
}

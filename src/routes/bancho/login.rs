use super::depends::*;
use crate::packets;

const DEFAULT_TOKEN: &str = "login_failed";

#[inline(always)]
pub async fn handler(
    resp: PacketBuilder,
    req: HttpRequest,
    body: Bytes,
    request_ip: String,
    osu_version: String,
    database: Data<Database>,
    player_sessions: Data<RwLock<PlayerSessions>>,
    channel_list: Data<RwLock<ChannelList>>,
    bancho_config: Data<RwLock<BanchoConfig>>,
    argon2_cache: Data<RwLock<Argon2Cache>>,
    counter: Data<IntCounterVec>,
    geo_db: Data<Option<Reader<Mmap>>>,
) -> HttpResponse {
    let bancho_config = bancho_config.read().await.clone();

    // Login is currently disabled
    if !bancho_config.login_enabled {
        return HttpResponse::Ok()
            .set_header("cho-token", "login_refused")
            .set_header("cho-protocol", "19")
            .body(
                resp.add(packets::notification("The server currently does not allow login, please wait or contact the administrator."))
                    .write_out(),
            );
    }

    // Blocked ip
    if bancho_config.login_disallowed_ip.contains(&request_ip) {
        return HttpResponse::Ok()
            .set_header("cho-token", "login_refused")
            .set_header("cho-protocol", "19")
            .body(
                resp.add(packets::login_reply(LoginFailed::InvalidCredentials))
                    .add(packets::notification("You are not allowed to login!"))
                    .write_out(),
            );
    }

    // Online user limit check
    if bancho_config.online_users_limit {
        // Get online players
        let online_users = player_sessions
            .read()
            .await
            .player_count
            .load(Ordering::SeqCst);

        if online_users >= bancho_config.online_users_max {
            return HttpResponse::Ok()
                .set_header("cho-token", "login_refused")
                .set_header("cho-protocol", "19")
                .body(
                    resp.add(packets::notification(&format!(
                        "The current server allows up to {} people to be online, please wait...",
                        online_users
                    )))
                    .write_out(),
                );
        };
    };

    let expire_secs = bancho_config.login_retry_expire_seconds;
    let max_failed_count = bancho_config.login_retry_max_count;

    let failed_key = format!("{}-bancho_login_failed", &request_ip);
    let failed_count = database.redis.get(&failed_key).await.unwrap_or(0);
    // Too many failed in 300s, refuse login
    if failed_count > max_failed_count {
        warn!(
            "ip: {} login refused, beacuse too many login failed_count({}) in {}s;",
            request_ip, failed_count, expire_secs
        );
        return HttpResponse::Ok()
            .set_header("cho-token", "login_refused")
            .set_header("cho-protocol", "19")
            .body(
                resp.add(packets::login_reply(LoginFailed::ServerError))
                    .add(packets::notification(
                        "You are not allowed to login now, please wait!",
                    ))
                    .write_out(),
            );
    }

    // Login handle
    let (resp_body, token) = match bancho::login(
        req,
        &body,
        &request_ip,
        osu_version,
        &database,
        &player_sessions,
        &channel_list,
        &bancho_config,
        &argon2_cache,
        &counter,
        &geo_db,
    )
    .await
    {
        Ok((packet_data, token)) => (packet_data, token),
        Err((error_str, packet_builder)) => {
            // Notification string
            let mut failed_notification = String::new();

            // Default failed login reply is InvalidCredentials
            let packet_builder = packet_builder
                .unwrap_or(resp.add(packets::login_reply(LoginFailed::InvalidCredentials)));

            // Record login failed
            counter
                .with_label_values(&["/bancho", "post", &format!("login.failed.{}", error_str)])
                .inc();
            // Increase failed count for this ip
            let failed_count: i32 = database.redis.query("INCR", &[&failed_key]).await;
            let _ = database.redis.expire(&failed_key, expire_secs).await;

            // Add notification string
            failed_notification.push_str(&format!("Login failed {} times!!\n", failed_count));

            // If reached the retires limit
            if failed_count > max_failed_count {
                failed_notification.push_str(&format!("Your login retries have reached the upper limit! \nPlease try again in {} seconds.", expire_secs));
                warn!(
                    "ip: {} login failed, count: {}, will temporarily restrict their login",
                    &request_ip, failed_count
                );
            } else {
                failed_notification.push_str(&format!(
                    "You can try {} more times.\n",
                    max_failed_count - failed_count + 1
                ))
            };

            // Returns
            (
                packet_builder
                    .add(packets::notification(&failed_notification))
                    .write_out(),
                DEFAULT_TOKEN.to_string(),
            )
        }
    };

    return HttpResponse::Ok()
        .set_header("cho-token", token)
        .set_header("cho-protocol", "19")
        .body(resp_body);
}

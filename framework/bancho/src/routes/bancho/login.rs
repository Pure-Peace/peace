use super::depends::*;

const DEFAULT_TOKEN: &str = "login_failed";

#[inline]
pub async fn handler(
    resp: PacketBuilder,
    req: HttpRequest,
    body: Bytes,
    request_ip: String,
    osu_version: String,
    bancho: Data<Bancho>,
    database: Data<Database>,
    caches: Data<Caches>,
    counter: Data<IntCounterVec>,
    geo_db: Data<Option<Reader<Mmap>>>,
) -> HttpResponse {
    let cfg_r = read_lock!(bancho.config);
    let cfg = &cfg_r.data;

    // Login is currently disabled
    if !cfg.login.enabled {
        return HttpResponse::Ok()
            .set_header("cho-token", "login_refused")
            .set_header("cho-protocol", "19")
            .body(
                resp.add(server_packet::notification("The server currently does not allow login, please wait or contact the administrator."))
                    .write_out(),
            );
    }

    // Blocked ip
    if cfg.login.disallowed_ip.contains(&request_ip) {
        return HttpResponse::Ok()
            .set_header("cho-token", "login_refused")
            .set_header("cho-protocol", "19")
            .body(
                resp.add(server_packet::login_reply(LoginFailed::InvalidCredentials))
                    .add(server_packet::notification("You are not allowed to login!"))
                    .write_out(),
            );
    }

    // Online user limit check
    if cfg.online_user_limit.enabled {
        // Get online players
        let online_users = read_lock!(bancho.player_sessions)
            .player_count
            .load(Ordering::SeqCst);

        if online_users >= cfg.online_user_limit.online_max {
            return HttpResponse::Ok()
                .set_header("cho-token", "login_refused")
                .set_header("cho-protocol", "19")
                .body(
                    resp.add(server_packet::notification(&format!(
                        "The current server allows up to {} people to be online, please wait...",
                        online_users
                    )))
                    .write_out(),
                );
        };
    };

    let failed_key = format!("{}-bancho_login_failed", &request_ip);
    let failed_count = database.redis.get(&failed_key).await.unwrap_or(0);
    // Too many failed in 300s, refuse login
    if failed_count > cfg.login.retry_max {
        warn!(
            "[LoginHandler] ip: {} login refused, beacuse too many login failed_count({}) in {}s;",
            request_ip, failed_count, cfg.login.retry_expire
        );
        return HttpResponse::Ok()
            .set_header("cho-token", "login_refused")
            .set_header("cho-protocol", "19")
            .body(
                resp.add(server_packet::login_reply(LoginFailed::ServerError))
                    .add(server_packet::notification(
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
        &bancho,
        &caches,
        &counter,
        &geo_db,
        cfg,
    )
    .await
    {
        Ok((packet_data, token)) => (packet_data, token),
        Err((error_str, packet_builder)) => {
            // Notification string
            let mut failed_notification = String::new();

            // Default failed login reply is InvalidCredentials
            let packet_builder = packet_builder
                .unwrap_or(resp.add(server_packet::login_reply(LoginFailed::InvalidCredentials)));

            // Record login failed
            counter
                .with_label_values(&["/bancho", "post", &format!("login.failed.{}", error_str)])
                .inc();
            // Increase failed count for this ip
            let failed_count: i32 = match database.redis.query("INCR", &[&failed_key]).await {
                Ok(i) => i,
                Err(err) => {
                    error!(
                        "[LoginHandler] Failed to INCR failed count for ip {}, err: {:?}",
                        request_ip, err
                    );
                    0
                }
            };
            let _ = database
                .redis
                .expire(&failed_key, cfg.login.retry_expire)
                .await;

            // Add notification string
            failed_notification.push_str(&format!("Login failed {} times!!\n", failed_count));

            // If reached the retires limit
            if failed_count > cfg.login.retry_max {
                failed_notification.push_str(&format!("Your login retries have reached the upper limit! \nPlease try again in {} seconds.", cfg.login.retry_expire));
                warn!(
                    "[LoginHandler] ip: {} login failed, count: {}, will temporarily restrict their login",
                    &request_ip, failed_count
                );
            } else {
                failed_notification.push_str(&format!(
                    "You can try {} more times.\n",
                    cfg.login.retry_max - failed_count + 1
                ))
            };

            // Returns
            (
                packet_builder
                    .add(server_packet::notification(&failed_notification))
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

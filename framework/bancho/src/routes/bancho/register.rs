use ntex::http::header;
use peace_utils::common::ContentDisposition;

use super::depends::*;

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub check: i32,
}

#[inline]
async fn parse_reg_form(mut form_data: Multipart) -> Option<RegisterForm> {
    let (mut username, mut email, mut password, mut check) = (None, None, None, None);
    while let Some(Ok(mut field)) = form_data.next().await {
        let disposition = unwrap_or_continue!(field.headers().get(&header::CONTENT_DISPOSITION));
        let disposition_str = unwrap_or_continue!(disposition.to_str());
        let name = unwrap_or_continue!(ContentDisposition::get_name(disposition_str)).to_string();
        if let Some(Ok(chunk)) = field.next().await {
            if let Ok(value) = String::from_utf8(chunk.to_vec()) {
                match name.as_str() {
                    "user[username]" => username = Some(value),
                    "user[user_email]" => email = Some(value),
                    "user[password]" => password = Some(value),
                    "check" => check = Some(value.parse::<i32>().ok()?),
                    _ => continue,
                }
            }
        }
    }
    Some(RegisterForm {
        username: username?,
        email: email?,
        password: password?,
        check: check?,
    })
}

#[post("/users")]
/// In-game registration handler
pub async fn osu_register(
    req: HttpRequest,
    form_data: Multipart,
    database: Data<Database>,
    geo_db: Data<Option<Reader<Mmap>>>,
    bancho: Data<Bancho>,
    caches: Data<Caches>,
) -> HttpResponse {
    let cfg_r = read_lock!(bancho.config);
    let cfg = &cfg_r.data;
    let r = &cfg.in_game_registration;
    // Register closed
    if !r.enabled {
        return HttpResponse::BadRequest().body(json!({
            "form_error": {
                "user": {
                    "username": ["In-game registration is currently closed! Please choose to register on the website or contact the administrator."],
                    "user_email": ["游戏内注册目前已被关闭！请选择在网站注册，或联系管理员。"],
                    "password": ["---hahahah--- NaN ---hahahaah---"]
                }
            }
        }));
    };

    let request_ip = peace_utils::web::get_realip(&req).await;
    if request_ip.is_err() {
        error!("Failed to get real ip");
        return HttpResponse::InternalServerError().body("Server Error");
    }
    let request_ip = request_ip.unwrap();

    // IP disallowed
    if cfg.server.ip_blacklist.contains(&request_ip) || r.disallowed_ip.contains(&request_ip) {
        error!("Disallowed ip finded, ip: {}", request_ip);
        return HttpResponse::InternalServerError().body("Server Error");
    }

    // Parse register form data
    let form_data = parse_reg_form(form_data).await;
    if form_data.is_none() {
        error!(
            "in-game registration failed! request_ip: {}, Missing required params",
            request_ip
        );
        return HttpResponse::BadRequest().body("Missing required params");
    };

    let form_data = form_data.unwrap();

    let mut username_errors = Vec::new();
    let mut email_errors = Vec::new();
    let mut password_errors = Vec::new();

    // Check username 1
    if !peace_constants::regexes::USERNAME_REGEX.is_match(&form_data.username) {
        username_errors.push("The length of the user name is 2-16 (alphanumeric as well as ][-_); 请使用英文注册，然后在网站设置中文名。");
    }

    // Check username 2
    if form_data.username.contains("_") && form_data.username.contains(" ") {
        username_errors.push(r#"You cannot include both "_" and " " in the name."#)
    }

    // disallowed names check
    if r.disallowed_usernames.contains(&form_data.username) {
        username_errors.push("This is a username that is not allowed, please change it.")
    }

    // sensitive word check
    for s_word in &cfg.server.sensitive_words {
        if form_data.username.contains(s_word) {
            username_errors.push("The username contains sensitive words, please change.");
            break;
        }
    }

    // Check username is already use (using name_safe to check)
    if database
        .pg
        .query_first(
            r#"SELECT 1 FROM "user"."base" WHERE "name_safe" = $1;"#,
            &[&form_data.username.to_lowercase().replace(" ", "_")],
        )
        .await
        .is_ok()
    {
        username_errors.push("Your username is already being used by someone else.")
    }

    // Check email 1
    if !peace_constants::regexes::EMAIL_REGEX.is_match(&form_data.email) {
        email_errors.push("Invalid email address.");
    }

    // Check email 2
    if r.disallowed_emails.contains(&form_data.email) {
        email_errors.push("Email that is not allowed, please change")
    }

    // Check email is already use
    if database
        .pg
        .query_first(
            r#"SELECT 1 FROM "user"."base" WHERE "email" = $1;"#,
            &[&form_data.email],
        )
        .await
        .is_ok()
    {
        email_errors.push("Your email is already being used by someone else.")
    }

    // Check password 1
    if (form_data.password.len() < 8) || (form_data.password.len() > 32) {
        password_errors.push("Password must be 8-32 characters.")
    }

    // Check password 2
    if r.disallowed_passwords.contains(&form_data.password) {
        password_errors
            .push("Don't use weak passwords, this is dangerous. Please change the password.")
    }

    // Return registration error if exists
    if !username_errors.is_empty() || !email_errors.is_empty() || !password_errors.is_empty() {
        return HttpResponse::BadRequest().body(json!({
            "form_error": {
                "user": {
                    "username": username_errors,
                    "user_email": email_errors,
                    "password": password_errors
                }
            }
        }));
    }

    if form_data.check == 0 {
        // Get country code
        let country = geo_db
            .as_ref()
            .as_ref()
            .and_then(
                |geo_db| match peace_utils::geoip::get_geo_ip_data(&request_ip, &geo_db) {
                    Ok(data) => data.country_code,
                    Err(_) => None,
                },
            )
            .unwrap_or("UN".to_string());

        // Get password md5, argon2
        let password_md5 = format!("{:x}", md5::compute(form_data.password));
        let password_argon2 = peace_utils::passwords::argon2_encode(password_md5.as_bytes()).await;

        // Cache it
        write_lock!(caches.argon2_cache).insert(password_argon2.clone(), password_md5);

        // Save to database
        // No need to do anything else, the trigger in the database will be completed:
        // safe name, registration time, create other data (such as stats, etc.)
        let user_id: i32 = match database
            .pg
            .query_first(
                r#"INSERT INTO "user"."base" ("name", "email", "password", "country") VALUES ($1, $2, $3, $4) RETURNING "id";"#,
                &[&form_data.username, &form_data.email, &password_argon2, &country],
            )
            .await {
                Ok(row) => row.get("id"),
                Err(error) => {
                    error!(
                        "Registration failed! username: {}, email: {}, request_ip: {}; error: {:?}",
                        form_data.username, form_data.email, request_ip, error
                    );
                    return HttpResponse::BadRequest().body("Unknown Error");
                }
            };

        info!(
            "New user {}({}) has registered! request_ip: {}",
            form_data.username, user_id, request_ip
        );
    };

    HttpResponse::Ok().body("ok")
}

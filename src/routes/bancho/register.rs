use super::depends::*;
use crate::utils;

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub check: i32,
}

#[post("/users")]
/// In-game registration handler
pub async fn osu_register(
    req: HttpRequest,
    mut form_data: Multipart,
    database: Data<Database>,
    geo_db: Data<Option<Reader<Mmap>>>,
    bancho_config: Data<RwLock<BanchoConfig>>,
    global_cache: Data<Caches>,
) -> HttpResponse {
    lazy_static::lazy_static! {
        static ref USERNAME_REGEX: Regex = Regex::new(r"(^[0-9a-zA-Z_ \[\]-]{2,16}$)|(^[\w \[\]-]{1,10}$)").unwrap();
        static ref EMAIL_REGEX: Regex = Regex::new(r"^[^@\s]{1,200}@[^@\s\.]{1,30}\.[^@\.\s]{2,24}$").unwrap();
    }

    let bancho_config = bancho_config.read().await;
    // Register closed
    if !bancho_config.registration_enabled {
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

    let request_ip = utils::get_realip(&req).await;
    if request_ip.is_err() {
        error!("Failed to get real ip");
        return HttpResponse::InternalServerError().body("Server Error");
    }
    let request_ip = request_ip.unwrap();

    // IP disallowed
    if bancho_config.ip_blacklist.contains(&request_ip)
        || bancho_config
            .registration_disallowed_ip
            .contains(&request_ip)
    {
        error!("Disallowed ip finded, ip: {}", request_ip);
        return HttpResponse::InternalServerError().body("Server Error");
    }

    // Parse register form data
    let form_data = {
        let mut temp: String = String::new();
        while let Some(item) = form_data.next().await {
            let mut field = item.unwrap();
            if let Some(content_type) = field.content_disposition() {
                let key = content_type.get_name();
                if key.is_none() {
                    continue;
                }
                while let Some(chunk) = field.next().await {
                    if chunk.is_err() {
                        continue;
                    }
                    let value = String::from_utf8(chunk.unwrap().to_vec()).unwrap_or(String::new());
                    if temp.len() > 0 {
                        temp.push('&');
                    }
                    temp.push_str(&format!("{}={}", key.unwrap(), value));
                }
            }
        }
        serde_qs::from_str::<RegisterForm>(
            &temp
                .replace("user[username]", "username")
                .replace("user[user_email]", "email")
                .replace("user[password]", "password"),
        )
    };
    if form_data.is_err() {
        error!(
            "in-game registration failed! request_ip: {}, error: {:?}",
            request_ip, form_data
        );
        return HttpResponse::BadRequest().body("Missing required params");
    };

    let form_data = form_data.unwrap();

    let mut username_errors = Vec::new();
    let mut email_errors = Vec::new();
    let mut password_errors = Vec::new();

    // Check username 1
    if !USERNAME_REGEX.is_match(&form_data.username) {
        username_errors.push("The length of the user name is 2-16 (alphanumeric as well as ][-_); if you use Chinese or non-English characters, the length is 1-10.");
    }

    // Check username 2
    if form_data.username.contains("_") && form_data.username.contains(" ") {
        username_errors.push(r#"You cannot include both "_" and " " in the name."#)
    }

    // disallowed names check
    if bancho_config
        .registration_disallowed_usernames
        .contains(&form_data.username)
    {
        username_errors.push("This is a username that is not allowed, please change it.")
    }

    // sensitive word check
    for s_word in &bancho_config.sensitive_words {
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
    if !EMAIL_REGEX.is_match(&form_data.email) {
        email_errors.push("Invalid email address.");
    }

    // Check email 2
    if bancho_config
        .registration_disallowed_emails
        .contains(&form_data.email)
    {
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
    if bancho_config
        .registration_disallowed_passwords
        .contains(&form_data.password)
    {
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
                |geo_db| match utils::get_geo_ip_data(&request_ip, &geo_db) {
                    Ok(data) => data.country_code,
                    Err(_) => None,
                },
            )
            .unwrap_or("UN".to_string());

        // Get password md5, argon2
        let password_md5 = format!("{:x}", md5::compute(form_data.password));
        let password_argon2 = utils::argon2_encode(password_md5.as_bytes()).await;

        // Cache it
        global_cache
            .argon2_cache
            .write()
            .await
            .insert(password_argon2.clone(), password_md5);

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

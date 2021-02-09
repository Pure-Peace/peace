use actix_multipart::Multipart;
use actix_web::post;
use futures::StreamExt;
use regex::Regex;
use serde::Deserialize;
use serde_json::json;

use crate::utils;

use super::depends::*;

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
    argon2_cache: Data<RwLock<Argon2Cache>>,
) -> HttpResponse {
    lazy_static::lazy_static! {
        static ref USERNAME_REGEX: Regex = Regex::new(r"(^[0-9a-zA-Z_ \[\]-]{2,16}$)|(^[\w \[\]-]{1,8}$)").unwrap();
        static ref EMAIL_REGEX: Regex = Regex::new(r"^[^@\s]{1,200}@[^@\s\.]{1,30}\.[^@\.\s]{2,24}$").unwrap();
    }

    let request_ip = utils::get_realip(&req).await;
    if request_ip.is_err() {
        return HttpResponse::InternalServerError().body("Server Error");
    }
    let request_ip = request_ip.unwrap();

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
        username_errors.push("The length of the user name is 2-15 (alphanumeric as well as ][-_); if you use Chinese or non-English characters, the length is 1-8.");
    }

    // Check username 2
    if form_data.username.contains("_") && form_data.username.contains(" ") {
        username_errors.push(r#"You cannot include both "_" and " " in the name."#)
    }

    // TODO: disallowed names check

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

    // Check email
    if !EMAIL_REGEX.is_match(&form_data.email) {
        email_errors.push("Invalid email address.");
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

    // Check password
    if (form_data.password.len() < 8) || !(form_data.password.len() > 32) {
        password_errors.push("Password must be 8-32 characters.")
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
        // Get password md5, argon2
        let password_md5 = format!("{:x}", md5::compute(form_data.password));
        let password_argon2 = utils::argon2_encode(password_md5.as_bytes()).await;

        // Cache it
        argon2_cache
            .write()
            .await
            .insert(password_argon2.clone(), password_md5);

        // Save to database
        // No need to do anything else, the trigger in the database will be completed:
        // safe name, registration time, create other data (such as stats, etc.)
        let user_id: i32 = match database
            .pg
            .query_first(
                r#"INSERT INTO "user"."base" ("name", "email", "password") VALUES ($1, $2, $3) RETURNING "id";"#,
                &[&form_data.username, &form_data.email, &password_argon2],
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
            "New user {}({}) has registered!",
            form_data.username, user_id
        );
    };

    HttpResponse::Ok().body("ok")
}

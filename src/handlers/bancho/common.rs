use async_std::sync::RwLock;
use peace_database::{serde_postgres, Database};

use crate::{objects::PlayerBase, types::Argon2Cache};

#[inline(always)]
pub async fn checking_password(
    player_base: &PlayerBase,
    password_hash: &String,
    argon2_cache: &RwLock<Argon2Cache>,
) -> bool {
    // Try read password hash from argon2 cache
    let cached_password_hash = {
        argon2_cache
            .read()
            .await
            .get(&player_base.password)
            .cloned()
    };

    // Cache hitted, checking
    if let Some(cached_password_hash) = cached_password_hash {
        debug!(
            "password cache hitted: {}({})",
            player_base.name, player_base.id
        );
        return &cached_password_hash == password_hash;
    }

    // Cache not hitted, do argon2 verify (ms level)
    let verify_result =
        peace_utils::passwords::argon2_verify(&player_base.password, password_hash).await;
    if verify_result {
        // If password is correct, cache it
        // key = argon2 cipher, value = password hash
        argon2_cache
            .write()
            .await
            .insert(player_base.password.clone(), password_hash.clone());
    }

    verify_result
}

#[inline(always)]
pub async fn get_player_base(username: &String, database: &Database) -> Option<PlayerBase> {
    let username_safe = username.to_lowercase().replace(" ", "_");

    // Select from database
    let from_base_row = match database
        .pg
        .query_first(
            r#"SELECT 
                    "id", "name", "u_name", "privileges", "country", "password"
                    FROM "user"."base" WHERE 
                    "name_safe" = $1 OR "u_name_safe" = $1;"#,
            &[&username_safe],
        )
        .await
    {
        Ok(from_base_row) => from_base_row,
        Err(_err) => {
            warn!(
                "failed to get playerbase: username ({}) is not exists! ",
                username
            );
            return None;
        }
    };
    // Try deserialize player base object
    match serde_postgres::from_row::<PlayerBase>(&from_base_row) {
        Ok(player_base) => Some(player_base),
        Err(err) => {
            error!(
                "could not deserialize playerbase: {}; err: {:?}",
                username, err
            );
            None
        }
    }
}

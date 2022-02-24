use std::io::Read;

use chrono::{DateTime, Local};
use colored::Colorize;
use serde_json::Value;

use peace_constants::common::DEFAULT_BANCHO_CONFIG_PATH;
use peace_database::Database;

use super::model::BanchoConfigData;

const TIPS: &str = "[BanchoConfig] Please check for errors.";

#[derive(Debug, Clone)]
pub struct BanchoConfig {
    pub name: String,
    pub comment: Option<String>,
    pub update_time: DateTime<Local>,
    pub data: BanchoConfigData,
}

impl BanchoConfig {
    #[inline]
    /// Initial bancho config from database
    pub async fn create(database: &Database) -> Option<Self> {
        match database
            .pg
            .query_simple(r#"SELECT * FROM "bancho"."config" WHERE "enabled" = TRUE LIMIT 1"#)
            .await
        {
            Ok(mut rows) => {
                // Not enabled settings exists
                if rows.len() == 0 {
                    return Self::config_not_found(database).await;
                };

                let data = Self::from_row(rows.remove(0), true);
                if data.is_some() {
                    return data;
                };

                // column 'settings' value is null
                return Self::config_not_found(database).await;
            }
            Err(err) => {
                error!("[BanchoConfig] Failed to get from database, err: {:?}", err);
                panic!("{}", TIPS)
            }
        }
    }

    #[inline]
    pub fn from_row(row: tokio_postgres::Row, parse_err_panic: bool) -> Option<Self> {
        // Try get settings json value
        let settings_val = match row.try_get::<'_, _, Option<Value>>("settings") {
            Ok(v) => v,
            Err(err) => {
                error!(
                    "[BanchoConfig] Failed to get column \"settings\" from database row, err: {:?}",
                    err
                );
                panic!("{}", TIPS)
            }
        };
        if let Some(settings_val) = settings_val {
            return Some(Self {
                name: row.get("name"),
                comment: row.get("comment"),
                update_time: row.get("update_time"),
                data: match serde_json::from_value(settings_val) {
                    Ok(d) => d,
                    Err(err) => {
                        error!("[BanchoConfig] Failed to parse json, err: {:?}", err);
                        if parse_err_panic {
                            panic!("{}", TIPS)
                        }
                        return None;
                    }
                },
            });
        }
        None
    }

    #[inline]
    pub async fn config_not_found(database: &Database) -> Option<Self> {
        let mut input = String::new();
        println!("\n{}\n", format!("**WARNING**: \n[BanchoConfig] Cannot found enabled config in database! 
You can manually set enable in the database table (bancho.config), or let us load the default config in \"{}\".", DEFAULT_BANCHO_CONFIG_PATH).yellow());
        println!("> load the default config? (y/n):");
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read from std::io::stdin.");
        if input.to_lowercase().trim() == "y" {
            let cfg = Self::get_from_file();
            if let Some(cfg) = &cfg {
                cfg.save_to_database(database, true).await;
            }
            return cfg;
        } else {
            println!(
                "\n{}\n",
                "You choose to configure manually, Peace will exit.".yellow()
            );
            panic!()
        };
    }

    #[inline]
    pub async fn save_to_database(&self, database: &Database, enabled: bool) -> bool {
        match database
            .pg
            .execute(
                r#"INSERT INTO "bancho"."config" ("name", comment, enabled, settings) 
                    VALUES ($1, $2, $3, $4) 
                    ON CONFLICT ("name") DO UPDATE SET enabled = EXCLUDED.enabled, settings = EXCLUDED.settings"#,
                &[
                    &self.name,
                    &self.comment,
                    &enabled,
                    &serde_json::to_value(self.data.clone()).unwrap(),
                ],
            )
            .await
        {
            Ok(_) => {
                info!("[BanchoConfig] Success to save! config name: {}", self.name);
                true
            }
            Err(err) => {
                error!(
                    "[BanchoConfig] Failed to save, config name: {}, err: {:?}",
                    self.name, err
                );
                false
            }
        }
    }

    #[inline]
    pub fn get_from_file() -> Option<Self> {
        let error = |e| {
            error!(
                "[BanchoConfig] Failed to read default config file: {}, err: {:?}",
                DEFAULT_BANCHO_CONFIG_PATH, e
            );
            None
        };
        let mut file = match std::fs::File::open(DEFAULT_BANCHO_CONFIG_PATH) {
            Ok(f) => f,
            Err(err) => return error(format!("{:?}", err)),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(err) => return error(format!("{:?}", err)),
        }
        let data = match serde_json::from_str::<BanchoConfigData>(&contents) {
            Ok(d) => d,
            Err(err) => return error(format!("{:?}", err)),
        };
        Some(Self {
            name: String::from("default"),
            comment: Some(String::from("default config for you")),
            update_time: Local::now(),
            data,
        })
    }

    #[inline]
    /// Update bancho config from database
    pub async fn update(&mut self, database: &Database) -> bool {
        let start = std::time::Instant::now();
        let new = BanchoConfig::create(database).await;
        if new.is_none() {
            error!("[BanchoConfig] update failed.");
            return false;
        };
        *self = new.unwrap();
        info!(
            "[BanchoConfig] new config({}) updated in {:?}; update time: {}",
            self.name,
            start.elapsed(),
            self.update_time
        );
        true
    }
}

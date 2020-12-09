use actix_web::web::Data;
use async_std::sync::RwLock;
use hashbrown::HashMap;

use colored::Colorize;

use crate::{database::Database, objects::PlayerSessions, types::ChannelList as ChannelListType};

use super::{base::ChannelBase, channel, Channel};

pub struct ChannelList {}

impl ChannelList {
    /// Initial channels list from database
    pub async fn new(
        database: &Database,
        player_sessions: Data<RwLock<PlayerSessions>>,
    ) -> ChannelListType {
        info!(
            "{}",
            "Initializing default chat channels...".bold().bright_blue()
        );
        let mut channels: ChannelListType = HashMap::new();
        // Get channels from database
        match database.pg.query(r#"SELECT "name", "title", "read_priv", "write_priv", "auto_join" FROM "bancho"."channels";"#, &[]).await {
            Ok(rows) => {
                let bases: Vec<ChannelBase> = serde_postgres::from_rows(&rows).unwrap();
                for base in bases {
                    channels.insert(base.name.clone(), Channel::from_base(&base, player_sessions.clone()).await);
                }
                info!("{}", format!("Channels successfully loaded: {:?};", channels.keys()).bold().green());
                channels
            },
            Err(err) => {
                error!("{}", format!("Failed to initialize chat channels, error: {:?}", err).bold().red());
                panic!();
            }
        }
    }
}

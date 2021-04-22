use std::sync::Arc;

use async_std::sync::RwLock;
use hashbrown::HashMap;

use colored::Colorize;
use peace_database::Database;

use crate::{objects::PlayerSessions, types::ChannelList};

use super::{base::ChannelBase, Channel};

pub struct ChannelListBuilder {}

impl ChannelListBuilder {
    /// Initial channels list from database
    pub async fn new(
        database: &Database,
        player_sessions: &Arc<RwLock<PlayerSessions>>,
    ) -> ChannelList {
        info!(
            "{}",
            "Initializing default chat channels...".bold().bright_blue()
        );
        let mut channels: ChannelList = HashMap::new();
        // Get channels from database
        match database.pg.query(r#"SELECT "name", "title", "read_priv", "write_priv", "auto_join" FROM "bancho"."channels";"#, &[]).await {
            Ok(rows) => {
                let channel_bases: Vec<ChannelBase> = serde_postgres::from_rows(&rows).unwrap();
                for base in channel_bases {
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

use actix_web::web::Data;
use async_std::sync::RwLock;

use super::{ChannelListBuilder, OsuApi};
use crate::utils::lock_wrapper;
use crate::{
    database::Database,
    settings::{bancho::BanchoConfig, model::LocalConfig},
};
use crate::{
    objects::{PPServerApi, PlayerSessions},
    renders::BanchoGet,
    types::ChannelList,
};

pub struct Bancho {
    pub player_sessions: Data<RwLock<PlayerSessions>>,
    pub channel_list: Data<RwLock<ChannelList>>,
    pub osu_api: Data<RwLock<OsuApi>>,
    pub pp_calculator: Data<PPServerApi>,
    pub render_get: Data<RwLock<BanchoGet>>,
    pub config: Data<RwLock<BanchoConfig>>,
    pub local_config: LocalConfig,
}

impl Bancho {
    pub async fn init(local_config: &LocalConfig, database: &Database) -> Self {
        // Create...
        let config = lock_wrapper(BanchoConfig::from_database(&database).await.unwrap());
        let player_sessions = lock_wrapper(PlayerSessions::new(1000, &database));
        let channel_list = lock_wrapper(ChannelListBuilder::new(database, &player_sessions).await);
        let osu_api = lock_wrapper(OsuApi::new(&config).await);
        let pp_calculator = Data::new(PPServerApi::new(&local_config.data));
        let render_get = lock_wrapper(BanchoGet::new(&config).await);

        Bancho {
            player_sessions,
            channel_list,
            osu_api,
            pp_calculator,
            render_get,
            config,
            local_config: local_config.clone(),
        }
    }
}

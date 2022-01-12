#[cfg(feature = "with_peace")]
use tokio::sync::RwLock;

use ntex::web::types::Data;
use peace_objects::osu_api::OsuApi;

use super::Caches;
use crate::renders::MainPage;
use crate::settings::LocalConfig;

#[cfg(feature = "with_peace")]
use peace_database::Database;
#[cfg(feature = "with_peace")]
use peace_objects::peace_api::PeaceApi;
#[cfg(feature = "with_peace")]
use peace_settings::bancho::BanchoConfig;
#[cfg(feature = "with_peace")]
use peace_utils::web::lock_wrapper as lw;

pub struct Glob {
    #[cfg(feature = "with_peace")]
    pub osu_api: Data<RwLock<OsuApi>>,
    #[cfg(not(feature = "with_peace"))]
    pub osu_api: Data<OsuApi>,

    #[cfg(feature = "with_peace")]
    pub peace_api: Data<PeaceApi>,

    pub caches: Data<Caches>,
    pub render_main_page: Data<MainPage>,
    pub local_config: LocalConfig,

    #[cfg(feature = "with_peace")]
    pub database: Data<Database>,
    #[cfg(feature = "with_peace")]
    pub config: Data<RwLock<BanchoConfig>>,
}

impl Glob {
    pub async fn init(
        local_config: &LocalConfig,
        #[cfg(feature = "with_peace")] database: &Database,
    ) -> Self {
        // Create...
        #[cfg(feature = "with_peace")]
        let cfg = BanchoConfig::create(&database).await.unwrap();
        #[cfg(feature = "with_peace")]
        let osu_api = lw(OsuApi::new(cfg.data.server.osu_api_keys.clone()).await);
        #[cfg(feature = "with_peace")]
        let config = lw(cfg);
        #[cfg(feature = "with_peace")]
        let peace_api = Data::new(PeaceApi::new(
            local_config.data.peace_key.clone(),
            local_config.data.peace_url.clone(),
        ));

        #[cfg(not(feature = "with_peace"))]
        let osu_api = Data::new(OsuApi::new(local_config.data.osu_api_keys.clone()).await);

        let render_main_page = Data::new(MainPage::new());
        let caches = Data::new(Caches::new(local_config.data.clone()));

        Glob {
            osu_api,
            #[cfg(feature = "with_peace")]
            database: Data::new(database.clone()),
            #[cfg(feature = "with_peace")]
            peace_api,
            caches,
            render_main_page,
            #[cfg(feature = "with_peace")]
            config,
            local_config: local_config.clone(),
        }
    }
}

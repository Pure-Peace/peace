use std::sync::Arc;

use askama::Template;
use tokio::sync::RwLock;

use peace_settings::bancho::BanchoConfig;

#[derive(Template, Clone)]
#[template(path = "bancho_get.html")]
pub struct BanchoGet {
    pub server_name: String,
    pub server_front: String,
    bancho_config: Arc<RwLock<BanchoConfig>>,
}

impl BanchoGet {
    pub async fn new(bancho_config: &Arc<RwLock<BanchoConfig>>) -> Self {
        let (server_name, server_front) = {
            let cfg = read_lock!(bancho_config);
            (
                cfg.data.server_info.name.clone(),
                cfg.data.server_info.front_url.clone(),
            )
        };
        BanchoGet {
            server_name: server_name,
            server_front: server_front,
            bancho_config: bancho_config.clone(),
        }
    }

    #[inline]
    pub async fn update(&mut self) {
        let (server_name, server_front) = {
            let cfg = read_lock!(self.bancho_config);
            (
                cfg.data.server_info.name.clone(),
                cfg.data.server_info.front_url.clone(),
            )
        };
        self.server_name = server_name;
        self.server_front = server_front;
    }
}

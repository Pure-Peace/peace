use crate::{database::Database, renders::BanchoGet};
use crate::{
    routes,
    types::{Argon2Cache, ChannelList},
};

use actix_cors::Cors;
use actix_web::{dev::Server, middleware::Logger, web::Data, App, HttpServer};
use async_std::sync::RwLock;
use config::Config;
use std::time::Instant;

use colored::Colorize;

use actix_web_prom::PrometheusMetrics;
use hashbrown::HashMap;
use maxminddb;
use prometheus::{opts, IntCounterVec};

use crate::handlers::bancho;
use crate::objects::{ChannelListBuilder, PlayerSessions};

use super::{bancho::BanchoConfig, model::Settings};

/// Actix before start
pub async fn before_start(cfg: &Config) -> (String, String) {
    // Load cfg
    let addr: String = cfg.get("addr").unwrap_or("127.0.0.1:8080".to_string());
    let log_format: String = cfg.get("logger.actix_log_format").unwrap_or_else(|error| {
        error!(
            "Failed to get config key: {}, use default value. Raw error: {}",
            "logger.actix_log_format".red(),
            error
        );
        r#"%{r}a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T" "%{Referer}i""#.to_string()
    });

    (addr, log_format)
}

/// Actix started
pub async fn started(_cfg: &Config, addr: &str) -> Instant {
    // Server started
    let text = format!("Service is Running at http://{}", addr)
        .bold()
        .green();
    info!("{}", text);
    Instant::now()
}

/// Actix stopped
pub async fn stopped(server: Server, start_time: Instant) -> std::io::Result<()> {
    // Waiting for server stopped
    let err = server.await;
    let title = format!("Service has Stopped!").bold().yellow();
    let time_string = format!("Service running time: {:?}\n", start_time.elapsed())
        .bold()
        .bright_blue();
    warn!("{} \n\n {}", title, time_string);
    err
}

/// Run actix
pub async fn start_server(
    cfg: Config,
    settings: Settings,
    database: Database,
    player_sessions: RwLock<PlayerSessions>,
) -> std::io::Result<()> {
    // Ready cfg
    let (addr, log_format): (String, String) = before_start(&cfg).await;
    let prom_exclude_endpoint_log = cfg
        .get_bool("prometheus.exclude_endpoint_log")
        .unwrap_or(false);
    let prom_endpoint = cfg
        .get_str("prometheus.endpoint")
        .unwrap_or("/metrics".to_string());
    let prom_namespace = cfg
        .get_str("prometheus.namespace")
        .unwrap_or("peace".to_string());
    let excludes_endpoint_log: Vec<String> = cfg
        .get("logger.exclude_endpoints")
        .unwrap_or(vec!["/favicon.ico".to_string()]);
    let excludes_endpoint_log_regex: Vec<String> =
        cfg.get("logger.exclude_endpoints_regex").unwrap_or(vec![]);
    let session_recycle_check_interval = cfg
        .get_int("bancho.session_recycle.check_interval")
        .unwrap_or(45) as u64;
    let recycle_check_duration = std::time::Duration::from_secs(session_recycle_check_interval);

    {
        // Ready prometheus
        let endpoint_tip = format!("Prometheus endpoint: {}", prom_endpoint).green();
        let namespace_tip = format!("Prometheus namespace: {}", prom_namespace).green();
        let prom_tip = format!(
            "Prometheus metrics address: http://{}{}",
            addr, prom_endpoint
        )
        .bold()
        .green();
        println!("> {}", endpoint_tip);
        println!("> {}", namespace_tip);
        println!("> {}\n", prom_tip);
    }

    // Labels
    let mut labels = std::collections::HashMap::new();
    labels.insert("job".to_string(), prom_namespace.to_string());

    // Counter
    let counter_opts = opts!("counter", "some random counter").namespace("api");
    let counter = IntCounterVec::new(counter_opts, &["endpoint", "method", "status"]).unwrap();

    // Init prometheus
    let prometheus = PrometheusMetrics::new(&prom_namespace, Some(&prom_endpoint), Some(labels));
    prometheus
        .registry
        .register(Box::new(counter.clone()))
        .unwrap();

    // Bancho config
    let bancho_config = Data::new(RwLock::new(
        BanchoConfig::from_database(&database).await.unwrap(),
    ));
    let bancho_config_cloned = bancho_config.clone();

    // Html renders
    let bancho_get_render = Data::new(RwLock::new(
        BanchoGet::new(bancho_config.clone().into_inner()).await,
    ));

    // Geo mmdb
    let geo_db = match settings.geoip.enabled {
        true => match maxminddb::Reader::open_mmap(&settings.geoip.mmdb_path) {
            Ok(reader) => Some(reader),
            Err(err) => {
                error!("Failed to start up geo-ip database reader: {:?}", err);
                None
            }
        },
        false => None,
    };
    let geo_db_data = Data::new(geo_db);

    // Player sessions
    let player_sessions = Data::new(player_sessions);
    let player_sessions_cloned = player_sessions.clone();

    // Channel list
    let channel_list: Data<RwLock<ChannelList>> = Data::new(RwLock::new(
        ChannelListBuilder::new(&database, player_sessions.clone().into_inner()).await,
    ));
    let channel_list_cloned: Data<RwLock<ChannelList>> = channel_list.clone();

    // Password cache
    let argon2_cache: Data<RwLock<Argon2Cache>> =
        Data::new(RwLock::new(HashMap::with_capacity(100)));

    // Start auto recycle task,
    // it will auto logout deactive players each interval
    async_std::task::spawn(async move {
        loop {
            async_std::task::sleep(recycle_check_duration).await;

            let session_timeout = bancho_config.read().await.timeout_player_session;
            bancho::sessions::recycle_handler(
                &player_sessions_cloned,
                &channel_list_cloned,
                session_timeout,
            )
            .await;
        }
    });

    // Run server
    info!("{}", "Starting http service...".bold().bright_blue());
    let server = HttpServer::new(move || {
        let settings_cloned = settings.clone();
        // Logger
        let make_logger = |mut logger: Logger| {
            for i in excludes_endpoint_log.iter() {
                logger = logger.exclude(i as &str);
            }
            for i in excludes_endpoint_log_regex.iter() {
                logger = logger.exclude_regex(i as &str);
            }
            logger
        };
        let logger = make_logger(match prom_exclude_endpoint_log {
            true => Logger::new(&log_format).exclude(&prom_endpoint),
            false => Logger::new(&log_format),
        });
        // App
        App::new()
            .wrap(logger)
            .wrap(prometheus.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_header()
                    .allow_any_method()
                    .supports_credentials(),
            )
            .app_data(player_sessions.clone())
            .app_data(channel_list.clone())
            .app_data(argon2_cache.clone())
            .app_data(geo_db_data.clone())
            .app_data(bancho_config_cloned.clone())
            .data(counter.clone())
            .data(database.clone())
            .data(bancho_get_render.clone())
            .configure(move |service_cfg| routes::init(service_cfg, settings_cloned))
    })
    .shutdown_timeout(2)
    .keep_alive(90)
    .bind(&addr)
    .unwrap()
    .run();
    // Wait for stopped
    let start_time = started(&cfg, &addr).await;
    stopped(server, start_time).await
}

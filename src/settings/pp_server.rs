use crate::database::Database;
use crate::routes;

use actix_cors::Cors;
use actix_web::{dev::Server, middleware::Logger, web::Data, App, HttpServer};
use async_std::sync::RwLock;
use config::Config;
use std::time::Instant;

use colored::Colorize;

use actix_web_prom::PrometheusMetrics;
use prometheus::{opts, IntCounterVec};

use super::{bancho::BanchoConfig, model::Settings};

/// Actix before start
pub async fn before_start(cfg: &Config) -> (String, String) {
    // Load cfg
    let addr: String = cfg
        .get("pp_server.addr")
        .unwrap_or("127.0.0.1:8086".to_string());
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
) -> std::io::Result<()> {
    // Ready cfg
    let (addr, log_format): (String, String) = before_start(&cfg).await;

    let prom_exclude_endpoint_log = cfg
        .get_bool("pp_server.prometheus.exclude_endpoint_log")
        .unwrap_or(false);
    let prom_endpoint = cfg
        .get_str("pp_server.prometheus.endpoint")
        .unwrap_or("/metrics".to_string());
    let prom_namespace = cfg
        .get_str("pp_server.prometheus.namespace")
        .unwrap_or("pp_server".to_string());

    let excludes_endpoint_log: Vec<String> = cfg
        .get("logger.exclude_endpoints")
        .unwrap_or(vec!["/favicon.ico".to_string()]);
    let excludes_endpoint_log_regex: Vec<String> =
        cfg.get("logger.exclude_endpoints_regex").unwrap_or(vec![]);

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
            .app_data(bancho_config_cloned.clone())
            .data(counter.clone())
            .data(database.clone())
            .configure(move |service_cfg| routes::pp_server::init(service_cfg, settings_cloned))
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

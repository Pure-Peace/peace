use crate::routes;
use crate::{database::Database, objects::Bancho};

use actix_cors::Cors;
use actix_web::{dev::Server, middleware::Logger, web::Data, App, HttpServer};
use async_std::channel::{unbounded, Receiver, Sender};
use memmap::Mmap;
use pyo3::Python;
use std::time::Instant;

use colored::Colorize;

use actix_web_prom::PrometheusMetrics;
use maxminddb::{self, Reader};
use prometheus::{opts, IntCounterVec};

use crate::handlers::bancho;
use crate::objects::Caches;

use crate::settings::model::{LocalConfig, Settings};

pub struct Peace {
    pub addr: String,
    pub bancho: Data<Bancho>,
    pub local_config: LocalConfig,
    pub database: Data<Database>,
    pub prometheus: PrometheusMetrics,
    pub counter: IntCounterVec,
    pub geo_db: Data<Option<Reader<Mmap>>>,
    pub global_cache: Data<Caches>,
    pub server: Option<Server>,
    pub sender: Sender<Option<Server>>,
    pub receiver: Receiver<Option<Server>>,
    pub start_time: Option<Instant>,
}

impl Peace {
    pub fn new(bancho: Data<Bancho>, database: Data<Database>) -> Self {
        let local_config = bancho.local_config.clone();
        let sets = &local_config.data;
        let addr = local_config
            .cfg
            .get("server.addr")
            .unwrap_or("127.0.0.1:8080".to_string());

        // Prometheus
        let (prometheus, counter) = Self::prom_init(&addr, sets);
        // Geo mmdb
        let geo_db = Data::new(Self::mmdb_init(sets));
        // Global cache
        let global_cache = Data::new(Caches::new());

        let (sender, receiver) = unbounded();

        Self {
            addr,
            bancho,
            local_config,
            database,
            prometheus,
            counter,
            geo_db,
            global_cache,
            server: None,
            sender,
            receiver,
            start_time: None,
        }
    }

    #[inline(always)]
    pub async fn run_server(&mut self) {
        // Run server
        info!("{}", "Starting http server...".bold().bright_blue());
        let server = {
            let settings_cloned = self.local_config.data.clone();
            let prom = self.prometheus.clone();
            let bancho = self.bancho.clone();
            let geo_db = self.geo_db.clone();
            let global_cache = self.global_cache.clone();
            let counter = self.counter.clone();
            let database = self.database.clone();
            let sender = Data::new(self.sender.clone());
            HttpServer::new(move || {
                // App
                App::new()
                    .wrap(Self::make_logger(&settings_cloned))
                    .wrap(prom.clone())
                    .wrap(
                        Cors::default()
                            .allow_any_origin()
                            .allow_any_header()
                            .allow_any_method()
                            .supports_credentials(),
                    )
                    .app_data(bancho.clone())
                    .app_data(geo_db.clone())
                    .app_data(global_cache.clone())
                    .app_data(sender.clone())
                    .app_data(database.clone())
                    .data(counter.clone())
                    .configure(|service_cfg| routes::peace::init(service_cfg, &settings_cloned))
            })
            .shutdown_timeout(2)
            .keep_alive(120)
            .bind(&self.addr)
            .unwrap()
            .run()
        };
        let _ = self.sender.send(Some(server)).await;
        self.start_time = Some(self.started());
    }

    pub async fn start(&mut self) -> std::io::Result<()> {
        self.python_init();

        self.session_recycle().await;

        // Start
        self.run_server().await;

        // Wait for stopped
        self.stopped().await
    }

    #[inline(always)]
    /// Initialize some methods into the global python interpreter
    ///
    /// NOTE: This is a temporary solution.
    /// Some problems cannot be solved temporarily.
    /// When the problem is solved, Python may be removed..
    ///
    pub fn python_init(&self) {
        info!("{}", "Initialing Python3...".bold().bright_blue());
        let code = include_str!("../utils/rijndael.py");
        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Err(err) = py.run(code, None, None) {
            error!("[Python] Failed to initial python3, err: {:?}", err);
            panic!()
        };
    }

    /// Server started
    pub fn started(&self) -> Instant {
        // Server started
        let text = format!("Server is Running at http://{}", self.addr)
            .bold()
            .green();
        info!("{}", text);
        Instant::now()
    }

    /// Server stopped
    pub async fn stopped(&self) -> std::io::Result<()> {
        let server = self.receiver.recv().await.unwrap().unwrap();
        // Waiting for server stopped
        let rx = self.receiver.clone();
        let srv = server.clone();
        async_std::task::spawn(async move {
            if let Ok(_) = rx.recv().await {
                warn!("Received shutdown signal, stop server...");
                srv.stop(true).await
            }
        });
        let err = server.await;
        let title = format!("Server has Stopped!").bold().yellow();
        let time_string = format!(
            "Server running time: {:?}\n",
            self.start_time.unwrap().elapsed()
        )
        .bold()
        .bright_blue();
        warn!("{} \n\n {}", title, time_string);
        err
    }

    pub fn make_logger(s: &Settings) -> Logger {
        let format = &s.logger.actix_log_format;
        let mut logger = match s.prom.exclude_endpoint_log {
            true => Logger::new(format).exclude(&s.prom.endpoint),
            false => Logger::new(format),
        };
        for i in s.logger.exclude_endpoints.iter() {
            logger = logger.exclude(i as &str);
        }
        for i in s.logger.exclude_endpoints_regex.iter() {
            logger = logger.exclude_regex(i as &str);
        }
        logger
    }

    /// Start auto recycle task,
    /// it will auto logout deactive players from bancho each interval
    pub async fn session_recycle(&mut self) {
        #[inline(always)]
        async fn handle_session_recycle(bancho: Data<Bancho>) {
            loop {
                let interval = bancho.config.read().await.session_recycle_check_interval as u64;
                // Sleep interval
                async_std::task::sleep(std::time::Duration::from_secs(interval)).await;
                // Check recycle
                bancho::sessions::recycle_handler(&bancho).await;
            }
        }
        info!("{}", "Starting session recycle...".bold().bright_blue());
        async_std::task::spawn(handle_session_recycle(self.bancho.clone()));
    }

    pub fn prom_init(addr: &String, sets: &Settings) -> (PrometheusMetrics, IntCounterVec) {
        // Ready prometheus
        println!(
            "> {}",
            format!("Prometheus endpoint: {}", sets.prom.endpoint).green()
        );
        println!(
            "> {}",
            format!("Prometheus namespace: {}", sets.prom.namespace).green()
        );
        println!(
            "> {}\n",
            format!(
                "Prometheus metrics address: http://{}{}",
                addr, sets.prom.endpoint
            )
            .bold()
            .green()
        );

        // Labels
        let mut labels = std::collections::HashMap::new();
        labels.insert("job".to_string(), sets.prom.namespace.to_string());

        // Counter
        let counter_opts = opts!("counter", "some random counter").namespace("api");
        let counter = IntCounterVec::new(counter_opts, &["endpoint", "method", "status"]).unwrap();

        // Init prometheus
        let prometheus = PrometheusMetrics::new(
            &sets.prom.namespace,
            Some(&sets.prom.endpoint),
            Some(labels),
        );
        prometheus
            .registry
            .register(Box::new(counter.clone()))
            .unwrap();

        (prometheus, counter)
    }

    pub fn mmdb_init(sets: &Settings) -> Option<Reader<Mmap>> {
        match sets.geoip.enabled {
            true => match maxminddb::Reader::open_mmap(&sets.geoip.mmdb_path) {
                Ok(reader) => Some(reader),
                Err(err) => {
                    error!("Failed to start up geo-ip database reader: {:?}", err);
                    None
                }
            },
            false => None,
        }
    }
}

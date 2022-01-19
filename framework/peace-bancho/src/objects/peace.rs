use {
    colored::Colorize,
    maxminddb::{self, Reader},
    memmap::Mmap,
    ntex::server::Server,
    ntex::web::{types::Data, App, HttpServer},
    peace_database::Database,
    prometheus::{opts, IntCounterVec},
    std::time::Instant,
    // use actix_web_prom::PrometheusMetrics;
    tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};

use crate::{handlers::bancho, objects::Bancho, objects::Caches, routes};

use ntex::time::Seconds;
use peace_settings::local::{LocalConfig, LocalConfigData};
use tokio::sync::Mutex;

pub struct Peace {
    pub addr: String,
    pub bancho: Data<Bancho>,
    pub local_config: LocalConfig,
    pub database: Data<Database>,
    // pub prometheus: PrometheusMetrics,
    pub counter: IntCounterVec,
    pub geo_db: Data<Option<Reader<Mmap>>>,
    pub caches: Data<Caches>,
    pub server: Option<Server>,
    pub sender: UnboundedSender<Option<Server>>,
    pub receiver: Data<Mutex<UnboundedReceiver<Option<Server>>>>,
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
        let counter = Self::prom_init(&addr, sets);
        // Geo mmdb
        let geo_db = Data::new(Self::mmdb_init(sets));
        // Global cache
        let caches = Data::new(Caches::new());

        let (sender, receiver) = unbounded_channel();

        Self {
            addr,
            bancho,
            local_config,
            database,
            // prometheus,
            counter,
            geo_db,
            caches,
            server: None,
            sender,
            receiver: Data::new(Mutex::new(receiver)),
            start_time: None,
        }
    }

    #[inline(always)]
    pub async fn run_server(&mut self) {
        // Run server
        info!("{}", "Starting http server...".bold().bright_blue());
        let server = {
            let s = self.local_config.data.clone();
            // let prom = self.prometheus.clone();
            let bancho = self.bancho.clone();
            let geo_db = self.geo_db.clone();
            let caches = self.caches.clone();
            let counter = self.counter.clone();
            let database = self.database.clone();
            let sender = Data::new(self.sender.clone());
            HttpServer::new(move || {
                // App
                App::new()
                    .wrap(peace_utils::web::make_logger(
                        &s.logger.server_log_format,
                        s.prom.exclude_endpoint_log,
                        &s.prom.endpoint,
                        &s.logger.exclude_endpoints,
                        &s.logger.exclude_endpoints_regex,
                    ))
                    // TODO: prometheus middleware
                    // .wrap(prom.clone())
                    /* .wrap(
                        Cors::default()
                            .allow_any_origin()
                            .allow_any_header()
                            .allow_any_method()
                            .supports_credentials(),
                    ) */
                    .app_data(bancho.clone())
                    .app_data(geo_db.clone())
                    .app_data(caches.clone())
                    .app_data(sender.clone())
                    .app_data(database.clone())
                    .data(counter.clone())
                    .configure(|service_cfg| routes::peace::init(service_cfg, &s))
            })
            .shutdown_timeout(Seconds(2))
            .keep_alive(120)
            .bind(&self.addr)
            .unwrap()
            .run()
        };
        let _ = self.sender.send(Some(server));
        self.start_time = Some(self.started());
    }

    pub async fn start(&mut self) -> std::io::Result<()> {
        self.session_recycle().await;

        // Start
        self.run_server().await;

        // Wait for stopped
        self.stopped().await
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
        let server = self.receiver.lock().await.recv().await.unwrap().unwrap();
        // Waiting for server stopped
        let rx = self.receiver.clone();
        let srv = server.clone();
        tokio::task::spawn(async move {
            if let Some(_) = rx.lock().await.recv().await {
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

    /// Start auto recycle task,
    /// it will auto logout deactive players from bancho each interval
    pub async fn session_recycle(&mut self) {
        #[inline(always)]
        async fn handle_session_recycle(bancho: Data<Bancho>) {
            loop {
                let interval = bancho
                    .config
                    .read()
                    .await
                    .data
                    .session_recycle
                    .check_interval as u64;
                // Sleep interval
                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
                // Check recycle
                bancho::sessions::recycle_handler(&bancho).await;
            }
        }
        info!("{}", "Starting session recycle...".bold().bright_blue());
        tokio::task::spawn(handle_session_recycle(self.bancho.clone()));
    }

    pub fn prom_init(addr: &String, sets: &LocalConfigData) -> IntCounterVec {
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
        /* let prometheus = PrometheusMetrics::new(
            &sets.prom.namespace,
            Some(&sets.prom.endpoint),
            Some(labels),
        );
        prometheus
            .registry
            .register(Box::new(counter.clone()))
            .unwrap(); */

        /* prometheus, */
        counter
    }

    pub fn mmdb_init(sets: &LocalConfigData) -> Option<Reader<Mmap>> {
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

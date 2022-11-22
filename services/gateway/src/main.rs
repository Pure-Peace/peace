#[macro_use]
extern crate peace_logs;

pub mod cmd;
pub mod components;
pub mod routes;

#[cfg(feature = "tls")]
pub mod tls;

#[tokio::main]
pub async fn main() -> Result<(), std::io::Error> {
    let args = cmd::PeaceGatewayArgs::get();
    peace_logs::init_with_args(args);

    let app = components::app(args);

    info!("\n\n{}\n", tools::pkg_metadata!(),);
    info!(
        ">> Routing method: [{}]",
        if args.hostname_router { "hostname" } else { "path" }
    );

    #[cfg(feature = "tls")]
    if args.tls {
        let https = tls::launch_https_server(app.clone(), args);
        if args.force_https {
            tokio::join!(tls::launch_ssl_redirect_server(args), https);
        } else {
            tokio::join!(components::launch_http_server(app, args), https);
        }
    } else {
        components::launch_http_server(app, args).await;
    }

    #[cfg(not(feature = "tls"))]
    components::launch_http_server(app, args).await;

    Ok(())
}

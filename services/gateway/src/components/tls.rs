use crate::components::cmd::PeaceGatewayArgs;

use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri},
    response::Redirect,
    BoxError, Router, Server,
};
use axum_server::tls_rustls::RustlsConfig;

pub fn redirect_replace(
    host: String,
    uri: Uri,
    http_port: &str,
    https_port: &str,
) -> Result<Uri, BoxError> {
    let mut parts = uri.into_parts();

    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

    if parts.path_and_query.is_none() {
        parts.path_and_query = Some("/".parse().unwrap());
    }

    let https_host = host.replace(http_port, https_port);
    parts.authority = Some(https_host.parse()?);

    Ok(Uri::from_parts(parts)?)
}

pub async fn launch_ssl_redirect_server(args: &PeaceGatewayArgs) {
    let http_port = args.http_addr.port().to_string();
    let https_port = args.https_addr.port().to_string();

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match redirect_replace(host, uri, &http_port, &https_port) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                warn!(%error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            },
        }
    };

    info!(">> [HTTP] (only redirect) listening on: {}", args.https_addr);
    Server::bind(&args.http_addr)
        .serve(redirect.into_make_service())
        .await
        .unwrap();
}

pub async fn launch_https_server(app: Router, args: &PeaceGatewayArgs) {
    let config = RustlsConfig::from_pem_file(
        args.ssl_cert
            .as_ref()
            .expect("ERROR: tls: Please make sure `--ssl-cert` are passed in."),
        args.ssl_key
            .as_ref()
            .expect("ERROR: tls: Please make sure `--ssl-key` are passed in."),
    )
    .await
    .unwrap();

    info!(">> [HTTPS] listening on: {}", args.https_addr);
    axum_server::bind_rustls(args.https_addr, config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

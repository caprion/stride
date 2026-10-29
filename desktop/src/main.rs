//! The desktop binary.
//!
//! ```text
//! stride                 serve the app and the API on 127.0.0.1:8000
//! stride convert-db      convert the plain database into an encrypted one
//! stride --help          this
//! ```
//!
//! The desktop is an app, not a server: one process, one port, the loopback
//! interface and nothing else.

use std::net::TcpListener;
use std::sync::Arc;

use anyhow::{Context, Result};

use stride_desktop::config;
use stride_desktop::db;
use stride_desktop::http::{self, Request, Response};
use stride_desktop::server::App;
use stride_desktop::tls;

const HELP: &str = "\
Stride — the desktop app.

Usage:
  stride [serve]        serve the SPA and the API (the default)
  stride convert-db     convert the plain database into an encrypted one
  stride --help         this text

Environment:
  STRIDE_BIND            the address to bind (default 127.0.0.1, loopback only)
  STRIDE_PORT            the port to bind (default 8000)
  STRIDE_TLS_HOST        the address the HTTPS listener binds (default 0.0.0.0)
  STRIDE_TLS_PORT        the port for it (default 8443)
  STRIDE_TLS_DIR         where the certificate and the key live
                         (default <data dir>/tls)
  STRIDE_TRUST_REMOTE    set it to trust the clients from the trusted networks
                         without a pairing code (default false)
  STRIDE_TRUSTED_NETWORKS the CIDRs those clients come from (default empty)
  STRIDE_DATA_DIR        where the database and the state file live
                         (default backend/data)
  STRIDE_DB_PATH         the database (default <data dir>/stride.db)
  STRIDE_DB_KEY          the key the database is encrypted with
  STRIDE_KEYVAULT_SECRET the Key Vault secret that holds the key
                         (default stride-db-key)
  STRIDE_CORS_ORIGINS    the origins that may call the API
                         (default http://localhost:5173)
  STRIDE_DISABLE_AUTH    set it to turn the capability gate off
";

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let command = arguments
        .iter()
        .find(|argument| !argument.starts_with('-'))
        .map(String::as_str)
        .unwrap_or("serve");

    match command {
        "serve" => serve(),
        "convert-db" => convert(),
        "help" => {
            print!("{HELP}");
            Ok(())
        }
        other => {
            eprintln!("stride: unknown command {other}");
            print!("{HELP}");
            Ok(())
        }
    }
}

/// Serve the app.
fn serve() -> Result<()> {
    let config = config::from_env().context("the configuration")?;
    let runtime = stride_desktop::build(&config).context("the application")?;

    if let Some(report) = &runtime.conversion {
        eprintln!(
            "stride: the database at {} was not encrypted; it has been converted to SQLCipher \
             ({}). The plain file is kept at {}.plaintext-backup",
            config.db_path.display(),
            report.summary(),
            config.db_path.display()
        );
        for line in &report.drift {
            eprintln!("stride: the schema drifted: {line}");
        }
    }
    eprintln!(
        "stride: the database key is from {}",
        runtime.key_source.name()
    );

    if !config.dist.join("index.html").is_file() {
        eprintln!(
            "stride: the app has not been built: {} has no index.html. \
             Build it with `cd frontend && npm install && npm run build`.",
            config.dist.display()
        );
    }

    let app: Arc<App> = Arc::new(runtime.app);

    // The HTTPS listener is for a phone's browser. It binds exactly what the
    // operator configured — no address detection, no external command — and it
    // runs only when there is a certificate to serve it with.
    start_tls(&config, Arc::clone(&app));

    let listener = TcpListener::bind(config.bind)
        .with_context(|| format!("could not bind {}", config.bind))?;
    eprintln!(
        "stride: serving {} on http://{}:{}",
        stride_desktop::describe(&config),
        config.bind.ip(),
        config.bind.port()
    );

    let handler: http::Handler =
        Arc::new(move |request: &Request| -> Response { app.handle(request) });
    http::serve(listener, handler).context("the server stopped")
}

/// Start the HTTPS listener, if there is a certificate.
///
/// The certificate is the operator's business: point `STRIDE_TLS_DIR` at the
/// directory that holds it. With no certificate the desktop serves plain HTTP
/// only, and says so.
fn start_tls(config: &config::Config, app: Arc<App>) {
    let server_config = match tls::load(&config.tls_dir) {
        Ok(Some(server_config)) => Arc::new(server_config),
        Ok(None) => {
            eprintln!(
                "stride: no TLS certificate in {} — the app is reachable over plain HTTP only",
                config.tls_dir.display()
            );
            return;
        }
        Err(err) => {
            eprintln!("stride: the TLS certificate could not be read: {err}");
            return;
        }
    };
    let listener = match TcpListener::bind(config.tls_bind) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!(
                "stride: could not bind the TLS listener on {}: {err}",
                config.tls_bind
            );
            return;
        }
    };
    eprintln!(
        "stride: serving https://{}:{}",
        config.tls_bind.ip(),
        config.tls_bind.port()
    );
    let handler: http::Handler =
        Arc::new(move |request: &Request| -> Response { app.handle(request) });
    let acceptor = Arc::clone(&server_config);
    let _ = std::thread::Builder::new()
        .name("stride-tls".to_string())
        .spawn(move || {
            let _ = http::serve_with(listener, handler, move |stream| {
                tls::accept(Arc::clone(&acceptor), stream)
            });
        });
}

/// Convert the plain database into an encrypted one.
fn convert() -> Result<()> {
    let config = config::from_env().context("the configuration")?;
    let (key, source) =
        db::resolve_key(config.db_key.clone(), &config.data_dir).context("the database key")?;
    eprintln!("stride: the database key is from {}", source.name());
    let report = db::convert_plaintext_to_encrypted(&config.db_path, &key)
        .with_context(|| format!("the database at {}", config.db_path.display()))?;
    eprintln!(
        "stride: converted {} ({} rows)",
        config.db_path.display(),
        report.rows
    );
    for (table, rows) in &report.tables {
        eprintln!("stride:   {table}: {rows} rows");
    }
    for line in &report.drift {
        eprintln!("stride:   the schema drifted: {line}");
    }
    Ok(())
}

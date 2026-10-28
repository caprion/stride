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

const HELP: &str = "\
Stride — the desktop app.

Usage:
  stride [serve]        serve the SPA and the API (the default)
  stride convert-db     convert the plain database into an encrypted one
  stride --help         this text

Environment:
  STRIDE_BIND            the address to bind (default 127.0.0.1, loopback only)
  STRIDE_PORT            the port to bind (default 8000)
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

    let listener = TcpListener::bind(config.bind)
        .with_context(|| format!("could not bind {}", config.bind))?;
    eprintln!(
        "stride: serving {} on http://{}:{}",
        stride_desktop::describe(&config),
        config.bind.ip(),
        config.bind.port()
    );

    let app: Arc<App> = Arc::new(runtime.app);
    let handler: http::Handler =
        Arc::new(move |request: &Request| -> Response { app.handle(request) });
    http::serve(listener, handler).context("the server stopped")
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

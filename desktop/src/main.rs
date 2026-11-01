//! The command line.
//!
//! Three commands, and no flags that change what the rig *is*:
//!
//! ```text
//! stride-desktop serve      serve the app and the router
//! stride-desktop migrate    bring the database up to the frozen schema
//! stride-desktop doctor     say what this rig is and where its files are
//! ```
//!
//! `serve` is the one a stranger runs. It opens the database, holds the
//! Garmin key (in memory, in the environment, or in a file with `0600`),
//! and serves the app on `http://127.0.0.1:8080`.
//!
//! Nothing here prints a key. The database key and the Garmin key are the
//! two secrets the rig holds, and neither of them can reach a line of
//! output: the types have no `Display`, and the doctor only ever says
//! *whether* a key is there.

use std::collections::{BTreeMap, BTreeSet};
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;

use stride_store::Store;

use stride_desktop::api::{Router, StubRouter};
use stride_desktop::dbkey::{self, DatabaseSecret};
use stride_desktop::error::{Error, Result};
use stride_desktop::key::KeyStore;
use stride_desktop::log;
use stride_desktop::paths;
use stride_desktop::server::{self, DesktopHost, Server};
use stride_desktop::spa::Spa;
use stride_desktop::trust::Trust;

/// The address the rig serves on when it is not told otherwise.
pub const DEFAULT_BIND: &str = "127.0.0.1";

/// The port the rig serves on when it is not told otherwise.
pub const DEFAULT_PORT: u16 = 8080;

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("stride-desktop: {error}");
            std::process::exit(1);
        }
    }
}

/// Run the command line.
fn run(arguments: &[String]) -> Result<i32> {
    let command = match arguments.first() {
        Some(command) => command.clone(),
        None => {
            print_help();
            return Ok(2);
        }
    };
    let rest = &arguments[1..];
    match command.as_str() {
        "serve" => serve(rest),
        "migrate" => migrate(rest),
        "doctor" => doctor(rest),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(0)
        }
        "version" | "--version" | "-V" => {
            println!("stride-desktop {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        other => Err(Error::config(format!(
            "{other} is not a command — try `stride-desktop --help`"
        ))),
    }
}

/// Serve the app and the router.
fn serve(arguments: &[String]) -> Result<i32> {
    let options = Options::parse(arguments)?;
    options.reject_unknown(&[
        "bind",
        "port",
        "data-dir",
        "static-dir",
        "tls-cert",
        "tls-key",
        "allow-plain-http",
    ])?;

    let directory = data_directory(&options)?;
    paths::ensure_data_dir(&directory)?;

    let database = paths::database_path(&directory);
    let database_key = DatabaseSecret::load_or_create(&paths::database_key_path(&directory))?;

    // The database is created with `0600`, so that the file and the
    // write-ahead log beside it are readable by their owner only.
    paths::ensure_private_file(&database)?;

    // A database that predates the encryption is converted once, and the
    // plain copy is kept beside it.
    if let Some(report) = dbkey::convert_plaintext_database(&database, &database_key)? {
        log::log(&format!(
            "converted the plain database at {} into an encrypted one; the plain copy is at {}",
            database.display(),
            dbkey::plaintext_backup_path(&database).display()
        ));
        let _ = report;
    }

    let store = Arc::new(Store::open_and_migrate(
        &database,
        &database_key.store_key()?,
    )?);
    let keys = garmin_key(&directory)?;
    let trust = Trust::from_environment()?;

    let bind: IpAddr = options
        .value("bind")
        .unwrap_or(DEFAULT_BIND)
        .parse()
        .map_err(|_| {
            Error::config(format!(
                "{} is not an address",
                options.value("bind").unwrap_or_default()
            ))
        })?;
    let port: u16 = match options.value("port") {
        Some(text) => text
            .parse()
            .map_err(|_| Error::config(format!("{text} is not a port")))?,
        None => DEFAULT_PORT,
    };
    let address = SocketAddr::new(bind, port);

    let tls_certificate = options.value("tls-cert").map(PathBuf::from);
    let tls_key = options.value("tls-key").map(PathBuf::from);
    let plain_allowed = options.flag("allow-plain-http");

    let listener = TcpListener::bind(address)
        .map_err(|source| Error::io(PathBuf::from(address.to_string()), source))?;
    let served = listener
        .local_addr()
        .map_err(|source| Error::io(PathBuf::from(address.to_string()), source))?;

    let spa = Spa::new(options.value("static-dir").map(PathBuf::from));
    let router: Arc<dyn Router> = Arc::new(StubRouter);
    let server = Server::new(trust, router, spa);

    let _host = DesktopHost::new(Arc::clone(&store), keys);

    println!("stride {}", env!("CARGO_PKG_VERSION"));
    println!("the app is on http://{served}");
    println!("the database is at {}", database.display());
    println!("{}", trust_line(server.trust()));

    match (tls_certificate, tls_key) {
        (Some(certificate), Some(key)) => {
            server::install_crypto_provider();
            let config = server::tls_config(&certificate, &key)?;
            println!(
                "the connection is TLS: the certificate is at {}",
                certificate.display()
            );
            log::log(&format!("serving on {served} with TLS"));
            server.serve_tls(listener, config)?;
        }
        (None, None) => {
            if !served.ip().is_loopback() && !plain_allowed {
                return Err(Error::config(format!(
                    "the rig is not serving plain HTTP on {}: a Garmin key would cross the network \
                     in the clear. Pass --tls-cert and --tls-key, or --allow-plain-http if the \
                     network is already yours (a tailnet, for example).",
                    served.ip()
                )));
            }
            if !served.ip().is_loopback() {
                log::log(&format!(
                    "serving plain HTTP on {} because it was asked to; the network is trusted by \
                     the athlete and by {TRUSTED_NETWORKS}",
                    served.ip()
                ));
            }
            log::log(&format!("serving on {served}"));
            server.serve(listener)?;
        }
        _ => {
            return Err(Error::config(
                "--tls-cert and --tls-key go together: one without the other is not a \
                 certificate",
            ));
        }
    }

    Ok(0)
}

/// Bring the database up to the frozen schema.
fn migrate(arguments: &[String]) -> Result<i32> {
    let options = Options::parse(arguments)?;
    options.reject_unknown(&["data-dir"])?;

    let directory = data_directory(&options)?;
    paths::ensure_data_dir(&directory)?;

    let database = paths::database_path(&directory);
    let database_key = DatabaseSecret::load_or_create(&paths::database_key_path(&directory))?;
    paths::ensure_private_file(&database)?;

    if let Some(report) = dbkey::convert_plaintext_database(&database, &database_key)? {
        println!(
            "converted the plain database into an encrypted one ({} rows); the plain copy is at {}",
            report.rows,
            dbkey::plaintext_backup_path(&database).display()
        );
    }

    let store = Store::open_and_migrate(&database, &database_key.store_key()?)?;
    let version = store.schema_version()?;
    println!(
        "the database at {} is at schema version {version}",
        database.display()
    );
    Ok(0)
}

/// Say what this rig is and where its files are.
fn doctor(arguments: &[String]) -> Result<i32> {
    let options = Options::parse(arguments)?;
    options.reject_unknown(&["data-dir"])?;

    let directory = data_directory(&options)?;
    let database = paths::database_path(&directory);
    let database_key = paths::database_key_path(&directory);
    let garmin_key_file = paths::garmin_key_path(&directory);

    println!("stride-desktop {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("the data directory   {}", directory.display());
    println!(
        "the database         {} ({})",
        database.display(),
        format_of(&database)?
    );
    println!(
        "the database key     {} ({})",
        database_key.display(),
        if database_key.exists() {
            "there"
        } else {
            "not yet — it is made on the first run"
        }
    );
    println!(
        "the Garmin key file  {} ({})",
        garmin_key_file.display(),
        if garmin_key_file.exists() {
            "there"
        } else {
            "not there"
        }
    );
    println!(
        "the Garmin key       {}",
        match garmin_key(&directory) {
            Ok(keys) => match keys.source() {
                Some(source) => format!("held, from {}", source.name()),
                None => "not held".to_string(),
            },
            Err(error) => format!("not held ({error})"),
        }
    );

    let trust = Trust::from_environment()?;
    println!(
        "the trusted networks {}",
        if trust.networks().is_empty() {
            format!("{TRUSTED_NETWORKS} is not set — the loopback only")
        } else {
            trust
                .networks()
                .iter()
                .map(|network| network.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        }
    );

    println!(
        "the app              {}",
        if stride_desktop::spa::bundle_is_built() {
            "built into this binary"
        } else {
            "not built — run `npm --prefix frontend run build` and build the rig again"
        }
    );

    if database.exists() {
        if let Ok(stored) = DatabaseSecret::load(&database_key) {
            if let Ok(store) = Store::open(&database, &stored.store_key()?) {
                println!("the schema version   {}", store.schema_version()?);
            }
        }
    }

    Ok(0)
}

/// The environment variable that names the trusted networks.
const TRUSTED_NETWORKS: &str = stride_desktop::trust::TRUSTED_NETWORKS_VARIABLE;

/// A sentence about what the rig answers.
fn trust_line(trust: &Trust) -> String {
    if trust.networks().is_empty() {
        return format!(
            "the rig answers the loopback, and no other network ({TRUSTED_NETWORKS} is not set)"
        );
    }
    let networks = trust
        .networks()
        .iter()
        .map(|network| network.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("the rig answers the loopback and {networks}")
}

/// The data directory: the option, or the environment, or the platform's.
fn data_directory(options: &Options) -> Result<PathBuf> {
    match options.value("data-dir") {
        Some(path) => Ok(PathBuf::from(path)),
        None => paths::data_dir(),
    }
}

/// The Garmin key: the environment, then the file, then nothing.
fn garmin_key(directory: &std::path::Path) -> Result<KeyStore> {
    let from_environment = KeyStore::from_environment()?;
    if from_environment.is_set() {
        return Ok(from_environment);
    }
    let file = paths::garmin_key_path(directory);
    if file.exists() {
        return KeyStore::from_file(&file);
    }
    Ok(KeyStore::empty())
}

/// What a file is: missing, plain, or encrypted.
fn format_of(path: &std::path::Path) -> Result<String> {
    use stride_store::export::{detect_format, Format};
    Ok(match detect_format(path)? {
        Format::Missing => "there is no file yet".to_string(),
        Format::Plaintext => "plain — it will be converted on the next run".to_string(),
        Format::Encrypted => "encrypted".to_string(),
    })
}

/// The options a command was given.
struct Options {
    values: BTreeMap<String, String>,
    flags: BTreeSet<String>,
}

impl Options {
    /// Read the options.
    fn parse(arguments: &[String]) -> Result<Options> {
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        let mut flags: BTreeSet<String> = BTreeSet::new();

        let mut index = 0;
        while index < arguments.len() {
            let argument = &arguments[index];
            let name = match argument.strip_prefix("--") {
                Some(name) => name,
                None => {
                    return Err(Error::config(format!(
                        "{argument} is not an option — the options are --name value"
                    )))
                }
            };
            if let Some((name, value)) = name.split_once('=') {
                values.insert(name.to_string(), value.to_string());
                index += 1;
                continue;
            }
            // A flag is an option with no value; whether it is a flag is
            // decided by the command that reads it.
            match arguments.get(index + 1) {
                Some(value) if !value.starts_with("--") => {
                    values.insert(name.to_string(), value.clone());
                    index += 2;
                }
                _ => {
                    flags.insert(name.to_string());
                    index += 1;
                }
            }
        }
        Ok(Options { values, flags })
    }

    /// The value of an option.
    fn value(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(|value| value.as_str())
    }

    /// Was a flag given?
    fn flag(&self, name: &str) -> bool {
        self.flags.contains(name)
    }

    /// Refuse an option this command does not know.
    ///
    /// A mistyped option that is quietly ignored is worse than one that
    /// stops the rig: `--datadir` instead of `--data-dir` would write the
    /// database somewhere the athlete did not ask for.
    fn reject_unknown(&self, known: &[&str]) -> Result<()> {
        for name in self.values.keys().chain(self.flags.iter()) {
            if !known.contains(&name.as_str()) {
                return Err(Error::config(format!(
                    "--{name} is not an option of this command"
                )));
            }
        }
        Ok(())
    }
}

/// The help text.
fn print_help() {
    println!(
        "\
stride {version} — the desktop rig: one binary that serves the app and the router

  stride-desktop serve     serve the app on http://127.0.0.1:8080
  stride-desktop migrate   bring the database up to the frozen schema
  stride-desktop doctor    say what this rig is and where its files are

the options of `serve`
  --bind <address>         the address to serve on (default {DEFAULT_BIND})
  --port <port>            the port to serve on (default {DEFAULT_PORT})
  --data-dir <directory>   where the files are (default {DATA_DIR_VARIABLE}, or the platform's)
  --static-dir <directory> a directory to serve the app from, for the development loop
  --tls-cert <file>        the certificate for a network the rig serves
  --tls-key <file>         the key for that certificate
  --allow-plain-http       serve a network without TLS, when the network is already yours

the environment
  {TRUSTED_NETWORKS}   the networks the rig answers on, as CIDR blocks
  {GARMIN_KEY_VARIABLE}       the Garmin Personal API key
  {DATA_DIR_VARIABLE}            where the rig keeps its files
",
        version = env!("CARGO_PKG_VERSION"),
        DEFAULT_BIND = DEFAULT_BIND,
        DEFAULT_PORT = DEFAULT_PORT,
        DATA_DIR_VARIABLE = paths::DATA_DIR_VARIABLE,
        GARMIN_KEY_VARIABLE = stride_desktop::key::GARMIN_KEY_VARIABLE,
        TRUSTED_NETWORKS = TRUSTED_NETWORKS,
    );
}

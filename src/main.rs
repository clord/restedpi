use color_eyre::Help;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use librpi::app;
use librpi::auth::password;
use librpi::config::Config;
use librpi::config::parse;
use librpi::webapp;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use structopt::StructOpt;
use warp::Filter;

use tracing::{error, info, warn};

// big picture:
// read configuration and decide what sensors and switches are available. start up application, then
// start running state machine. finally, present a rest api to the outside world to interact with the
// application.

#[derive(Debug, StructOpt)]
#[structopt(
    name = "restedpi",
    about = "restedpi talks to gpio, i2c, spi, etc., and provides easy io and scheduling"
)]
struct Opt {
    #[structopt(short, long, env, default_value = "info")]
    log_level: String,

    #[structopt(subcommand)]
    command: Command,

    /// Override the config file location. Will also be where other files are kept.
    #[structopt(short, long)]
    config_file: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Run the main server.
    Server,

    /// A REPL that shows how boolean expressions parse.
    BooleanRepl,

    /// Add a user to the config file
    AddUser {
        /// Username to use for this password
        #[structopt(short, long)]
        username: String,

        /// Specify password on argument list instead of prompting (note: bash_history)
        #[structopt(short, long)]
        password: Option<String>,
    },
}

/// Get the best location for config file
fn get_config_path(maybe_override: Option<PathBuf>) -> Option<PathBuf> {
    maybe_override.or_else(|| {
        // Try user config dir first
        if let Some(mut user_config) = dirs::config_dir() {
            user_config.push("restedpi");
            user_config.push("config.toml");
            if user_config.exists() {
                return Some(user_config);
            }
        }
        // Try system config
        let system_config = PathBuf::from("/etc/restedpi/config.toml");
        if system_config.exists() {
            return Some(system_config);
        }
        // No config file found - that's OK, we'll use env vars and defaults
        None
    })
}

fn get_config(config_file: Option<&PathBuf>) -> Result<Config, color_eyre::Report> {
    Config::load(config_file.map(|p| p.as_path()))
        .wrap_err("Failed to load configuration")
        .suggestion(
            "create a config file at ~/.config/restedpi/config.toml or /etc/restedpi/config.toml",
        )
        .suggestion("or pass --config-file <path> to use a config file at another location")
        .note(
            "settings can also be provided via environment variables \
             with the RESTEDPI_ prefix (e.g. RESTEDPI_PORT=8080)",
        )
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    let (command, config_file) = setup();
    match command {
        Command::AddUser { username, password } => {
            add_user(config_file.as_ref(), password, username)
        }
        Command::BooleanRepl => bool_repl(config_file.as_ref()),
        Command::Server => server(config_file).await,
    }
}

async fn server(config_file: Option<PathBuf>) -> Result<(), color_eyre::Report> {
    let config = get_config(config_file.as_ref())?;
    if let Some(app_secret_path) = &config.app_secret_path {
        let app_secret = fs::read_to_string(app_secret_path)
            .wrap_err_with(|| format!("Failed to read app secret from {:?}", app_secret_path))
            .with_suggestion(|| {
                format!(
                    "check that {:?} exists and is readable, \
                     or update 'app_secret_path' in config.toml",
                    app_secret_path
                )
            })?
            .trim()
            .to_string();
        // SAFETY: This is called early in main() before other threads are spawned
        unsafe { env::set_var("APP_SECRET", app_secret) };
    }
    let listen = config.listen.clone().unwrap_or("127.0.0.1".to_string());
    let port = config.port.unwrap_or(3030);
    let bus = config.i2cbus.unwrap_or(1);
    let tls_key_path = config.tls_key_path.clone();
    let tls_cert_path = config.tls_cert_path.clone();
    // For db_path: use config value, or config file directory, or current directory
    let db_path = config.db_path.unwrap_or_else(|| {
        config_file
            .as_ref()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    });
    let users = config.users.unwrap_or_else(HashMap::new).clone();
    let here = (config.lat, config.long);

    info!("Starting RestedPi server");
    info!("  I2C bus: {}", bus);
    info!("  Database path: {:?}", db_path);
    info!("  Location: ({}, {})", here.0, here.1);

    let app = app::channel::start_app(bus, here, &db_path, users)
        .await
        .wrap_err("Failed to start app")
        .with_suggestion(|| format!("check that the database path {:?} is accessible", db_path))
        .suggestion("set 'db_path' in your config.toml or use --config-file")?;

    let api = webapp::filters::graphql_api(app);

    let addr: SocketAddr = format!("{}:{}", listen, port)
        .parse()
        .wrap_err_with(|| format!("Invalid listen address '{}:{}'", listen, port))
        .suggestion(
            "set 'listen' to an IP address (e.g. 0.0.0.0) and 'port' to a port number \
             in config.toml, or via RESTEDPI_LISTEN and RESTEDPI_PORT",
        )?;
    let serve = warp::serve((api.with(warp::log("web"))).recover(webapp::handle_rejection));
    match (tls_key_path, tls_cert_path) {
        (Some(key_path), Some(cert_path)) => {
            info!("RestedPi listening: https://{}", addr);
            serve
                .tls()
                .cert_path(cert_path)
                .key_path(key_path)
                .run(addr)
                .await
        }
        (None, None) => {
            info!("RestedPi listening: http://{} (no TLS)", addr);
            serve.run(addr).await
        }
        (Some(_), None) => {
            return Err(eyre::eyre!("TLS key provided but missing certificate")
                .suggestion("set 'tls_cert_path' in config.toml")
                .suggestion("or remove 'tls_key_path' to serve plain HTTP"));
        }
        (None, Some(_)) => {
            return Err(eyre::eyre!("TLS certificate provided but missing key")
                .suggestion("set 'tls_key_path' in config.toml")
                .suggestion("or remove 'tls_cert_path' to serve plain HTTP"));
        }
    }
    Ok(())
}

fn bool_repl(config_file: Option<&PathBuf>) -> Result<(), color_eyre::Report> {
    let history_path = config_file
        .and_then(|p| p.parent())
        .map(|p| p.join("repl.history"))
        .unwrap_or_else(|| PathBuf::from("repl.history"));
    println!("restedpi boolean expression evaluator");
    println!("=====================================");
    println!();
    println!("example: ");
    println!("   a or b and x and y");
    println!();
    let mut rl = Editor::<()>::new();
    if rl.load_history(&history_path).is_err() {
        eprintln!("No previous history");
    }
    loop {
        match rl.readline(">> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                if line.trim().is_empty() {
                    continue;
                }
                match parse::bool_expr(&line) {
                    Ok(r) => println!("Result: {:?}", r),
                    Err(e) => error!("Unable to evaluate expression: {:?}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D");
                break;
            }
            Err(err) => {
                error!("readline error: {:?}", err);
                break;
            }
        }
    }
    if let Err(e) = rl.save_history(&history_path) {
        warn!("Unable to save REPL history to {:?}: {}", history_path, e);
    }
    Ok(())
}

fn add_user(
    config_file: Option<&PathBuf>,
    password: Option<String>,
    username: String,
) -> Result<(), color_eyre::Report> {
    let config_file = config_file.ok_or_else(|| {
        eyre::eyre!("Config file required for add-user command")
            .suggestion(
                "create a config file at ~/.config/restedpi/config.toml \
                 or /etc/restedpi/config.toml",
            )
            .suggestion("or pass --config-file <path> to use a config file at another location")
    })?;
    let mut config = get_config(Some(config_file))?;
    let password = match password {
        Some(p) => p,
        None => rpassword::read_password_from_tty(Some("User's Password: "))
            .wrap_err("Failed to read password from terminal")
            .suggestion("pass --password <password> if no interactive terminal is available")?,
    };
    if password.trim().len() < 8 {
        return Err(eyre::eyre!("Password is too short")
            .suggestion("use a password of at least 8 characters"));
    }
    info!(
        "Setting password for user '{}' in config file {:?}...",
        username, config_file
    );
    let hashed = password::hash(&password).wrap_err("Failed to hash password")?;
    let users = config.users.get_or_insert_with(HashMap::new);
    users.insert(username, hashed);
    // write config file back
    let as_str =
        toml::to_string(&config).wrap_err("Failed to serialize the updated configuration")?;
    fs::write(config_file, as_str)
        .wrap_err_with(|| format!("Failed to write config file {:?}", config_file))
        .with_suggestion(|| format!("check that {:?} is writable", config_file))?;
    info!("Success");
    Ok(())
}

fn setup() -> (Command, Option<PathBuf>) {
    // SAFETY: These set_var calls happen at program startup before any threads are spawned
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        unsafe { std::env::set_var("RUST_LIB_BACKTRACE", "1") };
    }
    color_eyre::install().unwrap();
    let Opt {
        log_level,
        config_file,
        command,
    } = Opt::from_args();
    if std::env::var("RUST_LOG").is_err() {
        unsafe { env::set_var("RUST_LOG", log_level) };
    }
    tracing_subscriber::fmt::init();
    let config_file = get_config_path(config_file);
    if let Some(ref path) = config_file {
        info!("Using config file: {:?}", path);
    } else {
        info!("No config file found, using environment variables and defaults");
    }
    (command, config_file)
}

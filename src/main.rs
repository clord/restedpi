use color_eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
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
use std::path::{Path, PathBuf};
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
fn get_config_path(maybe_override: Option<PathBuf>) -> PathBuf {
    maybe_override
        .or_else(|| {
            dirs::config_dir().and_then(|mut y| {
                y.push("restedpi");
                y.push("config.toml");

                if y.exists() { Some(y) } else { None }
            })
        })
        .unwrap_or_else(|| PathBuf::from("/etc/restedpi/config.toml"))
}

fn get_config(config_file: &Path) -> Result<Config, color_eyre::Report> {
    let contents = fs::read_to_string(config_file)
        .map_err(|e| eyre::eyre!("Failed to read config file {:?}: {}", config_file, e))?;

    match toml::from_str(&contents) {
        Ok(cfg) => Ok(cfg),
        Err(e) => {
            warn!("error parsing config, using defaults. error: {}", e);
            Ok(Config::new())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    let (command, config_file) = setup();
    let _ = command.bright_white();
    match command {
        Command::AddUser { username, password } => add_user(config_file, password, username),
        Command::BooleanRepl => bool_repl(config_file),
        Command::Server => server(config_file).await,
    }
}

async fn server(config_file: PathBuf) -> Result<(), color_eyre::Report> {
    let mut config_file = config_file.clone();
    let config = get_config(&config_file)?;
    if let Some(app_secret_path) = &config.app_secret_path {
        let app_secret = fs::read_to_string(app_secret_path)
            .map_err(|e| {
                eyre::eyre!(
                    "Failed to read app secret from {:?}: {}",
                    app_secret_path,
                    e
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
    let key_and_cert = config.key_and_cert_path.clone();
    let db_path = config.db_path.unwrap_or_else(|| {
        config_file.pop();
        config_file
    });
    let users = config.users.unwrap_or_else(HashMap::new).clone();
    let here = (config.lat, config.long);

    info!("Starting RestedPi server with I2C bus {:?}", bus);
    let app = app::channel::start_app(bus, here, &db_path, users)
        .await
        .map_err(|e| eyre::eyre!("Failed to start app: {}", e))?;

    let api = webapp::filters::graphql_api(app);

    let addr: SocketAddr = format!("{}:{}", listen, port)
        .parse()
        .map_err(|e| eyre::eyre!("Invalid listen address '{}:{}': {}", listen, port, e))?;
    let serve = warp::serve((api.with(warp::log("web"))).recover(webapp::handle_rejection));
    if let Some((key_path, cert_path)) = key_and_cert {
        info!("RestedPi listening: https://{}", addr);
        serve
            .tls()
            .cert_path(cert_path)
            .key_path(key_path)
            .run(addr)
            .await
    } else {
        error!("Missing keys in configuration; can't start in TLS mode. set key_and_cert_path");
    }
    Ok(())
}

fn bool_repl(config_file: PathBuf) -> Result<(), color_eyre::Report> {
    let mut history_path = config_file.clone();
    history_path.set_file_name("repl.history");
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
    rl.save_history(&history_path).unwrap();
    Ok(())
}

fn add_user(
    config_file: PathBuf,
    password: Option<String>,
    username: String,
) -> Result<(), color_eyre::Report> {
    let mut config = get_config(&config_file)?;
    let password = match password {
        Some(p) => p,
        None => rpassword::read_password_from_tty(Some("User's Password: "))
            .map_err(|e| eyre::eyre!("Failed to read password: {}", e))?,
    };
    if password.trim().len() < 8 {
        error!("password too short");
        return Err(librpi::error::Error::PasswordIssue.into());
    }
    info!(
        "Setting password for user '{}' in config file {:?}...",
        username, config_file
    );
    match password::hash(&password) {
        Ok(hashed) => {
            let users = config.users.get_or_insert_with(HashMap::new);
            users.insert(username, hashed);
            // write config file back
            match toml::to_string(&config) {
                Ok(as_str) => {
                    fs::write(&config_file, as_str).map_err(|e| {
                        eyre::eyre!("Failed to write config file {:?}: {}", config_file, e)
                    })?;
                    info!("Success");
                    Ok(())
                }
                Err(e) => {
                    error!("failed to save config: {}", e);
                    Err(
                        librpi::error::Error::Config(format!("failed to save config: {}", e))
                            .into(),
                    )
                }
            }
        }
        Err(e) => {
            error!("failed to hash password: {}", e);
            Err(librpi::error::Error::PasswordIssue.into())
        }
    }
}

fn setup() -> (Command, PathBuf) {
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
    (command, config_file)
}

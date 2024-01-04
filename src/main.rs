use color_eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use librpi::app;
use librpi::auth::password;
use librpi::config::parse;
use librpi::config::Config;
use librpi::webapp;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use warp::Filter;

use tracing::{error, info, warn};
use tracing_subscriber;

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
    Server {
        /// The secret used to hash tokens. changing will invalidate all existing tokens.
        #[structopt(short, long, env)]
        app_secret: String,
    },

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
            dirs::config_dir()
                .map(|mut y| {
                    y.push("restedpi");
                    y.push("config.toml");

                    if y.exists() {
                        Some(y)
                    } else {
                        None
                    }
                })
                .flatten()
        })
        .unwrap_or(std::path::PathBuf::from("/etc/restedpi/config.toml"))
}

fn get_config(config_file: &Path) -> Config {
    let contents = match fs::read_to_string(&config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("invalid config file {:?}: {}", &config_file, e);
            panic!("config file invalid");
        }
    };

    match toml::from_str(&contents) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!("error parsing config, using defaults. error: {}", e);
            Config::new()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), eyre::Error> {
    let (command, config_file) = setup();
    command.bright_white();
    match command {
        Command::AddUser { username, password } => add_user(config_file, password, username),
        Command::BooleanRepl => bool_repl(config_file),
        Command::Server { app_secret } => server(config_file, app_secret).await,
    }
}

async fn server(config_file: PathBuf, app_secret: String) -> Result<(), color_eyre::Report> {
    let mut config_file = config_file.clone();
    let config = get_config(&config_file);
    env::set_var("APP_SECRET", app_secret);
    let listen = config.listen.clone().unwrap_or("127.0.0.1".to_string());
    let port = config.port.unwrap_or(3030);
    let key_and_cert = config.key_and_cert_path.clone();
    config_file.pop();
    let users = config.users.unwrap_or_else(|| HashMap::new()).clone();
    let here = (config.lat, config.long);

    let app = app::channel::start_app(here, &config_file, users)
        .await
        .expect("app failed to start");

    let api = webapp::filters::graphql_api(app);

    let addr = SocketAddr::new(listen.parse().expect("IP address"), port);
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
    println!("");
    println!("example: ");
    println!("   a or b and x and y");
    println!("");
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
    let mut config = get_config(&config_file);
    let password = password
        .unwrap_or_else(|| rpassword::read_password_from_tty(Some("User's Password: ")).unwrap());
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
            let users = config.users.get_or_insert_with(|| HashMap::new());
            users.insert(username, hashed);
            // write config file back
            match toml::to_string(&config) {
                Ok(as_str) => {
                    fs::write(config_file, as_str).expect("failed to write change");
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
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }
    color_eyre::install().unwrap();
    let Opt {
        log_level,
        config_file,
        command,
    } = Opt::from_args();
    if std::env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", log_level);
    }
    tracing_subscriber::fmt::init();
    let config_file = get_config_path(config_file);
    (command, config_file)
}

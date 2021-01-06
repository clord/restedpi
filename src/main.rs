#[macro_use]
extern crate log;

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
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use warp::Filter;

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
async fn main() {
    let Opt {
        log_level,
        config_file,
        command,
    } = Opt::from_args();
    env::set_var("LOG", log_level);
    pretty_env_logger::init_custom_env("LOG");
    let config_file = get_config_path(config_file);

    match command {
        Command::AddUser { username, password } => {
            let mut config = get_config(&config_file);
            // TODO: Warn for existing user?

            let password = password.unwrap_or_else(|| {
                rpassword::read_password_from_tty(Some("User's Password: ")).unwrap()
            });
            if password.trim().len() < 8 {
                error!("password too short");
                return;
            }

            info!(
                "Setting password for user '{}' in config file {:?}...",
                username, config_file
            );

            match password::hash(&password, 1) {
                Ok(hashed) => {
                    let users = config.users.get_or_insert_with(|| HashMap::new());
                    users.insert(username, hashed);
                    // write config file back
                    match toml::to_string(&config) {
                        Ok(as_str) => {
                            fs::write(&config_file, as_str).expect("failed to write change");
                            info!("Success",);
                        }
                        Err(e) => {
                            error!("failed to save config: {}", e)
                        }
                    }
                }
                Err(e) => {
                    error!("failed to hash password: {}", e)
                }
            }
        }

        Command::BooleanRepl => {
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
        }

        Command::Server { app_secret } => {
            let mut config_file = config_file.clone();
            let config = get_config(&config_file);
            env::set_var("APP_SECRET", app_secret);

            let listen = config.listen.clone().unwrap_or("127.0.0.1".to_string());
            let port = config.port.unwrap_or(3030);
            let key_and_cert = config.key_and_cert_path.clone();
            config_file.pop();
            let users = config.users.unwrap_or_else(|| HashMap::new()).clone();

            let app = app::channel::start_app((config.lat, config.long), &config_file, users)
                .expect("app failed to start");
            let app = Arc::new(Mutex::new(app));

            let api = webapp::filters::api(app);
            let addr = SocketAddr::new(listen.parse().expect("IP address"), port);

            let serve = warp::serve(api.with(warp::log("web")).recover(webapp::handle_rejection));
            if let Some((key_path, cert_path)) = key_and_cert {
                info!("RestedPi listening: https://{}", addr);
                serve
                    .tls()
                    .cert_path(cert_path)
                    .key_path(key_path)
                    .run(addr)
                    .await
            } else {
                error!(
                    "Missing keys in configuration; can't start in TLS mode. set key_and_cert_path"
                );
                return ();
            }
        }
    }
}

use api::Api;
use config::Config;
use simple_logger::SimpleLogger;

mod api;
mod release;
mod db;
mod config;

const PATHS: [&str; 2] = [
    "~/.config/btapi/config.toml",
    "/etc/btapi/config.toml"
];

#[tokio::main]
async fn main() {
    let mut config: Option<Config> = None;
    for path in PATHS {
        if let Ok(cfg) = Config::open(shellexpand::full(path).unwrap().to_string()) {
            config = Some(cfg);
            break;
        }
    }
    let config = config.unwrap_or_default();

    SimpleLogger::new().with_level(config.log_level()).init().unwrap();

    let api = Api::new(config);
    api.run().await;
}

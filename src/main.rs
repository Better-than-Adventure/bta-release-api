use api::Api;
use log::warn;
use simple_logger::SimpleLogger;

mod api;
mod release;
mod db;

#[tokio::main]
async fn main() {
    SimpleLogger::new().with_level(log::LevelFilter::Debug).init().unwrap();

    let api = Api::new();
    api.run().await;
}

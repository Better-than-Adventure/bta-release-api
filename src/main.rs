use api::Api;
use simple_logger::SimpleLogger;

mod api;
mod release;
mod db;

#[tokio::main]
async fn main() {
    SimpleLogger::new().init().unwrap();

    let api = Api::new();
    api.run().await;
}

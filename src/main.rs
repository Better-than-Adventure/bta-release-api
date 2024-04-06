use api::Api;

mod api;
mod release;

#[tokio::main]
async fn main() {
    let api = Api::new();
    api.run().await;
}

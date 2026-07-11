use catsquad_api::server;
use catsquad_log::prelude::*;

#[tokio::main]
async fn main() {
    init_log();
    server().await;
}

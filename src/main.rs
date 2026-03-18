mod commands;
mod config;
mod oauth;
mod paths;
mod protobuf;
mod token;

#[tokio::main]
async fn main() {
    commands::cmd_info().await;
}

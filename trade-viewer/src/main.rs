use std::env;
use std::net::SocketAddr;

use axum::routing::get;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref API_PORT: u16 = env::var("API_PORT").unwrap_or("8000".to_owned()).parse::<u16>().unwrap();
}

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", get(index));
    let address = SocketAddr::from(([0, 0, 0, 0], *API_PORT));

    println!("Starting Order Entry API on port 8000");
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> String {
    "Hello".to_owned()
}

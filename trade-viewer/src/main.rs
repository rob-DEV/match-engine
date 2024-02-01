use std::net::SocketAddr;

use axum::routing::get;

#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", get(index));
    let address = SocketAddr::from(([0, 0, 0, 0], 8000));

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

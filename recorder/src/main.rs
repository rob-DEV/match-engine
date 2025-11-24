use crate::engine_out_msg_thread::msg_out_thread;
use common::transport::sequenced_message::SequencedEngineMessage;
use sqlx::postgres::PgPoolOptions;
use std::error::Error;

mod engine_out_msg_thread;
mod persistable_entities;
mod persistence;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Initializing Recorder...");
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect("postgres://root:password@localhost/engine")
        .await?;

    let persistence = persistence::Persistence::new(db);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SequencedEngineMessage>();

    let msg_thread = std::thread::spawn(move || msg_out_thread(3500, tx).unwrap());

    tokio::spawn(async move {
        while let Some(ev) = rx.recv().await {
            if let Err(e) = persistence.persist_event(ev).await {
                eprintln!("Persistence error: {}", e);
            }
        }
    });

    msg_thread.join().unwrap();

    Ok(())
}

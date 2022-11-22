use crate::{Message, SOCKET_ADDRESS};
use ruisseau_db::repository::Repository;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use tokio::net::UnixListener;

async fn listen() -> eyre::Result<()> {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@postgres/ruisseau".to_string());

    tracing::debug!("Connecting to {db_connection_str}");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .idle_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    tracing::debug!("Running database migrations");

    let listener = UnixListener::bind(SOCKET_ADDRESS)?;

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let stream = stream.into_std()?;
                if let Err(err) = handle_client(stream, &pool).await {
                    tracing::error!("IPC error: {err}")
                }
            }
            Err(err) => {
                tracing::error!("IPC error: {err}")
            }
        }
    }
}

async fn handle_client(mut stream: UnixStream, db: &PgPool) -> eyre::Result<()> {
    let mut data = vec![];
    let _ = stream.read_to_end(&mut data)?;
    let message: Message = rmp_serde::from_slice(&data)?;
    match message {
        Message::SetRepositoryDefaultBranch {
            repository_owner,
            repository_name,
            branch,
        } => {
            let repo = Repository::by_namespace(&repository_owner, &repository_name, db).await?;
            repo.set_default_branch(&branch, db).await?;
        }
    }
    Ok(())
}

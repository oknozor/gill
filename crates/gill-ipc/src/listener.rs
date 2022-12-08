use crate::{Message, SOCKET_ADDRESS};
use gill_db::repository::Repository;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::fs;
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::time::Duration;
use tokio::net::UnixListener;
use tracing::debug;

pub struct IPCListener;

impl IPCListener {
    pub async fn listen(self) -> anyhow::Result<()> {
        let db_connection_str = "postgres://postgres:postgres@localhost/gill";

        debug!("Connecting to {db_connection_str}");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .idle_timeout(Duration::from_secs(3))
            .connect(db_connection_str)
            .await
            .expect("can connect to database");

        let listener = UnixListener::bind(SOCKET_ADDRESS)?;
        debug!("Unix listener connected to {SOCKET_ADDRESS}");

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
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
}

async fn handle_client(mut stream: UnixStream, db: &PgPool) -> anyhow::Result<()> {
    let mut data = vec![];
    let _ = stream.read_to_end(&mut data)?;
    let message: Message = rmp_serde::from_slice(&data)?;
    debug!("Got ipc message: {message:?}");
    match message {
        Message::PostReceiveEvent {
            repository_owner,
            repository_name,
            git_ref,
            ..
        } => {
            let repository_name = repository_name
                .strip_suffix(".git")
                .expect("Invalid repo path, expected '.git' suffix");
            if let Some(branch) = git_ref.strip_prefix("refs/heads/") {
                let repo = Repository::by_namespace(&repository_owner, repository_name, db).await?;
                if repo.get_default_branch(db).await.is_none() {
                    repo.set_default_branch(branch, db).await?;
                    debug!("Default branch set '{branch}' for repo: {repository_owner}/{repository_name}");
                }
            };
        }
    }
    Ok(())
}

// Clean up the socket file when dropping the listener
// so we can safely recreate it on restart
impl Drop for IPCListener {
    fn drop(&mut self) {
        let _ = fs::remove_file(SOCKET_ADDRESS);
    }
}

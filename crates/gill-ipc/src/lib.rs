pub mod listener;

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::os::unix::net::UnixStream;

const SOCKET_ADDRESS: &str = "/tmp/gill-socket";

/// Gill Ipc Message, these are primarily used to write to the database
/// When a git hook occurs
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    PostReceiveEvent {
        repository_owner: String,
        repository_name: String,
        git_ref: String,
        sha: String,
    },
}

impl Message {
    pub fn send(self) -> eyre::Result<()> {
        let mut stream = UnixStream::connect(SOCKET_ADDRESS)?;
        let message = rmp_serde::to_vec(&self)?;
        stream.write_all(&message)?;
        Ok(())
    }
}

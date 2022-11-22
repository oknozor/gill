pub mod listener;

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::os::unix::net::UnixStream;

const SOCKET_ADDRESS: &str = "/tmp/ruisseau-socket";

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    SetRepositoryDefaultBranch {
        repository_owner: String,
        repository_name: String,
        branch: String,
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

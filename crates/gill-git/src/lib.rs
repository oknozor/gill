extern crate core;

use std::io::Write;
use std::{fs, io};

pub mod repository;
pub mod traversal;

pub fn append_ssh_key(ssh_key: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true) // This is needed to append to file
        .open("/home/git/.ssh/authorized_keys")?;

    write!(file, "command=\"./bin/gill-git-server\" {ssh_key}")
}

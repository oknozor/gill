use std::io::Write;
use std::{fs, io};

pub mod repository;

pub fn append_ssh_key(ssh_key: &str, user_id: i32) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true) // This is needed to append to file
        .open("/home/git/.ssh/authorized_keys")?;

    write!(
        file,
        r#"command="./bin/gill-git-server {user_id}" {ssh_key}"#
    )
}

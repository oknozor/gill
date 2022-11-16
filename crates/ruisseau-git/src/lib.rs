use git_repository::init::Error;
use git_repository::Repository;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};

mod init;

pub fn init_bare(namespace: &str, name: &str) -> Result<Repository, Error> {
    let path = PathBuf::from(namespace);
    if !path.exists() {
        fs::create_dir(&path).expect("Failed to create dir");
    }

    init::bare(path, name)
}

pub fn append_ssh_key(ssh_key: &str) -> io::Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true) // This is needed to append to file
        .open("/home/git/.ssh/authorized_keys")?;

    write!(file, "command=\"ruisseau-git-server\" {ssh_key}")
}

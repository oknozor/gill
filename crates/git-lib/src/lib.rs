use std::{fs, io};
use std::io::Write;
use std::path::PathBuf;
use git_repository::init::Error;
use git_repository::Repository;

mod init;

pub fn init_bare(name: &str) -> Result<Repository, Error> {
    let path = PathBuf::from("okno");
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

    write!(file, "command=\"gitserve\" {ssh_key}")
}

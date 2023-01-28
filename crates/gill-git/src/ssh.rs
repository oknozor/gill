use std::io;

pub fn append_key(ssh_key: &str, user_id: i32) -> io::Result<()> {
    imp::append_ssh_key(ssh_key, user_id, "/home/git/.ssh/authorized_keys")
}

mod imp {
    use std::io::Write;
    use std::path::Path;
    use std::{fs, io};

    pub fn append_ssh_key<S: AsRef<Path>>(ssh_key: &str, user_id: i32, path: S) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().write(true).append(true).open(path)?;

        writeln!(file, r#"command="gill-git-server {user_id}" {ssh_key}"#)
    }
}

#[cfg(test)]
mod test {
    use sealed_test::prelude::*;
    use speculoos::prelude::*;
    use std::fs;

    #[sealed_test]
    fn should_append_key_to_file() -> anyhow::Result<()> {
        let ed25519 = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIH8LAX4tJ/0CrE7CIsbi6J454nP67G0aCYK+cVHrdB3l okno@archlinux";

        let rsa = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC4oKbuajSfuNoLEhSqXoE+TLyFr0eopBDF3X= johnyboy@hometown";

        fs::File::create("authorized_keys")?;
        super::imp::append_ssh_key(ed25519, 1, "authorized_keys")?;
        super::imp::append_ssh_key(rsa, 2, "authorized_keys")?;
        let result = fs::read_to_string("authorized_keys")?;

        assert_that!(result).is_equal_to(
r#"command="gill-git-server 1" ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIH8LAX4tJ/0CrE7CIsbi6J454nP67G0aCYK+cVHrdB3l okno@archlinux
command="gill-git-server 2" ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQC4oKbuajSfuNoLEhSqXoE+TLyFr0eopBDF3X= johnyboy@hometown
"#.to_string());
        Ok(())
    }
}

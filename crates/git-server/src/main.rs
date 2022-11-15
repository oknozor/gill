use std::env;
use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::io::Write;

fn main() -> eyre::Result<()> {
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/home/git/log.txt")?;

    let cmd = env::var("SSH_ORIGINAL_COMMAND")?;
    let words = shellwords::split(&cmd)?;
    let verb = &words[0];
    let repo_path = &words[1];

    match verb.as_str() {
        "git-upload-pack" | "git-receive-pack" => {
            writeln!(log_file, "command={cmd}, args={repo_path}")?;

            let output = Command::new(verb)
                .current_dir("/home/git")
                .env("HOME", "/home/git")
                .stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .arg(repo_path)
                .output()?;

            let output = String::from_utf8_lossy(&output.stdout);

            writeln!(log_file, "{output}")?;
        }

        _  => writeln!(log_file, "unwknown command: {cmd}")?
    };

    Ok(())
}
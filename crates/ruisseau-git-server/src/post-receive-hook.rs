use std::env;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> eyre::Result<()> {
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/home/git/post-receive-logs.txt")?;

    let branch = env::var("BRANCH")?;
    let git_dir = env::var("GIT_DIR")?;
    let target = env::var("TARGET")?;

    writeln!(
        log_file,
        "Triggered post-receive hook barnch={branch}, git_dir={git_dir}, target={target}"
    )?;

    Ok(())
}

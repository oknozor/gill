use std::env;
use std::fs::OpenOptions;
use std::io::{stdin, BufRead};
use std::path::PathBuf;

fn main() -> eyre::Result<()> {
    let _log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/home/git/post-receive-logs.txt")?;

    let git_dir = env::var("GIT_DIR")?;

    // Post receive hooks args are received via stdin
    let args: String = stdin().lock().lines().filter_map(Result::ok).collect();

    let args: Vec<&str> = args.split(' ').collect();

    let [_, sha, git_ref] = args.as_slice() else {
        panic!("Unhandled post-receive hook arguments {args:?}");
    };

    let git_dir = PathBuf::from(git_dir).canonicalize()?;

    let repository_name = git_dir
        .file_name()
        .expect("GIT_DIR is not set")
        .to_string_lossy();

    let repository_owner = git_dir
        .parent()
        .expect("Failed to get repository owner from GIT_DIR")
        .file_name()
        .expect("Failed to get ownername from GIT_DIR (Utf8 error")
        .to_string_lossy();

    gill_ipc::Message::PostReceiveEvent {
        repository_owner: repository_owner.to_string(),
        repository_name: repository_name.to_string(),
        git_ref: git_ref.to_string(),
        sha: sha.to_string(),
    }
    .send()?;

    Ok(())
}

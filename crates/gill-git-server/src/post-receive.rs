use gill_db::repository::Repository;
use gill_db::PgPoolOptions;
use gill_settings::SETTINGS;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{stdin, BufRead};
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = PgPoolOptions::new()
        .max_connections(1)
        .idle_timeout(Duration::from_secs(3))
        // TODO
        .connect(&SETTINGS.database_url())
        .await
        .expect("can connect to database");

    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/home/git/post-receive-logs.txt")?;

    let git_dir = env::var("GIT_DIR")?;

    // Post receive hooks args are received via stdin
    let args: String = stdin().lock().lines().filter_map(Result::ok).collect();

    let args: Vec<&str> = args.split(' ').collect();

    let [_, _sha, git_ref] = args.as_slice() else {
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
        .expect("Failed to get owner name from GIT_DIR (Utf8 error")
        .to_string_lossy();

    let repository_name = repository_name
        .strip_suffix(".git")
        .expect("Invalid repo path, expected '.git' suffix");

    match git_ref.strip_prefix("refs/heads/") {
        Some(branch) => {
            let repo = Repository::by_namespace(&repository_owner, repository_name, &db).await?;
            if repo.get_default_branch(&db).await.is_none() {
                repo.set_default_branch(branch, &db).await?;
                writeln!(
                    log_file,
                    "default branch {branch} set for {repository_owner}/{repository_name}"
                )?;
            } else {
                writeln!(log_file, "already have a default branch")?;
            }
        }
        None => writeln!(log_file, "branch not found")?,
    }

    log_file.flush()?;
    Ok(())
}

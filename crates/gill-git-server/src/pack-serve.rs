use gill_db::repository::Repository;
use gill_db::user::User;
use gill_db::PgPoolOptions;
use gill_settings::SETTINGS;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{exit, Command, Stdio};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("/home/git/log.txt")?;

    let db = PgPoolOptions::new()
        .max_connections(1)
        .idle_timeout(Duration::from_secs(3))
        .connect(&SETTINGS.database_url())
        .await
        .expect("can connect to database");

    let args: Vec<String> = env::args().collect();
    writeln!(log_file, "args: {args:?}")?;
    let [_, user_id] = args.as_slice() else {
        panic!("Expected user_id in pack-serve arguments {args:?}");
    };
    let user_id = user_id.parse()?;
    let cmd = env::var("SSH_ORIGINAL_COMMAND")?;
    let words = shellwords::split(&cmd)?;
    let verb = &words[0];
    let repo_path = &words[1];
    let (owner, repo_name) = repo_path.rsplit_once('/').expect("/ in repo path");
    // Ensure we are stripping out any absolute path components
    let owner = owner.split('/').last().expect("owner");
    let repo_name = repo_name.strip_suffix(".git").expect(".git prefix");

    if Repository::by_namespace(owner, repo_name, &db)
        .await
        .is_err()
    {
        eprintln!("Repository {owner}/{repo_name} not found");
        exit(1);
    } else {
        eprintln!("Repository found {owner}/{repo_name}");
    }

    let user = User::by_id(user_id, &db).await?;
    if user
        .get_local_repository_by_name(repo_name, &db)
        .await
        .is_err()
    {
        eprintln!("You don't have access to {owner}/{repo_name}");
        exit(2);
    } else {
        eprintln!("Push access granted for user {}", user.username);
    }

    match verb.as_str() {
        "git-upload-pack" | "git-receive-pack" => {
            Command::new(verb)
                .current_dir("/home/git")
                .env("HOME", "/home/git")
                .stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit())
                .arg(repo_path)
                .output()?;

            writeln!(log_file, "cmd {cmd}")?
        }

        _ => writeln!(log_file, "Unknown command: {cmd}")?,
    };

    log_file.flush()?;

    Ok(())
}

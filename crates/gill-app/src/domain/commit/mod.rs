use crate::domain::repository::Repository;
use crate::domain::user::User;
use crate::error::AppResult;
use gill_git::commits::OwnedCommit;
use gill_git::diffs::Diff;
use gill_git::GitRepository;
use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub summary: String,
    pub body: Option<String>,
    pub author: Author,
    pub created_at: u32,
    pub authored_at: u32,
}

#[derive(Debug, Clone)]
pub enum Author {
    Known(String),
    Raw(String),
}

impl Commit {
    async fn from_git_commit(commit: OwnedCommit, db: &PgPool) -> AppResult<Self> {
        let author = User::by_email(&commit.email, db).await;
        let author = match author {
            Ok(author) => Author::Known(author.username),
            Err(_) => Author::Raw(commit.author),
        };

        Ok(Self {
            id: commit.id,
            summary: commit.summary,
            body: None,
            author,
            created_at: commit.created_at,
            authored_at: commit.authored_at,
        })
    }
}

impl Repository {
    pub async fn history(
        owner: &str,
        name: &str,
        branch: &str,
        db: &PgPool,
    ) -> AppResult<Vec<Commit>> {
        let repo = GitRepository::open(owner, name)?;
        let git_commits = repo.history(branch)?;
        let mut commits = vec![];

        // TODO: Sql query to resolve all username onces
        for commit in git_commits {
            let commit = Commit::from_git_commit(commit, db).await?;
            commits.push(commit)
        }

        Ok(commits)
    }

    pub async fn commit_with_diff(
        owner: &str,
        name: &str,
        sha: &str,
        db: &PgPool,
    ) -> AppResult<(Commit, Vec<Diff>)> {
        let repo = GitRepository::open(owner, name)?;
        let git_commits = repo.find_commit(sha)?;
        let commit = Commit::from_git_commit(git_commits, db).await?;
        let diff = repo.commit_diff(sha)?;
        Ok((commit, diff))
    }
}

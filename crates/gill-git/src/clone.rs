use crate::GitRepository;
use git_repository::clone::PrepareFetch;
use git_repository::progress::Discard;
use git_repository::remote::fetch::Status;
use git_repository::{create, interrupt, open, remote, worktree};

impl GitRepository {
    fn clone(&self) -> anyhow::Result<()> {
        let repository_path = self.inner.path();
        let mut dest = self.inner.path().to_path_buf();
        dest.pop();
        let dest = dest.join("non-bare-copy");
        let mut prepare = PrepareFetch::new(
            repository_path,
            dest,
            create::Kind::WithWorktree,
            create::Options::default(),
            {
                let mut opts = open::Options::default();
                opts.permissions.config.git_binary = true;
                opts
            },
        )?;
        let (mut checkout, fetch_outcome) =
            prepare.fetch_then_checkout(Discard, &interrupt::IS_INTERRUPTED)?;

        let (repo, outcome) = checkout.main_worktree(Discard, &interrupt::IS_INTERRUPTED)?;

        match fetch_outcome.status {
            Status::NoPackReceived { .. } => {
                unreachable!("clone always has changes")
            }
            Status::DryRun { .. } => unreachable!("dry-run unsupported"),
            Status::Change { update_refs, .. } => {
                let remote = repo
                    .find_default_remote(remote::Direction::Fetch)
                    .expect("one origin remote")?;
                let ref_specs = remote.refspecs(remote::Direction::Fetch);
            }
        };

        if let worktree::index::checkout::Outcome {
            collisions, errors, ..
        } = outcome
        {
            if !(collisions.is_empty() && errors.is_empty()) {
                let mut messages = Vec::new();
                if !errors.is_empty() {
                    messages.push(format!("kept going through {} errors(s)", errors.len()));
                    for record in errors {
                        eprintln!("{}: {}", record.path, record.error);
                    }
                }
                if !collisions.is_empty() {
                    messages.push(format!("encountered {} collision(s)", collisions.len()));
                    for col in collisions {
                        eprintln!("{}: collision ({:?})", col.path, col.error_kind);
                    }
                }
            }
        }

        Ok(())
    }
}

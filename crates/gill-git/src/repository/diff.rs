use crate::repository::ref_to_tree;
use git_repository::object::tree::diff::change::Event;
use git_repository::object::tree::diff::{Action, Change};
use git_repository::objs::tree::EntryMode;
use std::fmt::Error;

pub fn diff() -> anyhow::Result<()> {
    let repo = git_repository::discover(".")?;

    let tree = repo.head()?.peel_to_commit_in_place()?.tree()?;
    let other = "heads/test1";
    let other = ref_to_tree(Some(other), &repo)?;
    tree.changes()
        .for_each_to_obtain_tree(&other, |changes: Change| {
            println!("{changes:?}");
            match changes.event {
                // TODO: get addition and convert to HTML just like in
                //  repository::traversal
                Event::Addition { id, entry_mode } => {
                    match entry_mode {
                        EntryMode::Tree => {}
                        EntryMode::Blob => {}
                        EntryMode::BlobExecutable => {}
                        EntryMode::Link => {}
                        EntryMode::Commit => {}
                    }
                    todo!()
                }
                // TODO: get deletion and convert to HTML just like in
                //  repository::traversal
                Event::Deletion { id, entry_mode } => {
                    match entry_mode {
                        EntryMode::Tree => {}
                        EntryMode::Blob => {}
                        EntryMode::BlobExecutable => {}
                        EntryMode::Link => {}
                        EntryMode::Commit => {}
                    }
                    todo!()
                }
                // TODO: get modification mix and match then convert to htlm with modification line
                //  marked. We probably want to take a look at git delta to do this
                Event::Modification {
                    previous_entry_mode,
                    previous_id,
                    entry_mode,
                    id,
                } => {
                    todo!()
                }
            }
            Ok::<Action, Error>(Action::Continue)
        })?;

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::repository::diff::diff;

    #[test]
    fn test() {
        diff().unwrap()
    }
}

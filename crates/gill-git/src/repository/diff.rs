use crate::repository::{ref_to_tree, GitRepository};
use git_repository::bstr::ByteSlice;
use git_repository::object::tree::diff::change::Event;
use git_repository::object::tree::diff::{Action, Change};
use git_repository::objs::tree::EntryMode;
use git_repository::{object, Id};

use imara_diff::intern::InternedInput;
use imara_diff::{Algorithm, UnifiedDiffBuilder};

pub fn diff_it(repo: &GitRepository) -> anyhow::Result<()> {
    repo.diff("main", "testdiff")?;
    Ok(())
}

#[derive(Debug, Default)]
pub struct DiffBuilder {
    out: String,
}

impl DiffBuilder {
    fn changed(&mut self, file_path: &str, previous_id: &Id, id: &Id, diff: String) {
        // TODO: find the correct mode (100644 is blob)
        let previous_id = &previous_id.to_string()[0..7];
        let id = &id.to_string()[0..7];
        let diff = format!(
            r#"diff --git a/ {file_path} b/{file_path}
index {previous_id}..{id} 100644
--- a/{file_path}
+++ b/{file_path}
{diff}"#
        );
        self.out.push_str(&diff);
    }

    fn addition(&mut self, file_path: &str, id: &Id, diff: String) {
        // TODO: find the correct mode (100644 is blob)
        let id = &id.to_string()[0..7];
        let diff = format!(
            r#"diff --git a/ {file_path} b/{file_path}
new file mode 100644
index 0000000..{id}
--- /dev/null
+++ b/{file_path}
{diff}"#
        );
        self.out.push_str(&diff);
    }

    fn deletion(&mut self, file_path: &str, previous_id: &Id, diff: String) {
        let previous_id = &previous_id.to_string()[0..7];
        // TODO: find the correct mode (100644 is blob)
        let diff = format!(
            r#"diff --git a/ {file_path} b/{file_path}
deleted file mode 100644
index {previous_id}..0000000
--- a/{file_path}
+++ /dev/null
{diff}"#
        );
        self.out.push_str(&diff);
    }
}

impl GitRepository {
    pub fn diff(&self, branch: &str, other: &str) -> anyhow::Result<String> {
        let repository = &self.inner;
        let mut diff_builder = DiffBuilder::default();
        let tree = ref_to_tree(Some(&format!("heads/{branch}")), repository)?;
        let other = ref_to_tree(Some(&format!("heads/{other}")), repository)?;
        tree.changes()
            .track_path()
            .for_each_to_obtain_tree(&other, |changes: Change| {
                let location = changes.location.to_str().expect("bstr to str");

                match changes.event {
                    // TODO: get modification mix and match then convert to htlm with modification line
                    //  marked. We probably want to take a look at git delta to do this (they use syntect too)
                    Event::Modification {
                        previous_entry_mode,
                        previous_id,
                        entry_mode,
                        id,
                    } => match (previous_entry_mode, entry_mode) {
                        (EntryMode::Blob, EntryMode::Blob)
                        | (EntryMode::Blob, EntryMode::BlobExecutable)
                        | (EntryMode::BlobExecutable, EntryMode::Blob)
                        | (EntryMode::BlobExecutable, EntryMode::BlobExecutable) => {
                            let object = repository.find_object(previous_id)?;
                            // Todo: Any chance we could avoid allocation here ?
                            //      Maybe we need to collect all git objects in a upper structure and handle the writing there ?
                            let previous_content =
                                String::from_utf8_lossy(&object.data).to_string();
                            let object = repository.find_object(id)?;
                            let content = String::from_utf8_lossy(&object.data).to_string();
                            let input =
                                InternedInput::new(previous_content.as_str(), content.as_str());
                            let diff: String = imara_diff::diff(
                                Algorithm::Histogram,
                                &input,
                                UnifiedDiffBuilder::new(&input),
                            );
                            diff_builder.changed(location, &previous_id, &id, diff);
                        }
                        _ => {
                            // TODO: Ignoring everything but blob to blob diff for now
                        }
                    },
                    Event::Addition { entry_mode, id } => match entry_mode {
                        EntryMode::Tree => {}
                        EntryMode::Blob | EntryMode::BlobExecutable => {
                            let object = repository.find_object(id)?;
                            let content = String::from_utf8_lossy(&object.data).to_string();
                            let input = InternedInput::new("", content.as_str());
                            let diff: String = imara_diff::diff(
                                Algorithm::Histogram,
                                &input,
                                UnifiedDiffBuilder::new(&input),
                            );
                            diff_builder.addition(location, &id, diff);
                        }
                        EntryMode::Link => {}
                        EntryMode::Commit => {}
                    },

                    Event::Deletion { entry_mode, id } => match entry_mode {
                        EntryMode::Tree => {}
                        EntryMode::BlobExecutable | EntryMode::Blob => {
                            let object = repository.find_object(id)?;
                            let content = String::from_utf8_lossy(&object.data).to_string();
                            let input = InternedInput::new(content.as_str(), "");
                            let diff: String = imara_diff::diff(
                                Algorithm::Histogram,
                                &input,
                                UnifiedDiffBuilder::new(&input),
                            );
                            diff_builder.deletion(location, &id, diff);
                        }
                        EntryMode::Link => {}
                        EntryMode::Commit => {}
                    },
                }

                Ok::<Action, object::find::existing::Error>(Action::Continue)
            })?;

        Ok(diff_builder.out)
    }
}

#[cfg(test)]
mod test {
    use crate::repository::GitRepository;
    use anyhow::Result;

    #[test]
    fn test() -> Result<()> {
        let repo = GitRepository {
            inner: git_repository::discover(".")?,
        };

        let diffs = repo.diff("main", "testdiff").unwrap();
        println!("{diffs}");
        Ok(())
    }
}

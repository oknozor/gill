use crate::{ref_to_tree, GitRepository};
use git_repository::bstr::ByteSlice;
use git_repository::object::tree::diff::change::Event;
use git_repository::object::tree::diff::{Action, Change};
use git_repository::objs::tree::EntryMode;
use git_repository::{object, Id};

use imara_diff::intern::InternedInput;
use imara_diff::{Algorithm, UnifiedDiffBuilder};

#[derive(Debug, Default)]
pub struct DiffBuilder {
    out: Vec<Diff>,
}

#[derive(Debug)]
pub enum Diff {
    Addition {
        id: String,
        file_path: String,
        hunk: Option<String>,
    },
    Deletion {
        id: String,
        file_path: String,
        hunk: Option<String>,
    },
    Changes {
        previous_id: String,
        id: String,
        file_path: String,
        hunk: Option<String>,
    },
}

impl Diff {
    pub fn path(&self) -> &str {
        match self {
            Diff::Addition { file_path, .. } => file_path.as_str(),
            Diff::Deletion { file_path, .. } => file_path.as_str(),
            Diff::Changes { file_path, .. } => file_path.as_str(),
        }
    }

    pub fn hunk(&self) -> Option<&str> {
        match self {
            Diff::Addition { hunk, .. } => hunk.as_deref(),
            Diff::Deletion { hunk, .. } => hunk.as_deref(),
            Diff::Changes { hunk, .. } => hunk.as_deref(),
        }
    }
}

impl DiffBuilder {
    fn changed(&mut self, file_path: &str, previous_id: &Id, id: &Id, hunk: Option<String>) {
        let previous_id = previous_id.to_string();
        let id = id.to_string();
        let file_path = file_path.to_owned();

        self.out.push(Diff::Changes {
            previous_id,
            id,
            file_path,
            hunk,
        });
    }

    fn addition(&mut self, file_path: &str, id: &Id, hunk: Option<String>) {
        let id = id.to_string();
        let file_path = file_path.to_owned();
        self.out.push(Diff::Addition {
            id,
            file_path,
            hunk,
        });
    }

    fn deletion(&mut self, file_path: &str, previous_id: &Id, hunk: Option<String>) {
        let id = previous_id.to_string();
        let file_path = file_path.to_owned();
        self.out.push(Diff::Deletion {
            id,
            file_path,
            hunk,
        });
    }
}

impl GitRepository {
    pub fn diff(&self, branch: &str, other: &str) -> anyhow::Result<Vec<Diff>> {
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
                            let data = object.detach().data;
                            let previous_content = String::from_utf8(data).ok();
                            let object = repository.find_object(id)?;
                            let data = object.detach().data;
                            let content = String::from_utf8(data).ok();

                            let diff = previous_content
                                .as_deref()
                                .zip(content.as_deref())
                                .map(|(previous_content, content)| {
                                    InternedInput::new(previous_content, content)
                                })
                                .map(|input| {
                                    imara_diff::diff(
                                        Algorithm::Histogram,
                                        &input,
                                        UnifiedDiffBuilder::new(&input),
                                    )
                                });

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
                            let data = object.detach().data;
                            let content = String::from_utf8(data).ok();
                            let hunk = content
                                .as_deref()
                                .map(|content| InternedInput::new("", content))
                                .map(|input| {
                                    imara_diff::diff(
                                        Algorithm::Histogram,
                                        &input,
                                        UnifiedDiffBuilder::new(&input),
                                    )
                                });
                            diff_builder.addition(location, &id, hunk);
                        }
                        EntryMode::Link => {}
                        EntryMode::Commit => {}
                    },

                    Event::Deletion { entry_mode, id } => match entry_mode {
                        EntryMode::Tree => {}
                        EntryMode::BlobExecutable | EntryMode::Blob => {
                            let object = repository.find_object(id)?;
                            let data = object.detach().data;
                            let content = String::from_utf8(data).ok();
                            let hunk = content
                                .as_deref()
                                .map(|content| InternedInput::new(content, ""))
                                .map(|input| {
                                    imara_diff::diff(
                                        Algorithm::Histogram,
                                        &input,
                                        UnifiedDiffBuilder::new(&input),
                                    )
                                });

                            diff_builder.deletion(location, &id, hunk);
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

        let _diffs = repo.diff("main", "testdiff").unwrap();
        Ok(())
    }
}

use crate::GitRepository;
use git_repository::bstr::ByteSlice;
use git_repository::object::tree::diff::change::Event;
use git_repository::object::tree::diff::{Action, Change};
use git_repository::objs::tree::EntryMode;
use git_repository::{object, Id, Tree};
use imara_diff::intern::InternedInput;
use imara_diff::{Algorithm, UnifiedDiffBuilder};

pub mod commit;
pub mod tree;

#[derive(Debug, Default)]
struct DiffBuilder {
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
    fn diff_tree_to_tree(&self, tree: Tree, other: Tree) -> anyhow::Result<Vec<Diff>> {
        let repository = &self.inner;
        let mut diff_builder = DiffBuilder::default();
        tree.changes()
            .track_path()
            .for_each_to_obtain_tree(&other, |changes: Change| {
                let location = changes.location.to_str().expect("bstr to str");

                match changes.event {
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

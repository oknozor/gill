use crate::{ref_to_tree, GitRepository};

use git_repository::ObjectId;

use crate::commits::OwnedCommit;

use std::path::{Path, PathBuf};

impl GitRepository {
    /// Traverse the whole repository and return a [`TreeMap`].
    pub fn get_tree_for_path(
        &self,
        branch: Option<&str>,
        path: Option<&str>,
    ) -> anyhow::Result<TreeEntry> {
        let reference = branch.map(|name| format!("heads/{name}"));
        let reference = reference.as_deref();
        let tree = ref_to_tree(reference, &self.inner)?;
        let path = path.map(Path::new).unwrap_or(Path::new(""));
        let repo_path = self.inner.path().to_string_lossy();
        let mut delegate = imp::Traversal::new(repo_path.to_string(), path);
        let _ = tree.traverse().breadthfirst(&mut delegate);
        let reference = reference
            .and_then(|reference| self.inner.find_reference(reference).ok())
            .map(|reference| reference.id())
            .unwrap_or(self.inner.head_id().expect("HEAD"));
        let tree = self.attach_commit_to_blob(reference, delegate.out)?;
        Ok(tree)
    }

    /// Returns this blob content
    pub fn blob_str(&self, blob: &BlobInfo) -> anyhow::Result<String> {
        let object = self.inner.find_object(blob.oid)?;
        let content = String::from_utf8_lossy(&object.data);
        Ok(content.to_string())
    }

    /// Returns this blob content
    pub fn blob_bytes(&self, blob: &BlobInfo) -> anyhow::Result<Vec<u8>> {
        let object = self.inner.find_object(blob.oid)?;
        Ok(object.data.clone())
    }

    pub fn blob_mime(&self, blob: &BlobInfo) -> BlobMime {
        let guess = mime_guess::from_path(&blob.path);
        let is_application = guess.iter().any(|mime| mime.type_() == "application");
        let is_image = guess.iter().any(|mime| mime.type_() == "image");

        if is_application {
            BlobMime::Application
        } else if is_image {
            BlobMime::Image
        } else {
            BlobMime::Text
        }
    }
}

/// A Graph representation of a given tree
#[derive(Debug)]
pub struct TreeEntry {
    /// The file name of this tree
    pub filename: String,
    /// blobs in this tree
    pub blobs: Vec<BlobInfo>,
    /// Children trees
    pub trees: Vec<TreeInfo>,
}

/// Wrap a blob filename an provide access to its content
#[derive(Debug)]
struct TraversBlobInfo {
    pub filename: String,
    path: PathBuf,
    oid: ObjectId,
}

/// Wrap a blob filename an provide access to its content
#[derive(Debug)]
pub struct BlobInfo {
    pub filename: String,
    pub commit: OwnedCommit,
    oid: ObjectId,
    path: PathBuf,
}

/// Wrap a tree filename and its corresponding commit
#[derive(Debug)]
pub struct TreeInfo {
    pub name: String,
    pub commit: OwnedCommit,
}

pub enum BlobMime {
    Text,
    Image,
    Application,
}

impl BlobInfo {
    pub fn filename(&self) -> String {
        self.path
            .file_name()
            .expect("filename")
            .to_string_lossy()
            .to_string()
    }
}

mod imp {
    use crate::commits::OwnedCommit;
    use crate::traversal::{BlobInfo, TraversBlobInfo, TreeEntry, TreeInfo};
    use crate::{id_to_commit, GitRepository};

    use git_repository::bstr::{BStr, BString, ByteSlice, ByteVec};
    use git_repository::object::tree::diff;

    use git_repository::objs::tree::EntryRef;
    use git_repository::path::Utf8Error;

    use git_repository::traverse::commit::Sorting;
    use git_repository::traverse::tree::visit::Action;
    use git_repository::traverse::tree::Visit;
    use git_repository::{Id, ObjectId};
    use std::collections::VecDeque;
    use std::fmt;
    use std::fmt::Formatter;
    use std::path::{Path, PathBuf};

    pub(super) struct Traversal<'a> {
        target_reached: bool,
        finished: bool,
        target: &'a Path,
        path_deque: VecDeque<BString>,
        path: BString,
        pub out: IntermediateTreeEntry,
    }

    #[derive(Debug)]
    pub(super) struct IntermediateTreeEntry {
        pub filename: String,
        pub blobs: Vec<TraversBlobInfo>,
        pub trees: Vec<String>,
    }

    impl fmt::Debug for Traversal<'_> {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            writeln!(f, "{:?}", self.path_deque)?;
            write!(f, "{:?}", self.path)
        }
    }

    impl IntermediateTreeEntry {
        fn new(filename: String) -> Self {
            Self {
                filename,
                blobs: vec![],
                trees: Default::default(),
            }
        }
    }

    impl<'a> Traversal<'a> {
        pub fn new(repo_path_relative: String, target: &'a Path) -> Self {
            let target_reached = target == Path::new("");
            Self {
                target_reached,
                finished: false,
                target,
                path_deque: Default::default(),
                path: BString::from(""),
                out: IntermediateTreeEntry::new(repo_path_relative),
            }
        }
    }

    impl Traversal<'_> {
        fn pop_element(&mut self) {
            if let Some(pos) = self.path.rfind_byte(b'/') {
                self.path.resize(pos, 0);
            } else {
                self.path.clear();
            }
        }

        fn push_element(&mut self, name: &BStr) {
            if !self.path.is_empty() {
                self.path.push(b'/');
            }
            self.path.push_str(name);
        }
    }

    impl Visit for Traversal<'_> {
        fn pop_front_tracked_path_and_set_current(&mut self) {
            if self.target_reached {
                self.finished = true;
                return;
            };

            self.path = self
                .path_deque
                .pop_front()
                .expect("every parent is set only once");

            if self.target.ends_with(self.path.to_str_lossy().as_ref()) {
                self.target_reached = true;
            };
        }

        fn push_back_tracked_path_component(&mut self, component: &BStr) {
            self.push_element(component);
            self.path_deque.push_back(self.path.clone());
        }

        fn push_path_component(&mut self, component: &BStr) {
            self.push_element(component);
        }

        fn pop_path_component(&mut self) {
            self.pop_element();
        }

        fn visit_tree(&mut self, entry: &EntryRef<'_>) -> Action {
            if self.finished {
                return Action::Cancel;
            };

            if self.target_reached {
                self.out.trees.push(entry.filename.to_string());
            };

            if self.target.starts_with(self.path.to_str_lossy().as_ref()) {
                Action::Continue
            } else {
                Action::Skip
            }
        }

        fn visit_nontree(&mut self, entry: &EntryRef<'_>) -> Action {
            let path = self.path.to_string();
            if !self.target_reached {
                return Action::Skip;
            };

            let root = self.out.filename.clone();

            self.out.blobs.push(TraversBlobInfo {
                filename: entry.filename.to_string(),
                path: PathBuf::from(root).join(path),
                oid: ObjectId::from(entry.oid),
            });

            Action::Continue
        }
    }

    impl GitRepository {
        pub(super) fn attach_commit_to_blob(
            &self,
            id: Id,
            mut tree_entry: IntermediateTreeEntry,
        ) -> anyhow::Result<TreeEntry> {
            let mut walk = self
                .inner
                .rev_walk(Some(id.detach()))
                .sorting(Sorting::ByCommitTimeNewestFirst)
                .all()?
                .peekable();

            let mut tree_with_data = TreeEntry {
                filename: tree_entry.filename,
                blobs: vec![],
                trees: vec![],
            };

            let mut previous = walk.next().expect("commit")?;
            for current in walk {
                let id = previous;
                let commit = id_to_commit(&id).unwrap();
                let tree = commit.tree().unwrap();
                let current = current.unwrap();
                let next_commit = id_to_commit(&current).unwrap();
                let next_tree = next_commit.tree().unwrap();
                let _ = tree
                    .changes()
                    .track_filename()
                    .track_path()
                    .for_each_to_obtain_tree(&next_tree, |change| {
                        let filename = change.location.to_str_lossy();
                        let filename = filename.as_ref();
                        let idx = tree_entry
                            .blobs
                            .iter()
                            .enumerate()
                            .find(|(_, b)| b.filename == filename)
                            .map(|(i, _)| i);

                        if let Some(idx) = idx {
                            let blob = tree_entry.blobs.remove(idx);
                            let commit = OwnedCommit::try_from(&commit).expect("commit");
                            let blob = BlobInfo {
                                filename: blob.filename,
                                commit,
                                oid: blob.oid,
                                path: blob.path,
                            };

                            tree_with_data.blobs.push(blob);
                        };

                        let idx = tree_entry
                            .trees
                            .iter()
                            .enumerate()
                            .find(|(_, t)| t.as_str() == filename)
                            .map(|(i, _t)| i);

                        if let Some(idx) = idx {
                            let tree = tree_entry.trees.remove(idx);
                            tree_with_data.trees.push(TreeInfo {
                                name: tree,
                                commit: OwnedCommit::try_from(&commit).expect("commit"),
                            });
                        };

                        // We have drained all the original tree and annotated each blob and tree with
                        // their respective commits
                        if tree_entry.blobs.is_empty() && tree_entry.trees.is_empty() {
                            return Ok::<diff::Action, Utf8Error>(diff::Action::Cancel);
                        };

                        Ok::<diff::Action, Utf8Error>(diff::Action::Continue)
                    });
                previous = current;
            }

            // If any blob remains at the end of the rev walk they where added by the first commit
            let first_commit = id_to_commit(&previous)?;
            let first_commit = OwnedCommit::try_from(&first_commit).expect("commit");
            let blobs = tree_entry.blobs.drain(..).map(|blob| BlobInfo {
                filename: blob.filename,
                commit: first_commit.clone(),
                oid: blob.oid,
                path: blob.path,
            });

            let trees = tree_entry.trees.drain(..);

            blobs.for_each(|blob| tree_with_data.blobs.push(blob));
            trees.for_each(|tree| {
                tree_with_data.trees.push(TreeInfo {
                    name: tree,
                    commit: first_commit.clone(),
                })
            });

            Ok(tree_with_data)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::traversal::BlobInfo;
    use crate::GitRepository;
    use speculoos::prelude::*;

    #[test]
    fn should_get_tree() -> anyhow::Result<()> {
        let repo = GitRepository {
            inner: git_repository::discover(".")?,
        };
        let tree = repo.get_tree_for_path(None, Some("crates/gill-git"))?;

        let blobs_in_root: Vec<String> = tree.blobs.iter().map(BlobInfo::filename).collect();

        assert_that!(blobs_in_root).contains_all_of(&[&"Cargo.toml".to_string()]);

        let tree_names: Vec<String> = tree.trees.into_iter().map(|tree| tree.name).collect();
        assert_that!(tree_names).is_equal_to(vec!["src".to_string()]);

        Ok(())
    }

    #[test]
    fn should_get_tree_root() -> anyhow::Result<()> {
        let repo = GitRepository {
            inner: git_repository::discover(".")?,
        };

        let tree = repo.get_tree_for_path(Some("main"), None)?;
        println!("{:#?}", tree.blobs);
        println!("{:#?}", tree.trees);
        let mut blobs_in_root: Vec<String> = tree.blobs.iter().map(BlobInfo::filename).collect();
        blobs_in_root.sort();

        assert_that!(blobs_in_root).is_equal_to(vec![
            ".env".to_string(),
            ".gitattributes".to_string(),
            ".gitignore".to_string(),
            ".gitmodules".to_string(),
            "Cargo.toml".to_string(),
            "Cross.toml".to_string(),
            "Dockerfile".to_string(),
            "README.md".to_string(),
            "docker-compose.yml".to_string(),
            "justfile".to_string(),
            "syntect".to_string(),
        ]);

        let tree_names: Vec<String> = tree.trees.into_iter().map(|tree| tree.name).collect();

        assert_that!(tree_names).is_equal_to(vec!["crates".to_string(), "docker".to_string()]);

        Ok(())
    }
}

use crate::{ref_to_tree, GitRepository};

use git_repository::ObjectId;

use std::collections::HashMap;
use std::path::PathBuf;

impl GitRepository {
    /// Traverse the whole repository and return a [`TreeMap`].
    pub fn get_tree_for_path(
        &self,
        branch: Option<&str>,
        path: Option<&str>,
    ) -> anyhow::Result<TreeMap> {
        let reference = branch.map(|name| format!("heads/{name}"));
        let reference = reference.as_deref();
        let tree = ref_to_tree(reference, &self.inner)?;
        let repo_path = self.inner.path().to_string_lossy();
        let mut delegate = imp::Traversal::new(repo_path.to_string());
        tree.traverse().breadthfirst(&mut delegate).unwrap();

        Ok(match path {
            None => delegate.tree_root,
            Some(path) => delegate.tree_root.get_tree(path)?,
        })
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
pub struct TreeMap {
    /// The file name of this tree
    pub filename: String,
    /// blobs in this tree
    pub blobs: Vec<BlobInfo>,
    /// Children trees
    pub trees: HashMap<String, TreeMap>,
}

/// Wrap a blob filename an provide access to its content
#[derive(Debug)]
pub struct BlobInfo {
    pub filename: String,
    path: PathBuf,
    oid: ObjectId,
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
    use crate::traversal::{BlobInfo, TreeMap};
    use anyhow::anyhow;
    use git_repository::bstr::{BStr, BString, ByteSlice, ByteVec};
    use git_repository::objs::tree::EntryRef;
    use git_repository::traverse::tree::visit::Action;
    use git_repository::traverse::tree::Visit;
    use git_repository::ObjectId;
    use std::collections::VecDeque;
    use std::fmt;
    use std::fmt::Formatter;
    use std::path::PathBuf;

    pub struct Traversal {
        path_deque: VecDeque<BString>,
        path: BString,
        pub tree_root: TreeMap,
    }

    impl fmt::Debug for Traversal {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            writeln!(f, "{:?}", self.path_deque)?;
            write!(f, "{:?}", self.path)
        }
    }

    impl TreeMap {
        fn new(filename: String) -> Self {
            Self {
                filename,
                blobs: vec![],
                trees: Default::default(),
            }
        }

        pub fn get_tree(self, tree_path: &str) -> anyhow::Result<Self> {
            let mut tree = self;
            let parts = tree_path.split('/');
            for path in parts {
                tree = tree
                    .trees
                    .remove(path)
                    .ok_or_else(|| anyhow!("Failed to find tree {tree_path}"))?
            }

            Ok(tree)
        }

        fn populate_tree(&mut self, tree_path: &str) -> &mut Self {
            match self.trees.get(tree_path) {
                None => {
                    self.trees
                        .insert(tree_path.to_string(), TreeMap::new(tree_path.to_string()));
                    self.trees.get_mut(tree_path).unwrap()
                }
                Some(_) => self.trees.get_mut(tree_path).unwrap(),
            }
        }
    }

    impl Traversal {
        pub fn new(repo_path_relative: String) -> Self {
            Self {
                path_deque: Default::default(),
                path: BString::from(""),
                tree_root: TreeMap::new(repo_path_relative),
            }
        }
    }

    impl Traversal {
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

    impl Visit for Traversal {
        fn pop_front_tracked_path_and_set_current(&mut self) {
            self.path = self
                .path_deque
                .pop_front()
                .expect("every parent is set only once");
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

        fn visit_tree(&mut self, _entry: &EntryRef<'_>) -> Action {
            let path = self.path.to_string();
            let parts = path.split('/');
            let mut current = &mut self.tree_root;
            for tree_path in parts {
                current = current.populate_tree(tree_path);
            }

            Action::Continue
        }

        fn visit_nontree(&mut self, entry: &EntryRef<'_>) -> Action {
            let root = self.tree_root.filename.clone();
            let path = self.path.to_string();
            let mut parts = path.split('/').peekable();
            let mut current = &mut self.tree_root;

            while let Some(tree_path) = parts.next() {
                if parts.peek().is_none() {
                    break;
                }
                current = current.populate_tree(tree_path);
            }

            let filename = entry.filename;

            current.blobs.push(BlobInfo {
                filename: filename.to_string(),
                path: PathBuf::from(root).join(path),
                oid: ObjectId::from(entry.oid),
            });

            Action::Continue
        }
    }
}

#[cfg(test)]
mod test {
    use crate::traversal::BlobInfo;
    use crate::GitRepository;
    use speculoos::prelude::*;

    #[test]
    fn should_get_tree_root() -> anyhow::Result<()> {
        let repo = GitRepository {
            inner: git_repository::discover(".")?,
        };
        let tree = repo.get_tree_for_path(None, None)?;
        let crates = tree.trees.get("crates").unwrap();

        let blobs_in_root: Vec<String> = tree.blobs.iter().map(BlobInfo::filename).collect();

        assert_that!(blobs_in_root).contains_all_of(&[
            &"Cargo.toml".to_string(),
            &"docker-compose.yml".to_string(),
            &"Dockerfile".to_string(),
            &"justfile".to_string(),
            &"README.md".to_string(),
        ]);

        assert_that!(crates.trees.keys()).contains_all_of(&[
            &"gill-app".to_string(),
            &"gill-git".to_string(),
            &"gill-git-server".to_string(),
        ]);

        Ok(())
    }

    #[test]
    fn should_get_tree() -> anyhow::Result<()> {
        let repo = GitRepository {
            inner: git_repository::discover(".")?,
        };
        let tree = repo.get_tree_for_path(Some("main"), Some("crates/gill-git"))?;

        let blobs_in_root: Vec<String> = tree.blobs.iter().map(BlobInfo::filename).collect();

        assert_that!(blobs_in_root).contains_all_of(&[&"Cargo.toml".to_string()]);

        assert_that!(tree.trees.keys()).contains_all_of(&[&"src".to_string()]);

        Ok(())
    }
}

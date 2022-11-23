use crate::traversal::imp::ref_to_tree;
use git_repository::bstr::BString;
use git_repository::ObjectId;
use std::collections::HashMap;
use std::path::Path;

/// Traverse the whole repository and return a [`TreeMap`].
pub fn get_tree_for_path<P: AsRef<Path>>(
    repo: P,
    branch: Option<&str>,
    path: Option<&str>,
) -> eyre::Result<TreeMap> {
    let repo = git_repository::open(repo.as_ref())?;
    let reference = branch.map(|name| format!("heads/{name}"));
    let reference = reference.as_deref();
    let tree = ref_to_tree(reference, &repo)?;
    let mut delegate = imp::Traversal::default();
    tree.traverse().breadthfirst(&mut delegate).unwrap();

    Ok(match path {
        None => delegate.tree_root,
        Some(path) => delegate.tree_root.get_tree(path)?,
    })
}

/// A Graph representation of a given tree
#[derive(Default, Debug)]
pub struct TreeMap {
    /// The file name of this treeM
    pub filename: String,
    /// blobs in this tree
    pub blobs: Vec<BlobInfo>,
    /// Children trees
    pub trees: HashMap<String, TreeMap>,
}

/// Wrap a blob filename an provide access to its content
#[derive(Debug)]
pub struct BlobInfo {
    pub filename: BString,
    oid: ObjectId,
}

impl BlobInfo {
    /// Returns this blob content
    pub fn content<P: AsRef<Path>>(&self, repo_path: P) -> eyre::Result<String> {
        let repo = git_repository::open(repo_path.as_ref())?;
        let object = repo.find_object(self.oid)?;
        let content = String::from_utf8_lossy(&object.data);
        Ok(content.to_string())
    }
}

mod imp {
    use crate::traversal::{BlobInfo, TreeMap};
    use eyre::eyre;
    use git_repository::bstr::{BStr, BString, ByteSlice, ByteVec};
    use git_repository::objs::tree::EntryRef;
    use git_repository::traverse::tree::visit::Action;
    use git_repository::traverse::tree::Visit;
    use git_repository::{ObjectId, Tree};
    use std::collections::VecDeque;
    use std::fmt;
    use std::fmt::Formatter;

    pub fn ref_to_tree<'repo>(
        reference: Option<&str>,
        repo: &'repo git_repository::Repository,
    ) -> eyre::Result<Tree<'repo>> {
        Ok(match reference {
            Some(reference) => repo
                .find_reference(reference)?
                .peel_to_id_in_place()?
                .object()?
                .try_into_commit()?
                .tree()?,
            None => repo.head()?.peel_to_commit_in_place()?.tree()?,
        })
    }

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

        pub fn get_tree(self, tree_path: &str) -> eyre::Result<Self> {
            let mut tree = self;
            let parts = tree_path.split('/');
            for path in parts {
                tree = tree
                    .trees
                    .remove(path)
                    .ok_or(eyre!("Failed to find tree {tree_path}"))?
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

    impl Default for Traversal {
        fn default() -> Self {
            Self {
                path_deque: Default::default(),
                path: BString::from(""),
                tree_root: Default::default(),
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
            let path = self.path.to_string();
            let mut parts = path.split('/').peekable();
            let mut current = &mut self.tree_root;

            while let Some(tree_path) = parts.next() {
                if parts.peek().is_none() {
                    break;
                }
                current = current.populate_tree(tree_path);
            }

            current.blobs.push(BlobInfo {
                filename: entry.filename.into(),
                oid: ObjectId::from(entry.oid),
            });

            Action::Continue
        }
    }
}

#[cfg(test)]
mod test {
    use crate::traversal::get_tree_for_path;
    use speculoos::prelude::*;

    #[test]
    fn should_get_tree_root() -> eyre::Result<()> {
        let repo = git_repository::discover(".")?;
        let tree = get_tree_for_path(repo.path(), None, None)?;
        let crates = tree.trees.get("crates").unwrap();

        let blobs_in_root: Vec<String> = tree
            .blobs
            .iter()
            .map(|blob| blob.filename.to_string())
            .collect();

        assert_that!(blobs_in_root).contains_all_of(&[
            &"Cargo.toml".to_string(),
            &"docker-compose.yml".to_string(),
            &"Dockerfile".to_string(),
            &"justfile".to_string(),
            &"README.md".to_string(),
        ]);

        assert_that!(crates.trees.keys()).contains_all_of(&[
            &"ruisseau-api".to_string(),
            &"ruisseau-git".to_string(),
            &"ruisseau-git-server".to_string(),
        ]);

        Ok(())
    }

    #[test]
    fn should_get_tree() -> eyre::Result<()> {
        let repo = git_repository::discover(".")?;
        let tree = get_tree_for_path(repo.path(), Some("main"), Some("crates/ruisseau-git"))?;

        let blobs_in_root: Vec<String> = tree
            .blobs
            .iter()
            .map(|blob| blob.filename.to_string())
            .collect();

        assert_that!(blobs_in_root).contains_all_of(&[&"Cargo.toml".to_string()]);

        assert_that!(tree.trees.keys()).contains_all_of(&[&"src".to_string()]);

        Ok(())
    }
}

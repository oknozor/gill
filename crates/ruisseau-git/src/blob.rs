use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use git_hash::ObjectId;
use git_object::Kind::Tree;
use git_object::Object::Blob;
use git_object::tree::EntryMode;
use git_odb::pack::FindExt;
use git_repository::config::Path;
use git_traverse::tree;
use git_traverse::tree::Recorder;

pub struct TreeInfo {
    path: String,
    children: Vec<TreeInfo>,
    blobs: Vec<BlobInfo>
}

pub struct BlobInfo {
    filename: String,
    data: Vec<u8>,
}

fn read_blob() -> Result<(), Box<dyn Error>> {
    let repo = git_repository::open("/home/okno/Code/ruisseau").unwrap();
    let mut buf = Vec::new();
    let mut buf2 = Vec::new();
    let db = repo.objects;
    let mut commit = db.find_commit_iter(hex_to_id("99d939d0a8e411d93399f0044d9b2145cccb07f3"), &mut buf)?.0;
    let mut recorder = tree::Recorder::default();
    git_traverse::tree::breadthfirst(
        db.find_tree_iter(commit.tree_id().expect("a tree is available in a commit"), &mut buf2)?
            .0,
        tree::breadthfirst::State::default(),
        |oid, buf| db.find_tree_iter(oid, buf).ok().map(|t| t.0),
        &mut recorder,
    )?;

    let mut tree = TreeInfo {
        path: Default::default(),
        children: vec![],
        blobs: vec![],
    };

    for record in recorder.records {
        let mut path = PathBuf::from(record.filepath.to_string());
        match record.mode {
            EntryMode::Tree => {
                if path.parent().is_none() {
                    tree.children.push(TreeInfo {
                        path: path.file_name().unwrap().to_string_lossy().to_string(),
                        children: vec![],
                        blobs: vec![]
                    })
                } else {
                    path.components()
                    for part in path.iter() {
                        pa
                    }
                }
            }
            EntryMode::Blob => {
                println!("Blob {}", record.filepath);
                let repo = git_repository::open("/home/okno/Code/ruisseau").unwrap();
                let data = &repo.find_object(record.oid).unwrap();
                let data = data.data.to_owned();
                let filename = path.file_name().unwrap().to_string_lossy().to_string();

                match &mut path.parent() {
                    None => tree.blobs.push(BlobInfo {
                        filename,
                        data
                    }),
                    Some(parent) => {
                        path.pop();
                        for path in path.iter() {
                            tree.children.iter_mut()
                                .find(|tree| tree.path)
                        }
                        for path in parent.parent()
                    }
                }

                let content = String::from_utf8_lossy(&data);

            }
            EntryMode::BlobExecutable => {
                println!("BlobEx {}", record.filepath);
            }
            EntryMode::Link => {
                println!("Link {}", record.filepath);
            }
            EntryMode::Commit => {
                println!("Commit {}", record.filepath);
            }
        }
    }

    Ok(())
}

fn hex_to_id(hex: &str) -> ObjectId {
    git_hash::ObjectId::from_hex(hex.as_bytes()).expect("40 bytes hex")
}

#[test]
fn test() {
    read_blob().unwrap();
}
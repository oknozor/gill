use crate::diff::{Line, LineChange, LineDiff};
use crate::repository::ref_to_tree;
use git_repository::object::tree::diff::change::Event;
use git_repository::object::tree::diff::{Action, Change};
use git_repository::objs::tree::EntryMode;
use git_repository::{object, Repository};

use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Diff<'a> {
    pub file: String,
    pub diff: LineDiff<'a>,
}

pub fn diff_it(repo: &Repository) -> anyhow::Result<()> {
    let _diff = diff(repo)?;
    Ok(())
}

/// Write git-diff like output for debugging purpose, we shall remove this later and build HTML instead
pub fn diff_to_output(diff: LineDiff, out: &mut impl Write) -> anyhow::Result<()> {
    let changes = &diff.changes;
    for change in changes {
        match change {
            LineChange::Equal { new: _, old, len } => {
                for line in *old..(*old + *len) {
                    let line = diff.get_old_line(line).unwrap();
                    out.write_all(line)?;
                    out.write_all(b"\n")?;
                }
            }

            LineChange::Deletion { old, new: _, len } => {
                for deleted in *old..(*old + *len) {
                    let deleted = diff.get_old_line(deleted).unwrap();
                    write_deleted(out, deleted)?;
                }
            }
            LineChange::Replace {
                old,
                old_len,
                new,
                new_len,
            } => {
                for deleted in *old..(*old + *old_len) {
                    let deleted = diff.get_old_line(deleted).unwrap();
                    write_deleted(out, deleted)?;
                }

                for inserted in *new..(*new + *new_len) {
                    let inserted = diff.get_new_line(inserted).unwrap();
                    write_inserted(out, inserted)?;
                }
            }

            LineChange::Insertion { old: _, new, new_len } => {
                for inserted in *new..(*new + *new_len) {
                    let inserted = diff.get_new_line(inserted).unwrap();
                    write_inserted(out, inserted)?;
                }
            }
        }
    }

    Ok(())
}

fn write_inserted(out: &mut (impl Write + Sized), inserted: &[u8]) -> io::Result<()> {
    out.write_all(b"\x1b[92;5;57m")?;
    out.write_all(b"++ ")?;
    out.write_all(inserted)?;
    out.write_all(b"\n")?;
    out.write_all(b"\x1b[92;5;m")?;
    Ok(())
}

fn write_deleted(out: &mut (impl Write + Sized), deleted: &[u8]) -> io::Result<()> {
    out.write_all(b"\x1b[91;5;57m")?;
    out.write_all(b"-- ")?;
    out.write_all(deleted)?;
    out.write_all(b"\n")?;
    out.write_all(b"\x1b[91;5;m")?;
    Ok(())
}

// TODO:
pub fn diff<'a>(repository: &'a Repository) -> anyhow::Result<String> {
    let tree = ref_to_tree(Some("heads/main"), repository)?;
    let other = ref_to_tree(Some("heads/testdiff"), repository)?;
    let mut out = vec![];
    tree.changes()
        .track_path()
        .for_each_to_obtain_tree(&other, |changes: Change| {
            match changes.event {
                // TODO: get modification mix and match then convert to htlm with modification line
                //  marked. We probably want to take a look at git delta to do this (they use syntect too)
                Event::Modification {
                    previous_entry_mode,
                    previous_id,
                    entry_mode,
                    id,
                } => match (previous_entry_mode, entry_mode) {
                    (EntryMode::Blob, EntryMode::Blob) => {
                        let object = repository.find_object(previous_id)?;
                        // Todo: Any chance we could avoid allocation here ?
                        //      Maybe we need to collect all git objects in a upper structure and handle the writing there ?
                        let previous_content = String::from_utf8_lossy(&object.data).to_string();
                        let object = repository.find_object(id)?;
                        let content = String::from_utf8_lossy(&object.data).to_string();

                        let old_line: Vec<Line> = previous_content
                            .lines()
                            .map(|line| Line {
                                data: line.as_bytes(),
                            })
                            .collect();

                        let new_lines: Vec<Line> = content
                            .lines()
                            .map(|line| Line {
                                data: line.as_bytes(),
                            })
                            .collect();

                        let diff = LineDiff::default()
                            .old(&old_line)
                            .new(&new_lines)
                            .diff()
                            .unwrap();

                        out.write_all(changes.location).unwrap();
                        out.write_all(b"\n").unwrap();
                        println!("{}", changes.location);
                        diff_to_output(diff, &mut out).unwrap();
                        out.write_all(b"\n").unwrap();
                    }
                    _ => {
                        // TODO: Ignoring everything but blob to blob diff for now
                    }
                },
                _ => {
                    // TODO: Handle all possible event
                }
            }
            Ok::<Action, object::find::existing::Error>(Action::Continue)
        })?;

    Ok(String::from_utf8_lossy(&out).to_string())
}

#[cfg(test)]
mod test {
    use crate::repository::diff::diff;
    use anyhow::Result;

    #[test]
    fn test() -> Result<()> {
        let repo = git_repository::discover(".")?;
        let vec = diff(&repo).unwrap();
        println!("{}", vec);
        Ok(())
    }
}

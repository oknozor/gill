use crate::diff::{Line, LineDiff};
use crate::repository::ref_to_tree;
use git_repository::object::tree::diff::change::Event;
use git_repository::object::tree::diff::{Action, Change};
use git_repository::objs::tree::EntryMode;
use git_repository::{object, Repository};
use std::fmt::Debug;

use crate::repository::diff::example::TerminalDiff;


use std::io::Write;

pub trait DiffWriter
where
    Self: Write + Debug,
{
    type Error: Debug;

    fn write_location(&mut self, location: String) -> Result<(), Self::Error>;
    fn write_diff(&mut self, diff: LineDiff) -> Result<(), Self::Error>;
    fn write_addition(&mut self, content: String) -> Result<(), Self::Error>;
    fn write_deletion(&mut self, content: String) -> Result<(), Self::Error>;
    fn finish_file(&mut self) -> Result<(), Self::Error>;
    fn finish(&mut self) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub struct Diff<'a> {
    pub file: String,
    pub diff: LineDiff<'a>,
}

pub fn diff_it(repo: &Repository) -> anyhow::Result<()> {
    diff(repo, "main", "testdiff", &mut TerminalDiff::new())?;
    Ok(())
}

pub fn diff<'a, T>(
    repository: &'a Repository,
    branch: &str,
    other: &str,
    writer: &mut T,
) -> anyhow::Result<Vec<Diff<'a>>>
where
    T: DiffWriter,
{
    let tree = ref_to_tree(Some(&format!("heads/{branch}")), repository)?;
    let other = ref_to_tree(Some(&format!("heads/{other}")), repository)?;
    let diffs = vec![];
    tree.changes()
        .track_path()
        .for_each_to_obtain_tree(&other, |changes: Change| {
            writer
                .write_location(changes.location.to_string())
                .expect("Failed to write diff");
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

                        writer.write_diff(diff).expect("Failed to write diff");
                    }
                    _ => {
                        // TODO: Ignoring everything but blob to blob diff for now
                    }
                },
                Event::Addition { entry_mode, id } => {
                    match entry_mode {
                        EntryMode::Tree => {}
                        EntryMode::Blob | EntryMode::BlobExecutable => {
                            let object = repository.find_object(id)?;
                            let content = String::from_utf8_lossy(&object.data).to_string();
                            writer.write_addition(content).expect("Write failure");
                        }
                        EntryMode::Link => {}
                        EntryMode::Commit => {}
                    }
                    // TODO: Handle all possible event
                }

                Event::Deletion { entry_mode, id } => match entry_mode {
                    EntryMode::Tree => {}
                    EntryMode::BlobExecutable | EntryMode::Blob => {
                        let object = repository.find_object(id)?;
                        let content = String::from_utf8_lossy(&object.data).to_string();
                        writer.write_deletion(content).expect("Write failure");
                    }
                    EntryMode::Link => {}
                    EntryMode::Commit => {}
                },
            }
            writer.finish_file().expect("Write failure");
            Ok::<Action, object::find::existing::Error>(Action::Continue)
        })?;

    writer.finish().expect("Write failure");
    Ok(diffs)
}

mod example {
    use crate::diff::{DiffChange, LineDiff, TextDiff};
    use crate::repository::diff::{DiffWriter};
    use std::io;
    use std::io::{stdout, Stdout, Write};

    #[derive(Debug)]
    pub struct TerminalDiff {
        out: Stdout,
    }

    impl TerminalDiff {
        pub fn new() -> Self {
            Self { out: stdout() }
        }
    }

    impl Write for TerminalDiff {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.out.write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            self.out.flush()
        }
    }

    impl DiffWriter for TerminalDiff {
        type Error = io::Error;

        fn write_location(&mut self, location: String) -> Result<(), Self::Error> {
            self.write_all(location.as_bytes())?;
            self.write_all(b"\n")
        }

        fn write_diff(&mut self, diff: LineDiff) -> Result<(), Self::Error> {
            self.diff_to_stdout(diff)?;
            Ok(())
        }

        fn write_addition(&mut self, content: String) -> Result<(), Self::Error> {
            self.write_all(b"\x1b[92;5;57m")?;
            for line in content.lines() {
                self.write_all(format!("+ {}", line).as_bytes())?;
                self.write_all(b"\n")?;
            }
            self.write_all(b"\x1b[92;5;m")?;
            Ok(())
        }

        fn write_deletion(&mut self, content: String) -> Result<(), Self::Error> {
            self.write_all(b"\x1b[91;5;57m")?;
            for line in content.lines() {
                self.write_all(format!("- {}", line).as_bytes())?;
                self.write_all(b"\n")?;
            }
            self.write_all(b"\x1b[91;5;m")?;
            Ok(())
        }

        fn finish_file(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn finish(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    impl TerminalDiff {
        /// Write git-diff like output for debugging purpose, we shall remove this later and build HTML instead
        pub fn diff_to_stdout(&mut self, diff: LineDiff) -> io::Result<()> {
            let changes = &diff.changes;
            for change in changes {
                match change {
                    DiffChange::Equal { new: _, old, len } => {
                        for line in *old..(*old + *len) {
                            let line = diff.get_old_line(line).unwrap();
                            self.write_all(line)?;
                            self.write_all(b"\n")?;
                        }
                    }

                    DiffChange::Deletion { old, new: _, len } => {
                        for deleted in *old..(*old + *len) {
                            let deleted = diff.get_old_line(deleted).unwrap();
                            self.write_deleted(deleted)?;
                        }
                    }
                    DiffChange::Replace {
                        old,
                        old_len,
                        new,
                        new_len,
                    } => {
                        let mut old_out = vec![];
                        let mut new_out = vec![];
                        if old_len > new_len {
                            for idx in *new..(*new + *new_len) {
                                let inserted = diff.get_new_line(idx).unwrap();
                                let deleted = diff.get_old_line(idx).unwrap();
                                let diff = TextDiff::default().diff(deleted, inserted).unwrap();

                                if diff.changes.len() < 3 {
                                    Self::write_replaced(
                                        &mut old_out,
                                        &mut new_out,
                                        inserted,
                                        deleted,
                                        diff.changes,
                                    )?;
                                } else {
                                    self.write_deleted(deleted)?;
                                    self.write_inserted(inserted)?;
                                }
                            }

                            for deleted in *old + *new_len..(*old + *old_len) {
                                let deleted = diff.get_old_line(deleted).unwrap();
                                self.write_deleted(deleted)?;
                            }
                        } else {
                            for idx in *old..(*old + *old_len) {
                                let inserted = diff.get_new_line(idx).unwrap();
                                let deleted = diff.get_old_line(idx).unwrap();
                                let diff = TextDiff::default().diff(deleted, inserted).unwrap();

                                if diff.changes.len() < 3 {
                                    Self::write_replaced(
                                        &mut old_out,
                                        &mut new_out,
                                        inserted,
                                        deleted,
                                        diff.changes,
                                    )?;
                                } else {
                                    self.write_deleted(deleted)?;
                                    self.write_inserted(inserted)?;
                                }
                            }

                            for inserted in *new + *old_len..(*new + *new_len) {
                                let inserted = diff.get_new_line(inserted).unwrap();
                                self.write_inserted(inserted)?;
                            }
                        }
                        self.write_all(&old_out)?;
                        self.write_all(&new_out)?;
                    }

                    DiffChange::Insertion {
                        old: _,
                        new,
                        new_len,
                    } => {
                        for inserted in *new..(*new + *new_len) {
                            let inserted = diff.get_new_line(inserted).unwrap();
                            self.write_inserted(inserted)?;
                        }
                    }
                }
            }

            Ok(())
        }

        fn write_replaced(
            out_old: &mut (impl Write + Sized),
            out_new: &mut (impl Write + Sized),
            inserted: &[u8],
            deleted: &[u8],
            changes: Vec<DiffChange>,
        ) -> io::Result<()> {
            for change in changes {
                match change {
                    DiffChange::Equal { old: _, new, len } => {
                        out_old.write_all(&deleted[new..new + len])?;
                        out_old.write_all(b"\n")?;
                    }
                    DiffChange::Deletion { old, new: _, len } => {
                        out_old.write_all(b"\x1b[91;5;57m")?;
                        out_old.write_all(b"-- ")?;
                        out_old.write_all(&deleted[old..old + len])?;
                        out_old.write_all(b"\n")?;
                        out_old.write_all(b"\x1b[91;5;m")?;
                    }
                    DiffChange::Insertion { old: _, new, new_len } => {
                        out_new.write_all(b"\x1b[92;5;57m")?;
                        out_new.write_all(b"++ ")?;
                        out_new.write_all(&inserted[new..new + new_len])?;
                        out_new.write_all(b"\n")?;
                        out_new.write_all(b"\x1b[92;5;m")?;
                    }
                    DiffChange::Replace {
                        old,
                        old_len,
                        new,
                        new_len,
                    } => {
                        out_old.write_all(b"\x1b[41;5;57m")?;
                        out_old.write_all(&deleted[old..old + old_len])?;
                        out_old.write_all(b"\x1b[41;5;m")?;
                        out_old.write_all(b"\n")?;

                        out_new.write_all(b"\x1b[42;5;57m")?;
                        out_new.write_all(&inserted[new..new + new_len])?;
                        out_new.write_all(b"\x1b[42;5;m")?;
                        out_new.write_all(b"\n")?;
                    }
                }
            }
            Ok(())
        }

        fn write_inserted(&mut self, inserted: &[u8]) -> io::Result<()> {
            self.write_all(b"\x1b[92;5;57m")?;
            self.write_all(b"++ ")?;
            self.write_all(inserted)?;
            self.write_all(b"\n")?;
            self.write_all(b"\x1b[92;5;m")?;
            Ok(())
        }

        fn write_deleted(&mut self, deleted: &[u8]) -> io::Result<()> {
            self.write_all(b"\x1b[91;5;57m")?;
            self.write_all(b"-- ")?;
            self.write_all(deleted)?;
            self.write_all(b"\n")?;
            self.write_all(b"\x1b[91;5;m")?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::repository::diff::example::TerminalDiff;
    use crate::repository::diff::{diff};
    use anyhow::Result;

    #[test]
    fn test() -> Result<()> {
        let repo = git_repository::discover(".")?;
        let _diffs = diff(&repo, "main", "testdiff", &mut TerminalDiff::new()).unwrap();
        let _writer = TerminalDiff::new();
        Ok(())
    }
}

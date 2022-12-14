use diffs::{Diff, Replace};
use std::fmt;
use std::fmt::Formatter;

#[derive(Default)]
pub struct LineDiff<'a> {
    pub changes: Vec<DiffChange>,
    old: &'a [Line<'a>],
    new: &'a [Line<'a>],
}

impl fmt::Debug for LineDiff<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "LineDiff {{ changes: {:?}, old_len: {}, new_len {} }}",
            self.changes,
            self.old.len(),
            self.new.len()
        )
    }
}

#[derive(Debug, Default)]
pub struct TextDiff {
    pub changes: Vec<DiffChange>,
}

impl LineDiff<'_> {
    pub fn get_old_line(&self, idx: usize) -> Option<&[u8]> {
        self.old.get(idx).map(|line| line.data)
    }

    pub fn get_new_line(&self, idx: usize) -> Option<&[u8]> {
        self.new.get(idx).map(|line| line.data)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Line<'a> {
    pub data: &'a [u8],
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiffChange {
    Equal {
        old: usize,
        new: usize,
        len: usize,
    },
    Deletion {
        old: usize,
        new: usize,
        len: usize,
    },
    Insertion {
        old: usize,
        new: usize,
        new_len: usize,
    },
    Replace {
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    },
}

impl TextDiff {
    pub fn diff(self, old: &[u8], new: &[u8]) -> anyhow::Result<Self> {
        let mut diff = Replace::new(self);
        let _ = diffs::myers::diff(&mut diff, old, 0, old.len(), new, 0, new.len());
        Ok(diff.into_inner())
    }
}

impl Diff for TextDiff {
    type Error = ();

    fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), Self::Error> {
        self.changes.push(DiffChange::Equal { old, len, new });
        Ok(())
    }

    fn delete(&mut self, old: usize, len: usize, new: usize) -> Result<(), Self::Error> {
        self.changes.push(DiffChange::Deletion { old, len, new });
        Ok(())
    }

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        self.changes
            .push(DiffChange::Insertion { old, new_len, new });
        Ok(())
    }

    fn replace(
        &mut self,
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    ) -> Result<(), Self::Error> {
        self.changes.push(DiffChange::Replace {
            old,
            old_len,
            new,
            new_len,
        });

        Ok(())
    }
}

impl<'a> LineDiff<'a> {
    pub fn old(mut self, old: &'a [Line]) -> Self {
        self.old = old;
        self
    }

    pub fn new(mut self, new: &'a [Line]) -> Self {
        self.new = new;
        self
    }

    pub fn diff(self) -> anyhow::Result<Self> {
        let old = self.old.to_vec();
        let new = self.new.to_vec();
        let old_len = self.old.len();
        let new_len = self.new.len();
        let mut diff = Replace::new(self);
        let _ = diffs::myers::diff(&mut diff, &old, 0, old_len, &new, 0, new_len);

        Ok(diff.into_inner())
    }
}

impl Diff for LineDiff<'_> {
    type Error = ();

    fn equal(&mut self, old: usize, new: usize, len: usize) -> Result<(), Self::Error> {
        self.changes.push(DiffChange::Equal { old, len, new });
        Ok(())
    }

    fn delete(&mut self, old: usize, len: usize, new: usize) -> Result<(), Self::Error> {
        self.changes.push(DiffChange::Deletion { old, len, new });
        Ok(())
    }

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        self.changes
            .push(DiffChange::Insertion { old, new_len, new });
        Ok(())
    }

    fn replace(
        &mut self,
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    ) -> Result<(), Self::Error> {
        self.changes.push(DiffChange::Replace {
            old,
            new_len,
            new,
            old_len,
        });

        Ok(())
    }
}

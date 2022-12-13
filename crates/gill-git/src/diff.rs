use diffs::{Diff, Replace};


#[derive(Debug, Default)]
pub struct LineDiff<'a> {
    pub changes: Vec<LineChange>,
    old: &'a [Line<'a>],
    new: &'a [Line<'a>],
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
pub enum LineChange {
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

impl LineChange {
    fn old(&self) -> usize {
        match self {
            LineChange::Equal { old, .. }
            | LineChange::Insertion { old, .. }
            | LineChange::Replace { old, .. }
            | LineChange::Deletion { old, .. } => *old,
        }
    }

    fn new(&self) -> usize {
        match self {
            LineChange::Equal { new, .. }
            | LineChange::Insertion { new, .. }
            | LineChange::Replace { new, .. }
            | LineChange::Deletion { new, .. } => *new,
        }
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
        self.changes.push(LineChange::Equal { old, len, new });
        Ok(())
    }

    fn delete(&mut self, old: usize, len: usize, new: usize) -> Result<(), Self::Error> {
        self.changes.push(LineChange::Deletion { old, len, new });
        Ok(())
    }

    fn insert(&mut self, old: usize, new: usize, new_len: usize) -> Result<(), Self::Error> {
        self.changes
            .push(LineChange::Insertion { old, new_len, new });
        Ok(())
    }

    fn replace(
        &mut self,
        old: usize,
        old_len: usize,
        new: usize,
        new_len: usize,
    ) -> Result<(), Self::Error> {
        self.changes.push(LineChange::Replace {
            old,
            new_len,
            new,
            old_len,
        });

        Ok(())
    }
}

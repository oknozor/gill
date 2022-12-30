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
    fn addition(&mut self, file_path: &str, id: &Id, hunk: Option<String>) {
        let id = id.to_string();
        let file_path = file_path.to_owned();
        self.out.push(Diff::Addition {
            id,
            file_path,
            hunk,
        });
    fn deletion(&mut self, file_path: &str, previous_id: &Id, hunk: Option<String>) {
        let id = previous_id.to_string();
        let file_path = file_path.to_owned();
        self.out.push(Diff::Deletion {
            id,
            file_path,
            hunk,
        });
    pub fn diff(&self, branch: &str, other: &str) -> anyhow::Result<Vec<Diff>> {
                            let data = object.detach().data;
                            let previous_content = String::from_utf8(data).ok();
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
        let _diffs = repo.diff("main", "testdiff").unwrap();
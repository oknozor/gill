use crate::SYNTAX_SET;
use std::fmt::Write;
use syntect::easy::HighlightLines;
use syntect::html::{append_highlighted_html_for_styled_line, IncludeBackground};

#[derive(Debug, PartialEq)]
pub enum State<'a> {
    Head,
    Diff {
        deletion_start: u32,
        addition_start: u32,
        deletion_nth: u32,
        addition_nth: u32,
        path: &'a str,
        first_line: Option<&'a str>,
    },
}

enum LineType {
    Add,
    Del,
    Unchanged,
}

pub fn diff2html(diff: &str) -> Result<String, syntect::Error> {
    let mut state = State::Head;
    let mut lines = diff.lines().peekable();
    let mut out = String::new();
    while let Some(line) = lines.next() {
        if line == "\\ No newline at end of file" {
            continue;
        }

        match state {
            State::Head => {
                println!("HEAD {}", line);
                let diff_line = line.strip_prefix("diff --git a/").unwrap();
                let (_previous_path, path) = diff_line.split_once(" b/").unwrap();
                let line = lines.peek().unwrap();
                // For now we just skip it but we could speed up the
                // parser by driving it to only iter through addition/deletion
                // depending on the diff mode
                if line.starts_with("deleted file") || line.starts_with("new file") {
                    let _deleted_or_new = lines.next();
                };

                let _index = lines.next();
                let _a = lines.next();
                let _b = lines.next();

                let line_hunk = lines.next().unwrap();
                println!("INFO {}", line_hunk);
                state = parse_line_info(path, line_hunk);
                out.push_str(r#"<div class="d">"#);
                out.push_str(&format!(
                    r#"<div class="h"><i class="ti ti-file-code-2 px-4"></i>{path}</div>"#
                ));
                out.push_str("<table>");
            }
            State::Diff {
                deletion_start,
                addition_start,
                deletion_nth,
                addition_nth,
                path,
                first_line,
            } => {
                let (_, ext) = path.rsplit_once('.').unzip();
                let mut highligher = ext.and_then(super::highlighter_for_extension);
                let mut addition_count = 0;
                let mut deletion_count = 0;

                let line_type = LineType::from_line(line);
                let start_line = match line_type {
                    LineType::Add => addition_start,
                    LineType::Del => deletion_start,
                    LineType::Unchanged => addition_start,
                };

                match highligher.as_mut() {
                    None => {
                        if let Some(first_line) = first_line {
                            line_to_html(first_line, start_line, &mut out, &LineType::Unchanged)
                        }
                        line_to_html(line, start_line + 1, &mut out, &line_type)
                    }
                    Some(highlighter) => {
                        if let Some(first_line) = first_line {
                            line_to_html_highlighted(
                                first_line,
                                start_line + 1,
                                &LineType::Unchanged,
                                &mut out,
                                highlighter,
                            )?
                        }
                        line_to_html_highlighted(
                            line,
                            start_line,
                            &line_type,
                            &mut out,
                            highlighter,
                        )?
                    }
                };

                update_line_counters(&mut addition_count, &mut deletion_count, line_type);
                while addition_count < addition_nth || deletion_count < deletion_nth {
                    let line = lines.next().unwrap();
                    let line_type = LineType::from_line(line);
                    let line_number = match line_type {
                        LineType::Add => addition_start + addition_count,
                        LineType::Del => deletion_start + deletion_count,
                        LineType::Unchanged => addition_start + addition_count,
                    };

                    match highligher.as_mut() {
                        None => line_to_html(line, line_number, &mut out, &line_type),
                        Some(highlighter) => line_to_html_highlighted(
                            line,
                            line_number,
                            &line_type,
                            &mut out,
                            highlighter,
                        )?,
                    };

                    update_line_counters(&mut addition_count, &mut deletion_count, line_type);
                }

                let line = lines.peek();

                if let Some(line) = line {
                    if line.starts_with("@@") {
                        let diff_state = parse_line_info(path, line);
                        state = diff_state;
                        lines.next();
                        continue;
                    }
                }
                state = State::Head;
                out.push_str("</table>");
                out.push_str("</div>");
            }
        }
    }

    Ok(out)
}

impl LineType {
    fn get_tailwind_bg(&self) -> Option<&'static str> {
        match self {
            LineType::Add => Some("bg-green-400"),
            LineType::Del => Some("bg-red-400"),
            LineType::Unchanged => None,
        }
    }
}

fn line_to_html_highlighted(
    line: &str,
    line_number: u32,
    line_type: &LineType,
    output: &mut String,
    highlighter: &mut HighlightLines,
) -> Result<(), syntect::Error> {
    match line_type.get_tailwind_bg() {
        None => output.push_str(&format!(r#"<tr id="L{line_number}">"#)),
        Some(bg) => output.push_str(&format!(r#"<tr class="{bg}" id="L{line_number}">"#)),
    };

    output.push_str(&format!(r#"<td class="l">{line_number}</td>"#));
    output.push_str(r#"<td class="c">"#);
    let regions = highlighter.highlight_line(line, &SYNTAX_SET)?;
    append_highlighted_html_for_styled_line(&regions[..], IncludeBackground::No, output)?;
    output.push_str("</td>");
    output.push_str("</tr>");

    Ok(())
}

fn line_to_html(line: &str, line_number: u32, output: &mut String, line_type: &LineType) {
    match line_type.get_tailwind_bg() {
        None => output.push_str(&format!(r#"<tr id="L{line_number}">"#)),
        Some(bg) => output.push_str(&format!(r#"<tr class="{bg}" id="L{line_number}">"#)),
    };

    output.push_str(&format!(r#"<td class="l"">{line_number}</td>"#));
    output.push_str(r#"<td class="c">"#);
    output.push_str(line);
    output.push_str("</td>");
    output.push_str("</tr>");
}

fn update_line_counters(addition_count: &mut u32, deletion_count: &mut u32, line_type: LineType) {
    match line_type {
        LineType::Add => *addition_count += 1,
        LineType::Del => *deletion_count += 1,
        LineType::Unchanged => {
            *addition_count += 1;
            *deletion_count += 1;
        }
    }
}

fn parse_line_info<'a>(path: &'a str, line: &'a str) -> State<'a> {
    let line = line.strip_prefix("@@").unwrap();
    let (line_numbers, content) = line.split_once("@@").unwrap();
    let (deletion, addition) = line_numbers.trim().split_once(' ').unwrap();
    let addition = addition.strip_prefix('+').unwrap();
    let (addition_start, addition_nth) = extract_line_start_and_nth(addition);
    let deletion = deletion.strip_prefix('-').unwrap();
    let (deletion_start, deletion_nth) = extract_line_start_and_nth(deletion);
    let content = content.trim();
    let first_line = (!content.is_empty()).then_some(content);
    State::Diff {
        deletion_start,
        addition_start,
        deletion_nth,
        path,
        addition_nth,
        first_line,
    }
}

impl LineType {
    pub fn from_line(line: &str) -> Self {
        if let Some(_line) = line.strip_prefix('-') {
            LineType::Del
        } else if let Some(_line) = line.strip_prefix('+') {
            LineType::Add
        } else {
            LineType::Unchanged
        }
    }
}

fn extract_line_start_and_nth(line_info: &str) -> (u32, u32) {
    line_info
        .split_once(',')
        .map(|(start, nth)| (start.parse().unwrap(), nth.parse().unwrap()))
        .unwrap_or_else(|| (line_info.parse::<u32>().unwrap(), 0))
}

#[cfg(test)]
mod test {

    const DIFF: &str = r#"diff --git a/crates/gill-app/src/syntax/mod.rs b/crates/gill-app/src/syntax/mod.rs
index d1f7f0f..300870e 100644
--- a/crates/gill-app/src/syntax/mod.rs
+++ b/crates/gill-app/src/syntax/mod.rs
@@ -1 +1,2 @@
 pub mod highlight;
+pub mod diff;
\ No newline at end of file
diff --git a/crates/gill-db/migrations/20221115110623_base_schema.sql b/crates/gill-db/migrations/20221115110623_base_schema.sql
index 1251133..6c801b8 100644
--- a/crates/gill-db/migrations/20221115110623_base_schema.sql
+++ b/crates/gill-db/migrations/20221115110623_base_schema.sql
@@ -55,7 +55,6 @@ CREATE TABLE repository
     send_patches_to   VARCHAR(255) NOT NULL,
     domain            VARCHAR(255) NOT NULL,
     is_local          BOOLEAN      NOT NULL,
-    item_count        INT          NOT NULL DEFAULT 0,
     CONSTRAINT Unique_Name_For_Repository UNIQUE (name, attributed_to)
 );

diff --git a/crates/gill-db/src/repository.rs b/crates/gill-db/src/repository.rs
index 279ba5f..621c93f 100644
--- a/crates/gill-db/src/repository.rs
+++ b/crates/gill-db/src/repository.rs
@@ -210,6 +210,11 @@ impl Repository {
         .ok()
     }

+    pub async fn create_branch(&self, branch_name: &str, db: &PgPool) -> sqlx::Result<()> {
+        Branch::create(branch_name, self.id, false, db).await?;
+        Ok(())
+    }
+
     pub async fn by_activity_pub_id(
         activity_pub_id: &str,
         pool: &PgPool,
diff --git a/crates/gill-git-server/src/post-receive.rs b/crates/gill-git-server/src/post-receive.rs
index aa0bec7..1dd11a3 100644
--- a/crates/gill-git-server/src/post-receive.rs
+++ b/crates/gill-git-server/src/post-receive.rs
@@ -1,4 +1,4 @@
-use gill_db::repository::Repository;
+use gill_db::repository::{Branch, Repository};
 use gill_db::PgPoolOptions;
 use gill_settings::SETTINGS;
 use std::env;
@@ -55,14 +55,22 @@ async fn main() -> anyhow::Result<()> {
     match git_ref.strip_prefix("refs/heads/") {
         Some(branch) => {
             let repo = Repository::by_namespace(&repository_owner, repository_name, &db).await?;
-            if repo.get_default_branch(&db).await.is_none() {
+            let branches = repo.list_branches(i64::MAX, 0, &db).await?;
+            let branches: Vec<&str> = branches
+                .iter()
+                .map(|branch| branch.name.as_str())
+                .collect();
+
+            if branches.is_empty() {
                 repo.set_default_branch(branch, &db).await?;
                 writeln!(
                     log_file,
                     "default branch {branch} set for {repository_owner}/{repository_name}"
                 )?;
+            } else if !branches.contains(&branch) {
+                repo.create_branch(branch, &db).await?;
             } else {
-                writeln!(log_file, "already have a default branch")?;
+                writeln!(log_file, "existing branch")?;
             }
         }
         None => writeln!(log_file, "branch not found")?,
diff --git a/crates/gill-git/src/repository/diff.rs b/crates/gill-git/src/repository/diff.rs
index 89b1cf1..3570e28 100644
--- a/crates/gill-git/src/repository/diff.rs
+++ b/crates/gill-git/src/repository/diff.rs
@@ -8,11 +8,6 @@ use git_repository::{object, Id};
 use imara_diff::intern::InternedInput;
 use imara_diff::{Algorithm, UnifiedDiffBuilder};

-pub fn diff_it(repo: &GitRepository) -> anyhow::Result<()> {
-    repo.diff("main", "testdiff")?;
-    Ok(())
-}
-
 #[derive(Debug, Default)]
 pub struct DiffBuilder {
     out: String,
diff --git a/docker/sshd_config b/docker/sshd_config
index 01f17da..506d0bc 100644
--- a/docker/sshd_config
+++ b/docker/sshd_config
@@ -1,4 +1,4 @@
-AuthorizedKeysFile	.ssh/authorized_keys
+AuthorizedKeysFile .ssh/authorized_keys
 PasswordAuthentication no
 Subsystem sftp /usr/lib/ssh/sftp-server
 AcceptEnv GIT_PROTOCOL
\ No newline at end of file"#;

    const DELETION: &str = r#"diff --git a/website/themes/adidoks/content/privacy-policy/_index.md b/website/themes/adidoks/content/privacy-policy/_index.md
deleted file mode 100644
index d8050da..0000000
--- a/website/themes/adidoks/content/privacy-policy/_index.md
+++ /dev/null
@@ -1,27 +0,0 @@
-+++
-title = "Privacy Policy"
-description = "We do not use cookies and we do not collect any personal data."
-date = 2021-05-01T08:00:00+00:00
-updated = 2020-05-01T08:00:00+00:00
-draft = false
-
-[extra]
-class = "page single"
-+++
-
-__TLDR__: We do not use cookies and we do not collect any personal data.
-
-## Website visitors
-
-- No personal information is collected.
-- No information is stored in the browser.
-- No information is shared with, sent to or sold to third-parties.
-- No information is shared with advertising companies.
-- No information is mined and harvested for personal and behavioral trends.
-- No information is monetized.
-
-## Contact us
-
-[Contact us](https://github.com/aaranxu/adidoks) if you have any questions.
-
-Effective Date: _1st May 2021_"#;

    const NEW_FILE: &str = r#"diff --git a/.github/dependabot.yaml b/.github/dependabot.yaml
new file mode 100644
index 0000000..c8ecc6a
--- /dev/null
+++ b/.github/dependabot.yaml
@@ -0,0 +1,8 @@
+version: 2
+updates:
+  - package-ecosystem: cargo
+    directory: /
+    schedule:
+      interval: daily
+      time: "14:05"
+    open-pull-requests-limit: 10
\ No newline at end of file"#;

    #[test]
    fn diff2html() {
        let diffs = super::diff2html(DIFF);
        assert!(diffs.is_ok());
    }

    #[test]
    fn diff2html_deletion_only() {
        let diffs = super::diff2html(DELETION);
        assert!(diffs.is_ok());
    }

    #[test]
    fn diff2html_new_file_only() {
        let diffs = super::diff2html(NEW_FILE);
        assert!(diffs.is_ok());
    }
}

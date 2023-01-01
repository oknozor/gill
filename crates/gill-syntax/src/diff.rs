use crate::SYNTAX_SET;
use gill_git::diff::Diff;
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

pub fn diff2html(diffs: &[Diff]) -> Result<String, syntect::Error> {
    let mut out = String::new();
    for diff in diffs {
        let path = diff.path();
        out.push_str(r#"<div class="d">"#);
        out.push_str(&format!(
            r#"<div class="h"><i class="ti ti-file-code-2 px-4"></i>{path}</div>"#
        ));
        out.push_str("<table>");

        match diff {
            Diff::Addition { .. } => generate_html_for_addition(&mut out, diff)?,
            Diff::Deletion { .. } => generate_html_for_deletion(&mut out, diff)?,
            Diff::Changes { .. } => generate_html_for_change(&mut out, diff)?,
        }

        out.push_str("</table>");
        out.push_str("</div>");
    }
    Ok(out)
}

fn generate_html_for_addition(out: &mut String, diff: &Diff) -> Result<(), syntect::Error> {
    let (_, ext) = diff.path().rsplit_once('.').unzip();

    let Some(hunk) = diff.hunk() else {
        return Ok(());
    };

    let mut highligher = ext.and_then(super::highlighter_for_extension);
    let mut lines = hunk.lines();
    let _ = lines.next();

    for (idx, line) in lines.enumerate() {
        match highligher.as_mut() {
            None => line_to_html(line, idx as u32 + 1, out, &LineType::Add),

            Some(hl) => line_to_html_highlighted(line, idx as u32 + 1, &LineType::Add, out, hl)?,
        }
    }

    Ok(())
}

fn generate_html_for_deletion(out: &mut String, diff: &Diff) -> Result<(), syntect::Error> {
    let (_, ext) = diff.path().rsplit_once('.').unzip();

    let Some(hunk) = diff.hunk() else {
        return Ok(());
    };

    let mut highligher = ext.and_then(super::highlighter_for_extension);
    let mut lines = hunk.lines();
    let _ = lines.next();

    for (idx, line) in lines.enumerate() {
        match highligher.as_mut() {
            None => line_to_html(line, idx as u32 + 1, out, &LineType::Del),

            Some(hl) => line_to_html_highlighted(line, idx as u32 + 1, &LineType::Del, out, hl)?,
        }
    }

    Ok(())
}

fn generate_html_for_change(out: &mut String, diff: &Diff) -> Result<(), syntect::Error> {
    let mut state = State::Head;

    let path = diff.path();
    let hunk = diff.hunk();
    if let Some(hunk) = hunk {
        let mut lines = hunk.lines().peekable();
        while let Some(line) = lines.next() {
            if line == "\\ No newline at end of file" {
                continue;
            }

            match state {
                State::Head => state = parse_line_info(path, line),
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
                                line_to_html(first_line, start_line, out, &LineType::Unchanged)
                            }
                            line_to_html(line, start_line + 1, out, &line_type)
                        }
                        Some(highlighter) => {
                            if let Some(first_line) = first_line {
                                line_to_html_highlighted(
                                    first_line,
                                    start_line + 1,
                                    &LineType::Unchanged,
                                    out,
                                    highlighter,
                                )?
                            }
                            line_to_html_highlighted(
                                line,
                                start_line,
                                &line_type,
                                out,
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
                            None => line_to_html(line, line_number, out, &line_type),
                            Some(highlighter) => line_to_html_highlighted(
                                line,
                                line_number,
                                &line_type,
                                out,
                                highlighter,
                            )?,
                        };

                        update_line_counters(&mut addition_count, &mut deletion_count, line_type);
                    }

                    let line = lines.peek();

                    if let Some(line) = line {
                        if line.starts_with("@@") {
                            state = parse_line_info(path, line);
                            lines.next();
                            continue;
                        }
                    }
                }
            }
        }
    }

    Ok(())
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
    use gill_git::diff::Diff;

    #[test]
    fn diff2html() {
        let diffs = vec![Diff::Changes {
            previous_id: "1234".to_string(),
            id: "1234".to_string(),
            file_path: "toto.rs".to_string(),
            hunk: Some(
                r#"@@ -1 +1,2 @@
 pub mod highlight;
+pub mod diff;
\ No newline at end of file"#
                    .to_string(),
            ),
        }];
        let diffs = super::diff2html(&diffs);
        assert!(diffs.is_ok());
    }

    #[test]
    fn diff2html_multiple_hunks() {
        let diffs = vec![Diff::Changes {
            previous_id: "1234".to_string(),
            id: "1234".to_string(),
            file_path: "toto.rs".to_string(),
            hunk: Some(
                r#"@@ -1,4 +1,4 @@
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
         None => writeln!(log_file, "branch not found")?,"#
                    .to_string(),
            ),
        }];
        let diffs = super::diff2html(&diffs);
        assert!(diffs.is_ok());
    }
}

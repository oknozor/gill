

use gill_git::diff::{DiffChange, LineDiff};
use gill_git::repository::diff::DiffWriter;
use std::io::Write;

use std::string::FromUtf8Error;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme};
use syntect::html::{append_highlighted_html_for_styled_line, IncludeBackground};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;


#[derive(Debug)]
pub struct HtmlDiff {
    out: Vec<u8>,
    syntax_set: SyntaxSet,
    theme: Theme,
    extension: Option<String>,
}

impl HtmlDiff {
    pub fn new(set: SyntaxSet, theme: Theme) -> Self {
        HtmlDiff {
            out: vec![],
            syntax_set: set,
            theme,
            extension: None,
        }
    }

    pub fn get_html(self) -> Result<String, FromUtf8Error> {
        let content = String::from_utf8(self.out)?;
        Ok(content)
    }
}

impl Write for HtmlDiff {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.out.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}

impl DiffWriter for HtmlDiff {
    type Error = anyhow::Error;

    fn write_location(&mut self, location: String) -> Result<(), Self::Error> {
        let extension = location.rsplit('.').collect::<Vec<&str>>();
        let extension = extension.first();
        if let Some(extension) = extension {
            self.extension = Some(extension.to_string());
        }

        let location =
            format!(r#"<span class="text-lg font-bold underline py-2">{location}</span>"#);
        self.write_all(location.as_bytes())?;
        self.write_all(br#"<div class="flex flex-col whitespace-pre-wrap border-2 px-2 blob-code rounded-md">"#)?;
        self.write_all(b"<table>")?;
        self.write_all(b"<tbody>")?;
        Ok(())
    }

    fn write_diff(&mut self, diff: LineDiff) -> Result<(), Self::Error> {
        let syntax_set = self.syntax_set.clone();
        let syntax: Option<&SyntaxReference> = self
            .extension
            .as_ref()
            .and_then(|extension| syntax_set.find_syntax_by_extension(extension));
        let theme = self.theme.clone();

        for change in &diff.changes {
            match change {
                DiffChange::Equal { old, new: _, len } => {
                    for line in *old..*old + *len {
                        let data = diff.get_old_line(line).unwrap();
                        self.highlighted_html_for_line(
                            data,
                            line,
                            syntax,
                            &syntax_set,
                            &theme,
                            Style::Unchanged,
                        )
                        .unwrap();
                    }
                }
                DiffChange::Deletion { old, new: _, len } => {
                    for line in *old..*old + *len {
                        let data = diff.get_old_line(line).unwrap();
                        self.highlighted_html_for_line(
                            data,
                            line,
                            syntax,
                            &syntax_set,
                            &theme,
                            Style::Deleted,
                        )
                        .unwrap();
                    }
                }
                DiffChange::Insertion { old: _, new, new_len } => {
                    for line in *new..*new + *new_len {
                        let data = diff.get_new_line(line).unwrap();
                        self.highlighted_html_for_line(
                            data,
                            line,
                            syntax,
                            &syntax_set,
                            &theme,
                            Style::Added,
                        )
                        .unwrap();
                    }
                }
                DiffChange::Replace {
                    old,
                    old_len,
                    new,
                    new_len,
                } => {
                    if old + old_len > new + new_len {
                        for idx in *new..(*new + *new_len) {
                            let inserted = diff.get_new_line(idx).unwrap();
                            let deleted = diff.get_old_line(idx).unwrap();

                            self.highlighted_html_for_line(
                                deleted,
                                idx,
                                syntax,
                                &syntax_set,
                                &theme,
                                Style::Deleted,
                            )
                            .unwrap();

                            self.highlighted_html_for_line(
                                inserted,
                                idx,
                                syntax,
                                &syntax_set,
                                &theme,
                                Style::Added,
                            )
                            .unwrap();
                        }

                        for idx in old + new_len..(old + old_len) {
                            let deleted = diff.get_old_line(idx).unwrap();
                            self.highlighted_html_for_line(
                                deleted,
                                idx,
                                syntax,
                                &syntax_set,
                                &theme,
                                Style::Deleted,
                            )
                            .unwrap();
                        }
                    } else {
                        for idx in *old..(*old + *old_len) {
                            let inserted = diff.get_new_line(idx).unwrap();
                            let deleted = diff.get_old_line(idx).unwrap();

                            self.highlighted_html_for_line(
                                deleted,
                                idx,
                                syntax,
                                &syntax_set,
                                &theme,
                                Style::Deleted,
                            )
                            .unwrap();

                            self.highlighted_html_for_line(
                                inserted,
                                idx,
                                syntax,
                                &syntax_set,
                                &theme,
                                Style::Added,
                            )
                            .unwrap();
                        }

                        for line in *new + *old_len..(*new + *new_len) {
                            let inserted = diff.get_new_line(line).unwrap();
                            self.highlighted_html_for_line(
                                inserted,
                                line,
                                syntax,
                                &syntax_set,
                                &theme,
                                Style::Added,
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn write_addition(&mut self, content: String) -> Result<(), Self::Error> {
        let syntax_set = self.syntax_set.clone();
        let syntax: Option<&SyntaxReference> = self
            .extension
            .as_ref()
            .and_then(|extension| syntax_set.find_syntax_by_extension(extension));
        let theme = self.theme.clone();
        self.highlighted_html_for_file(content, &syntax_set, syntax, &theme, FileStyle::Added)
            .expect("Write failure");
        Ok(())
    }

    fn write_deletion(&mut self, content: String) -> Result<(), Self::Error> {
        let syntax_set = self.syntax_set.clone();
        let syntax: Option<&SyntaxReference> = self
            .extension
            .as_ref()
            .and_then(|extension| syntax_set.find_syntax_by_extension(extension));
        let theme = self.theme.clone();

        self.highlighted_html_for_file(content, &syntax_set, syntax, &theme, FileStyle::Deleted)
            .expect("Write failure");
        Ok(())
    }

    fn finish_file(&mut self) -> Result<(), Self::Error> {
        self.extension = None;
        self.write_all(b"</tbody>")?;
        self.write_all(b"</table>")?;
        self.write_all(b"</div>")?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

enum Style {
    Added,
    Deleted,
    Unchanged,
}

enum FileStyle {
    Added,
    Deleted,
}

impl HtmlDiff {
    fn highlighted_html_for_line(
        &mut self,
        line: &[u8],
        line_number: usize,
        syntax: Option<&SyntaxReference>,
        syntax_set: &SyntaxSet,
        theme: &Theme,
        style: Style,
    ) -> Result<(), syntect::Error> {
        let bg = match style {
            Style::Added => "bg-green-300",
            Style::Deleted => "bg-red-300",
            Style::Unchanged => "",
        };
        let line = String::from_utf8_lossy(line);
        self.write_all(format!(r#"<tr class="flex" id="line-{line_number}">"#).as_bytes())?;
        self.write_all(
            format!(r#"<td class="px-3 grow-0 bg-zinc-300">{line_number}</td>"#).as_bytes(),
        )?;
        self.write_all(format!(r#"<td class="grow {bg}">"#).as_bytes())?;
        match syntax {
            Some(syntax) => {
                let mut highlighter = HighlightLines::new(syntax, theme);
                let regions = highlighter.highlight_line(line.as_ref(), syntax_set)?;
                let mut hl = String::new();
                append_highlighted_html_for_styled_line(
                    &regions[..],
                    IncludeBackground::No,
                    &mut hl,
                )?;
                self.write_all(hl.as_bytes())?;
            }
            None => self.write_all(line.as_bytes())?,
        }

        self.write_all(b"</td>")?;
        self.write_all(b"</tr>")?;
        Ok(())
    }

    fn highlighted_html_for_file(
        &mut self,
        content: String,
        syntax_set: &SyntaxSet,
        syntax: Option<&SyntaxReference>,
        theme: &Theme,
        style: FileStyle,
    ) -> Result<(), syntect::Error> {
        let bg = match style {
            FileStyle::Added => "bg-green-300",
            FileStyle::Deleted => "bg-red-300",
        };

        match syntax {
            None => {
                for (idx, line) in content.lines().enumerate() {
                    let line_number = idx;
                    self.write_all(
                        format!(r#"<tr class="flex" id="line-{line_number}">"#).as_bytes(),
                    )?;
                    self.write_all(
                        format!(r#"<td class="px-3 grow-0 bg-zinc-300">{line_number}</td>"#)
                            .as_bytes(),
                    )?;
                    self.write_all(format!(r#"<td class="grow {bg}">"#).as_bytes())?;
                    self.write_all(line.as_bytes())?;
                    self.write_all(b"</td>")?;
                }
            }
            Some(syntax) => {
                let mut highlighter = HighlightLines::new(syntax, theme);
                for (idx, line) in LinesWithEndings::from(&content).enumerate() {
                    let line_number = idx + 1;
                    self.write_all(
                        format!(r#"<tr class="flex" id="line-{line_number}">"#).as_bytes(),
                    )?;
                    self.write_all(
                        format!(r#"<td class="px-3 grow-0 bg-zinc-300">{line_number}</td>"#)
                            .as_bytes(),
                    )?;
                    self.write_all(format!(r#"<td class="grow {bg}">"#).as_bytes())?;
                    let regions = highlighter.highlight_line(line, syntax_set)?;
                    let mut hl = String::new();
                    append_highlighted_html_for_styled_line(
                        &regions[..],
                        IncludeBackground::No,
                        &mut hl,
                    )?;
                    self.write_all(hl.as_bytes())?;
                }
            }
        }
        self.write_all(b"</td>")?;
        self.write_all(b"</tr>")?;
        Ok(())
    }
}

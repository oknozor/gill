use anyhow::anyhow;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Color, Theme};

use syntect::html::{append_highlighted_html_for_styled_line, IncludeBackground};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

pub fn highlight_blob(
    blob_content: &str,
    extension: &str,
    syntax_set: SyntaxSet,
    theme: &Theme,
) -> anyhow::Result<String> {
    let syntax = syntax_set
        .find_syntax_by_extension(extension)
        .ok_or_else(|| anyhow!("Syntax not found for extension {extension}"))?;

    let html = highlighted_html_for_string(blob_content, &syntax_set, syntax, theme)?;
    Ok(html)
}

fn highlighted_html_for_string(
    content: &str,
    syntax_set: &SyntaxSet,
    syntax: &SyntaxReference,
    theme: &Theme,
) -> Result<String, syntect::Error> {
    let mut highlighter = HighlightLines::new(syntax, theme);
    let (mut output, bg) = start_highlighted_html(theme);
    output.push_str("<tbody>");
    for (idx, line) in LinesWithEndings::from(content).enumerate() {
        let line_number = idx + 1;
        output.push_str(&format!(r#"<tr id="line-{line_number}">"#));
        output.push_str(&format!(
            r#"<td class="px-3 bg-zinc-200">{line_number}</td>"#
        ));
        output.push_str("<td>");
        let regions = highlighter.highlight_line(line, syntax_set)?;
        append_highlighted_html_for_styled_line(
            &regions[..],
            IncludeBackground::IfDifferent(bg),
            &mut output,
        )?;
        output.push_str("<td/>");
    }
    output.push_str("</tbody>");
    output.push_str("</table>");
    Ok(output)
}

pub fn start_highlighted_html(t: &Theme) -> (String, Color) {
    let c = t.settings.background.unwrap_or(Color::WHITE);
    (
        format!(
            "<table style=\"background-color:#{:02x}{:02x}{:02x};\">\n",
            c.r, c.g, c.b
        ),
        c,
    )
}

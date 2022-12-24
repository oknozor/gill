use anyhow::anyhow;

use syntect::highlighting::{Color, Theme};

use crate::{highlighter_for_extension, SYNTAX_SET, THEME};
use syntect::html::{append_highlighted_html_for_styled_line, IncludeBackground};

use syntect::util::LinesWithEndings;

pub fn highlight_blob(content: &str, extension: &str) -> anyhow::Result<String> {
    let mut highlighter = highlighter_for_extension(extension)
        .ok_or_else(|| anyhow!("syntax set not found for extension: {extension}"))?;

    let (mut output, bg) = start_highlighted_html(&THEME);
    output.push_str("<tbody>");
    for (idx, line) in LinesWithEndings::from(content).enumerate() {
        let line_number = idx + 1;
        output.push_str(&format!(r#"<tr id="line-{line_number}">"#));
        output.push_str(&format!(
            r#"<td class="px-3 bg-zinc-200">{line_number}</td>"#
        ));
        output.push_str("<td>");
        let regions = highlighter.highlight_line(line, &SYNTAX_SET)?;
        append_highlighted_html_for_styled_line(
            &regions[..],
            IncludeBackground::IfDifferent(bg),
            &mut output,
        )?;
        output.push_str("</td>");
        output.push_str("</tr>");
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

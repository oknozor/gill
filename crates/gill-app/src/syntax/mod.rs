use once_cell::sync::{Lazy};
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::{SyntaxSet};

pub mod diff;
pub mod highlight;

const SYNTAX_SET_DATA: &[u8] = include_bytes!("syntax.bin");
const THEME_DATA: &[u8] = include_bytes!("theme.bin");

pub static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(|| syntect::dumps::from_binary(SYNTAX_SET_DATA));
pub static THEME: Lazy<Theme> = Lazy::new(|| syntect::dumps::from_binary(THEME_DATA));

pub fn highlighter_for_extension(extension: &str) -> Option<HighlightLines> {
    SYNTAX_SET
        .find_syntax_by_extension(extension)
        .map(|syntax| HighlightLines::new(syntax, &THEME))
}

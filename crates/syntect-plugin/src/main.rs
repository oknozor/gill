use anyhow::Result;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSetBuilder;

fn main()  -> Result<()> {
    dump_theme("theme.bin")?;
    dump_syntax("syntax.bin")?;
    Ok(())
}

pub fn dump_theme(out: &str) -> Result<()> {
    let mut themes = ThemeSet::new();

    // FIXME, we don't have a theme anymore
    themes
        .add_from_folder("syntect/default_theme.tmTheme")
        .expect("Failed to load syntect theme");

    let theme = themes
        .themes
        .get("default_theme")
        .expect("Default theme missing");

    syntect::dumps::dump_to_file(&theme, out)?;
    Ok(())
}

pub fn dump_syntax(out: &str) -> Result<()> {
    let mut syntax_definitions = SyntaxSetBuilder::new();
    syntax_definitions
        .add_from_folder("syntect", false)
        .expect("Failed to load syntax definitions");

    let set = syntax_definitions.build();
    syntect::dumps::dump_to_file(&set, out)?;
    Ok(())
}


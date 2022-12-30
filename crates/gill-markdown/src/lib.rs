use std::io::Cursor;

use pulldown_cmark::{html, Options, Parser};
use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesStart, Event as HtmlEvent};
use quick_xml::name::QName;
use quick_xml::reader::Reader;
use quick_xml::Writer;

pub fn render(markdown_input: &str, owner: &str, repository: &str) -> String {
    let parser = Parser::new_ext(markdown_input, Options::all());
    let mut out = String::new();
    html::push_html(&mut out, parser);
    let out = render_html(&out, owner, repository).expect("Valid html");
    out
}

fn render_html(
    html_input: &str,
    owner: &str,
    repository: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(html_input);
    reader.expand_empty_elements(true);
    reader.trim_text(true);
    reader.check_end_names(false);
    let mut out = vec![];
    let mut writer = Writer::new(Cursor::new(&mut out));

    loop {
        match reader.read_event() {
            Ok(HtmlEvent::Start(img)) if img.name().as_ref() == b"img" => {
                match img.try_get_attribute("src")? {
                    Some(src) => {
                        let link = &src.value;
                        let link = String::from_utf8_lossy(link);
                        let link = link.as_ref();
                        if is_relative(link) {
                            let link = prepend_namespace(link, owner, repository);
                            let src = Attribute::from(("src", link.as_str()));
                            let mut new_img = BytesStart::new("img");
                            let attributes: Vec<Attribute> = img
                                .attributes()
                                .map(|attr| attr.expect("Valid attribute"))
                                .filter(|attr| attr.key != QName(b"src"))
                                .collect();

                            new_img.extend_attributes(attributes);
                            new_img.push_attribute(src);
                            writer.write_event(HtmlEvent::Start(new_img))?
                        } else {
                            writer.write_event(HtmlEvent::Start(img))?
                        }
                    }
                    None => writer.write_event(HtmlEvent::Start(img))?,
                };
            }
            Ok(HtmlEvent::Eof) => break,
            Ok(e) => writer.write_event(e)?,
            Err(_err) => {
                // Silently ignored
            }
        }
    }
    Ok(String::from_utf8_lossy(&out).to_string())
}

fn is_relative(link: &str) -> bool {
    if link.starts_with("https://") || link.starts_with("http://") {
        return false;
    };

    true
}

fn prepend_namespace(link: &str, owner: &str, repository: &str) -> String {
    if link.starts_with('/') {
        format!("/{owner}/{repository}{link}")
    } else if let Some(link) = link.strip_prefix("./") {
        format!("/{owner}/{repository}/{link}")
    } else {
        format!("/{owner}/{repository}/{link}")
    }
}

#[cfg(test)]
mod test {
    use crate::render;
    use speculoos::prelude::*;

    #[test]
    fn should_canonicalize_image_link() {
        let markdown = r#"<img src="/docs/assets/img.png" alt="image" />"#;
        let html = render(markdown, "oknozor", "gill");
        assert_that!(html).is_equal_to(
            &r#"<img alt="image" src="/oknozor/gill/docs/assets/img.png"></img>"#.to_owned(),
        );
    }

    #[test]
    fn should_canonicalize_image_link_relative() {
        let markdown = r#"<img src="./docs/assets/img.png" alt="image" />"#;
        let html = render(markdown, "oknozor", "gill");
        assert_that!(html).is_equal_to(
            &r#"<img alt="image" src="/oknozor/gill/docs/assets/img.png"></img>"#.to_owned(),
        );
    }

    #[test]
    fn should_canonicalize_image_link_no_slash() {
        let markdown = r#"<img src="docs/assets/img.png" alt="image" />"#;
        let html = render(markdown, "oknozor", "gill");
        assert_that!(html).is_equal_to(
            &r#"<img alt="image" src="/oknozor/gill/docs/assets/img.png"></img>"#.to_owned(),
        );
    }
}

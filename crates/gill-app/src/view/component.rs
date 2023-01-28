use askama::Template;

#[derive(Template, Debug)]
#[template(path = "components/markdown-preview-form.html", ext = "html")]
pub struct MarkdownPreviewForm {
    pub with_title: bool,
    pub action_href: String,
    pub submit_value: String,
    pub owner: String,
    pub repository: String,
}

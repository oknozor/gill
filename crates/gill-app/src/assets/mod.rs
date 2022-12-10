use axum::Router;
use axum::routing::get;

pub fn router() -> Router {
    Router::new()
        .route("/css/tabler-icons.css", get(|| async { include_str!("../../assets/css/tabler-icons.css") }))
        .route("/css/markdown.css", get(|| async { include_str!("../../assets/css/markdown.css") }))
        .route("/style.css", get(|| async { include_str!("../../assets/css/style.css") }))
        .route("/css/fonts/tabler-icons.woff2", get(|| async { include_bytes!("../../assets/css/fonts/tabler-icons.woff2").to_vec() }))
        .route("/css/tailwind.css", get(|| async { include_str!("../../assets/css/tailwind.css") } ))
        .route("/js/uri-helpers.js", get(|| async { include_str!("../../assets/js/uri-helpers.js") }))
}

use crate::api::user::User;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    tracing::debug!("Getting mock user for test");
    req.extensions_mut().insert(mock_user());
    Ok(next.run(req).await)
}

fn mock_user() -> User {
    User {
        id: 0,
        username: "alice".to_string(),
        domain: "".to_string(),
        email: "alice@wonder.land".to_string(),
        public_key: "".to_string(),
        private_key: None,
        activity_pub_id: "".to_string(),
        inbox_url: "".to_string(),
        outbox_url: "".to_string(),
        followers_url: "".to_string(),
        is_local: false,
    }
}

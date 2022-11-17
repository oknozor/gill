use crate::route::user::User;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth<B>(mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    tracing::debug!("Getting mock user for test");
    req.extensions_mut().insert(User {
        id: 0,
        username: "alice".to_string(),
        email: "alice@wonder.land".to_string(),
    });
    Ok(next.run(req).await)
}

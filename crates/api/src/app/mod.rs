use crate::route;
use aide::axum::routing::{get, post};
use aide::axum::ApiRouter;
use aide::axum_redoc::Redoc;
use aide::openapi::{Info, OpenApi};
use axum::Extension;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

pub fn app(pool: PgPool) -> ApiRouter {
    ApiRouter::new()
        .nest("/docs", Redoc::setup("/openapi.json").into())
        .route("/openapi.json", get(route::openapi::serve_api))
        .api_route("/", get(|| async { "Hello, World!" }))
        .api_route("/users", post(route::user::create))
        .api_route("/users/:id", get(route::user::by_id))
        .api_route("/repository/init", post(route::repository::init_repository))
        .api_route("/ssh_key/register", post(route::user::register_ssh_key))
        .layer(Extension(pool))
        .layer(TraceLayer::new_for_http())
}

pub async fn serve(db: PgPool, addr: SocketAddr) -> eyre::Result<()> {
    let mut api = OpenApi {
        info: Info {
            description: Some("Legit Api".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    axum::Server::bind(&addr)
        .serve(
            app(db)
                .finish_api(&mut api)
                .layer(Extension(api))
                .into_make_service(),
        )
        .await?;

    Ok(())
}

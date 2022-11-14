use axum::{routing::get, routing::post, Router, extract};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // build our application with a single route
    println!("starting server");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/repository/init", post(init_repository));


    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct InitRepository {
    name: String,
}

async fn init_repository(extract::Json(repository): extract::Json<InitRepository>) -> String {
    println!("Creating repository");
    gitox::init_bare(&repository.name)
        .expect("Failed to init repository");

    "Ok".to_string()
}
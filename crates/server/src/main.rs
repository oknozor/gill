use axum::{routing::get, routing::post, Router, extract};
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // build our application with a single route
    println!("starting server");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/repository/init", post(init_repository))
        .route("/ssh_key/register", post(register_ssh_key));


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

#[derive(Deserialize)]
struct CreateSSHKey {
    key: String,
}

async fn init_repository(extract::Json(repository): extract::Json<InitRepository>) -> String {
    println!("Creating repository");
    gitox::init_bare(&repository.name)
        .expect("Failed to init repository");

    "Ok".to_string()
}

async fn register_ssh_key(extract::Json(ssh_key): extract::Json<CreateSSHKey>) -> String {
    println!("Append ssh key {}", ssh_key.key);
    gitox::append_ssh_key(&ssh_key.key)
        .expect("Failed to append ssh key");

    "Ok".to_string()
}


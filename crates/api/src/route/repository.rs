use serde::Deserialize;
use aide::axum::IntoApiResponse;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
pub struct InitRepository {
    name: String,
}

pub async fn init_repository(axum::Json(repository): axum::Json<InitRepository>) -> impl IntoApiResponse {
    println!("Creating repository");
    git_lib::init_bare(&repository.name)
        .expect("Failed to init repository");

    "Ok".to_string()
}

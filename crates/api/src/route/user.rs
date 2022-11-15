use serde::Deserialize;
use aide::axum::IntoApiResponse;
use schemars::JsonSchema;

#[derive(Deserialize, JsonSchema)]
pub struct CreateSSHKey {
    key: String,
}

pub async fn register_ssh_key(axum::Json(ssh_key): axum::Json<CreateSSHKey>) -> impl IntoApiResponse {
    println!("Append ssh key {}", ssh_key.key);
    git_lib::append_ssh_key(&ssh_key.key)
        .expect("Failed to append ssh key");

    "Ok".to_string()
}

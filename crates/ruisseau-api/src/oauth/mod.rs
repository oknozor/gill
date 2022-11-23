use async_session::{MemoryStore, Session, SessionStore};
use axum::{
    async_trait,
    extract::{
        rejection::TypedHeaderRejectionReason, FromRef, FromRequestParts, Query, State, TypedHeader,
    },
    http::{header::SET_COOKIE, HeaderMap},
    response::{IntoResponse, Redirect, Response},
    RequestPartsExt,
};
use http::{header, request::Parts};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;

#[cfg(not(feature = "integration"))]
pub mod service;

#[cfg(feature = "integration")]
pub mod service_mock;

#[cfg(feature = "integration")]
pub use service_mock as service;

static COOKIE_NAME: &str = "RUISSEAU_SESSION";

#[derive(Clone)]
pub struct AppState {
    pub store: MemoryStore,
    pub oauth_client: BasicClient,
    /// Fixme: Make those static
    pub syntax_set: SyntaxSet,
    pub theme: Theme,
}

impl FromRef<AppState> for MemoryStore {
    fn from_ref(state: &AppState) -> Self {
        state.store.clone()
    }
}

impl FromRef<AppState> for BasicClient {
    fn from_ref(state: &AppState) -> Self {
        state.oauth_client.clone()
    }
}

impl FromRef<AppState> for SyntaxSet {
    fn from_ref(state: &AppState) -> Self {
        state.syntax_set.clone()
    }
}

impl FromRef<AppState> for Theme {
    fn from_ref(state: &AppState) -> Self {
        state.theme.clone()
    }
}

pub fn oauth_client() -> BasicClient {
    /// FIXME: take those from config or env
    let client_id = "ruisseau".to_string();
    let client_secret = "8Nup063EeIOYzSsEyVZkbo67sUpIX0Bc".to_string();
    let redirect_url = "http://127.0.0.1:3000/auth/authorized".to_string();
    let auth_url =
        "https://keycloak.cloud.hoohoot.org/auth/realms/hoohoot/protocol/openid-connect/auth"
            .to_string();
    let token_url =
        "https://keycloak.cloud.hoohoot.org/auth/realms/hoohoot/protocol/openid-connect/token"
            .to_string();

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(auth_url).unwrap(),
        Some(TokenUrl::new(token_url).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Oauth2User {
    pub sub: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub given_name: String,
    pub family_name: String,
    pub email: String,
}

pub struct AuthRedirect;

impl IntoResponse for AuthRedirect {
    fn into_response(self) -> Response {
        Redirect::temporary("/auth/ruisseau").into_response()
    }
}

pub async fn openid_auth(State(client): State<BasicClient>) -> impl IntoResponse {
    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Redirect::to(auth_url.as_ref())
}

pub async fn logout(
    State(store): State<MemoryStore>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
) -> impl IntoResponse {
    let cookie = cookies.get(COOKIE_NAME).unwrap();
    let session = match store.load_session(cookie.to_string()).await.unwrap() {
        Some(s) => s,
        // No session active, just redirect
        None => return Redirect::to("/"),
    };

    store.destroy_session(session).await.unwrap();

    Redirect::to("/")
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub async fn login_authorized(
    Query(query): Query<AuthRequest>,
    State(store): State<MemoryStore>,
    State(oauth_client): State<BasicClient>,
) -> impl IntoResponse {
    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code.clone()))
        .request_async(async_http_client)
        .await
        .unwrap();

    let client = reqwest::Client::new();
    /// FIXME: get from env/config
    let user_data: Oauth2User = client
        .get("https://keycloak.cloud.hoohoot.org/auth/realms/hoohoot/protocol/openid-connect/userinfo")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .unwrap()
        .json::<Oauth2User>()
        .await
        .unwrap();

    let mut session = Session::new();
    session.insert("user", &user_data).unwrap();
    let cookie = store.store_session(session).await.unwrap().unwrap();
    let cookie = format!("{}={}; SameSite=Lax; Path=/", COOKIE_NAME, cookie);
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());

    (headers, Redirect::to("/"))
}

#[async_trait]
impl<S> FromRequestParts<S> for Oauth2User
where
    MemoryStore: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthRedirect;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let store = MemoryStore::from_ref(state);
        let cookies = parts
            .extract::<TypedHeader<headers::Cookie>>()
            .await
            .map_err(|e| match *e.name() {
                header::COOKIE => match e.reason() {
                    TypedHeaderRejectionReason::Missing => AuthRedirect,
                    _ => panic!("unexpected error getting Cookie header(s): {}", e),
                },
                _ => panic!("unexpected error getting cookies: {}", e),
            })?;
        let session_cookie = cookies.get(COOKIE_NAME).ok_or(AuthRedirect)?;
        let session = store
            .load_session(session_cookie.to_string())
            .await
            .unwrap()
            .ok_or(AuthRedirect)?;

        let user = session.get::<Oauth2User>("user").ok_or(AuthRedirect)?;

        Ok(user)
    }
}

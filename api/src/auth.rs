use argon2::{self, Config};
use regex::Regex;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket::{http::Status, State};
use serde::{Deserialize, Serialize};

pub(crate) struct BearerToken<'r>(&'r str);

#[derive(Debug)]
pub(crate) enum BearerTokenError {
    Missing,
}

fn extract(key: &str) -> Option<&str> {
    let re = Regex::new(r"^(?i)bearer(?-i)\s*(.+)\s*$").expect("Failed to compile bearer regex");
    if let Some(capt) = re.captures(key) {
        if let Some(token) = capt.get(1) {
            return Some(token.as_str());
        }
    }
    None
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BearerToken<'r> {
    type Error = BearerTokenError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("authorization") {
            None => Outcome::Failure((Status::Unauthorized, BearerTokenError::Missing)),
            Some(key) => match extract(key) {
                Some(token) => Outcome::Success(BearerToken(token)),
                None => Outcome::Failure((Status::Unauthorized, BearerTokenError::Missing)),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AuthResponse {
    #[serde(skip_serializing_if = "Option::is_none", rename = "X-Hasura-User-Id")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "X-Hasura-Actor-Id")]
    pub actor_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "X-Hasura-Role")]
    pub role: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "X-Hasura-Allowed-Roles"
    )]
    pub allowed_roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "Cache-Control")]
    pub cache_control: Option<String>,
}

impl AuthResponse {
    fn build_auth_server() -> Self {
        AuthResponse {
            user_id: None,
            actor_id: None,
            role: Some("auth-server".to_string()),
            allowed_roles: None,
            cache_control: None,
        }
    }
}

async fn is_internal_token(token: &str, cfg: &crate::config::AppConfiguration) -> bool {
    token.eq(&cfg.internal_token)
}

pub(crate) fn hash_password(password: &str) -> crate::Result<String> {
    let mut salt: Vec<u8> = Vec::new();

    for _n in 0..32 {
        salt.push(rand::random::<u8>());
    }

    let config = Config::default();
    let hash = argon2::hash_encoded(password.as_bytes(), &salt, &config).map_err(|e| {
        crate::ArgonHashSnafu {
            msg: format!("{}", e),
        }
        .build()
    })?;

    Ok(hash)
}

pub(crate) fn validate_password(password: &str, hashed_password: &str) -> crate::Result<bool> {
    let v = argon2::verify_encoded(hashed_password, password.as_bytes()).map_err(|e| {
        crate::ArgonHashSnafu {
            msg: format!("{}", e),
        }
        .build()
    })?;

    Ok(v)
}

pub(crate) fn new_token(
    _cfg: &crate::config::AppConfiguration,
) -> crate::Result<(String, String)> {
    let mut binary: Vec<u8> = Vec::new();

    for _n in 0..32 {
        binary.push(rand::random::<u8>());
    }

    let plain_text = base64::encode(&binary);
    let hashed = sha256::digest_bytes(binary.as_slice());

    // sha256 + base64 encode it
    Ok((plain_text, hashed))
}

fn convert_plain_token(token: &str) -> crate::Result<String> {
    // decode base64 token
    let binary = base64::decode(token).map_err(|e| {
        crate::UnauthorizedSnafu {
            msg: "Invalid token".to_string(),
            detail: format!("Not base64 decodable: {}", e),
        }
        .build()
    })?;

    // sha256 + base64 encode it
    Ok(sha256::digest_bytes(binary.as_slice()))
}

#[rocket::get("/auth")]
pub(crate) async fn auth(
    token: BearerToken<'_>,
    cfg: &State<crate::config::AppConfiguration>,
) -> crate::Result<Json<AuthResponse>> {
    if is_internal_token(token.0, cfg).await {
        Ok(Json(AuthResponse::build_auth_server()))
    } else {
        let hashed_token = convert_plain_token(token.0)?;

        println!("Checking token {}", hashed_token);
        let result =
            crate::operations::auth::fetch_auth_token::resolve_auth_token(&hashed_token, cfg)
                .await
                .map_err(|e| {
                    crate::UnauthorizedSnafu {
                        msg: "Invalid token".to_string(),
                        detail: format!("Lookup in hasura failed: {}", e),
                    }
                    .build()
                })?;

        if let Some(response) = result {
            Ok(Json(response))
        } else {
            println!("No such token found");
            crate::UnauthorizedSnafu {
                msg: "Invalid token".to_string(),
                detail: "No such token found".to_string(),
            }
            .fail()
        }
    }
}

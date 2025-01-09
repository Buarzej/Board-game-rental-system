use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Claims {
    pub(crate) sub: i32,
    pub(crate) is_admin: bool,
    iat: usize,
    exp: usize,
}

/// Generates a JWT token for the given user.
pub(crate) fn generate_jwt(
    user_id: i32,
    is_admin: bool,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        sub: user_id,
        is_admin,
        iat: Utc::now().timestamp() as usize,
        exp: (Utc::now() + Duration::minutes(30)).timestamp() as usize,
    };
    let key = env::var("JWT_SECRET").unwrap();

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(key.as_ref()),
    )
}

/// Verifies a JWT token and returns the claims if the token is valid.
pub(crate) fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let key = env::var("JWT_SECRET").unwrap();
    jsonwebtoken::decode::<Claims>(
        token,
        &DecodingKey::from_secret(key.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
}

/// Hashes a password using the Argon2 algorithm.
pub(crate) fn hash_password(password: String) -> Result<String, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

/// Verifies a password against a password hash.
pub(crate) fn verify_password(
    password: String,
    password_hash: String,
) -> Result<bool, password_hash::Error> {
    let parsed_hash = PasswordHash::new(&password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// TODO: handle email verification

// TODO: handle forgot password

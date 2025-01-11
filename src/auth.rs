use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use lettre::transport::smtp::{authentication::Credentials, Error as SmtpError};

use lettre::{Message, SmtpTransport, Transport};
use serde::{Deserialize, Serialize};
use std::env;
use lettre::message::Mailbox;
use uuid::Uuid;

const CONFIRMATION_EMAIL_SUBJECT: &str = "Potwierdź swój email";

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

/// Sends a confirmation email to the given email address.
pub(crate) fn send_confirmation_email(
    user_id: i32,
    email: String,
    token: Uuid,
) -> Result<(), SmtpError> {
    let credentials = Credentials::new(
        env::var("MAILER_USERNAME").unwrap(),
        env::var("MAILER_PASSWORD").unwrap(),
    );

    let smtp_server = env::var("MAILER_HOST").unwrap();
    let smtp_port = env::var("MAILER_PORT").unwrap().parse::<u16>().unwrap();

    let mailer = SmtpTransport::starttls_relay(&smtp_server)?
        .port(smtp_port)
        .credentials(credentials)
        .build();

    let from: Mailbox = match env::var("MAILER_USERNAME").unwrap().parse() {
        Ok(mailbox) => mailbox,
        Err(e) => panic!("Invalid email address: {}", e),
    };
    let to: Mailbox = match email.parse() {
        Ok(mailbox) => mailbox,
        Err(e) => panic!("Invalid email address: {}", e),
    };
    let confirmation_link = format!(
        "{}/api/user/confirm/{}/{}",
        env::var("FRONTEND_URL").unwrap(),
        user_id,
        token
    );

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(CONFIRMATION_EMAIL_SUBJECT)
        .body(confirmation_link)
        .unwrap();

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{rand_core::OsRng, SaltString};
use base64::Engine;

pub fn hash_password(password: &str) -> Result<String, String> {
  let salt = SaltString::generate(&mut OsRng);
  Argon2::default()
    .hash_password(password.as_bytes(), &salt)
    .map(|h| h.to_string())
    .map_err(|e| format!("{:?}", e))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
  let parsed_hash = PasswordHash::new(hash).map_err(|e| format!("{:?}", e))?;
  Argon2::default()
    .verify_password(password.as_bytes(), &parsed_hash)
    .map(|_| true)
    .map_err(|e| format!("{:?}", e))
}

pub fn generate_csrf_token() -> String {
  let bytes: [u8; 32] = rand::random();
  base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Extract the CSRF token from the `X-CSRF-Token` request header.
pub fn extract_csrf_token(headers: &axum::http::HeaderMap) -> Option<String> {
  headers
    .get("X-CSRF-Token")
    .and_then(|v| v.to_str().ok())
    .map(|s| s.to_string())
}

/// Constant-time comparison of provided vs expected CSRF token.
pub fn validate_csrf_token(provided: &str, expected: &str) -> bool {
  if provided.len() != expected.len() {
    return false;
  }
  let mut diff = 0u8;
  for (a, b) in provided.as_bytes().iter().zip(expected.as_bytes().iter()) {
    diff |= a ^ b;
  }
  diff == 0
}
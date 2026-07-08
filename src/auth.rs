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
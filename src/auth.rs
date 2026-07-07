use base64::Engine;
use sha2::{Digest, Sha256};

pub fn hash_password(password: &str) -> Result<String, String> {
  let mut hasher = Sha256::new();
  hasher.update(password.as_bytes());
  let result = hasher.finalize();
  Ok(hex::encode(result))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
  let entered_hash = hash_password(password)?;
  Ok(entered_hash == hash)
}

pub fn generate_csrf_token() -> String {
  let bytes = rand::random::<[u8; 32]>();
  base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
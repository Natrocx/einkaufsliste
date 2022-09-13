pub(crate) mod article;
pub(crate) mod item;
pub(crate) mod shop;
pub(crate) mod user;

pub(crate) fn hash_password_with_salt(password: &str, salt: &[u8]) -> Vec<u8> {
  let mut hasher = blake3::Hasher::new();
  hasher.update(password.as_bytes());
  hasher.update(salt);

  hasher.finalize().as_bytes().to_vec()
}

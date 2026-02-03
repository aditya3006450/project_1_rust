use bcrypt::{DEFAULT_COST, hash, verify};

pub fn hash_password(password: String) -> String {
    hash(password, DEFAULT_COST).unwrap()
}

pub fn verify_password(password: String, hash: String) -> bool {
    verify(password, hash.as_str()).unwrap_or(false)
}

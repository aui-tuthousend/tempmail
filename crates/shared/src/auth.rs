use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, Params};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::{Result, TempMailError};

const TOKEN_BYTES: usize = 32;

#[derive(Debug, Clone)]
pub struct PasswordService {
    argon2: Argon2<'static>,
}

impl PasswordService {
    pub fn new() -> Result<Self> {
        let params =
            Params::new(19_456, 2, 1, None).map_err(|source| TempMailError::AuthConfig {
                message: source.to_string(),
            })?;

        Ok(Self {
            argon2: Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params),
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|source| TempMailError::PasswordHash {
                message: source.to_string(),
            })
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(password_hash).map_err(|source| TempMailError::PasswordHash {
                message: source.to_string(),
            })?;

        Ok(self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

impl Default for PasswordService {
    fn default() -> Self {
        Self::new().expect("valid Argon2id password parameters")
    }
}

#[derive(Debug, Clone, Default)]
pub struct TokenService;

impl TokenService {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_token(&self) -> String {
        let mut bytes = [0_u8; TOKEN_BYTES];
        rand::thread_rng().fill_bytes(&mut bytes);
        URL_SAFE_NO_PAD.encode(bytes)
    }

    pub fn hash_token(&self, token: &str) -> String {
        let digest = Sha256::digest(token.as_bytes());
        hex::encode(digest)
    }

    pub fn generate_token_pair(&self) -> TokenPair {
        let token = self.generate_token();
        let token_hash = self.hash_token(&token);

        TokenPair { token, token_hash }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenPair {
    pub token: String,
    pub token_hash: String,
}

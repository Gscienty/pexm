use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Claim {
    iss: String,      // issuer
    exp: u64,         // expiration time
    sub: u64,         // subject
    aud: Vec<String>, // audience
    nbf: u64,         // not before
    iat: u64,         // issued at
    jti: u64,         // jwt id

    name: String,
}

impl Claim {
    pub fn new(sub: u64, name: String, jti: u64) -> Self {
        let iat = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(now) => now.as_secs(),
            Err(_) => 0,
        };

        Self {
            sub,
            name,
            jti,
            iat,
            nbf: iat,

            ..Default::default()
        }
    }

    pub const fn get_subject(&self) -> u64 {
        self.sub
    }

    pub fn set_issuer(&mut self, iss: &str) {
        self.iss = iss.to_owned();
    }

    pub fn set_expiration_time(&mut self, exp: u64) {
        self.exp = self.iat + exp;
    }

    pub fn set_audiences(&mut self, audiences: &[String]) {
        self.aud = audiences.to_owned()
    }

    pub fn set_not_before(&mut self, nbf: u64) {
        self.nbf = nbf;
    }

    pub fn validate_time(&self, now: u64) -> bool {
        if now < self.nbf || now > self.exp {
            return false;
        }

        true
    }
}

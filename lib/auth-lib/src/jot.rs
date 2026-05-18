use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error,
};
use ring::error::KeyRejected;

use crate::claim::Claim;

pub struct Jot {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,

    header: Header,
    validation: Validation,
}

impl Jot {
    pub fn new(pkey: &[u8], pubkey: &[u8]) -> Result<Self, KeyRejected> {
        let encoding_key = EncodingKey::from_ed_pem(pkey).unwrap();
        let decoding_key = DecodingKey::from_ed_pem(pubkey).unwrap();

        let validation = Validation::new(Algorithm::EdDSA);

        Ok(Self {
            encoding_key,
            decoding_key,

            header: Header::new(Algorithm::EdDSA),
            validation,
        })
    }

    pub fn encode(&self, claims: &Claim) -> Result<String, Error> {
        encode(&self.header, claims, &self.encoding_key)
    }

    pub fn set_validation_audience(&mut self, items: &[String]) {
        self.validation.set_audience(items)
    }

    pub fn decode(&self, token: &str) -> Result<Claim, Error> {
        let token_data = decode::<Claim>(token, &self.decoding_key, &self.validation)?;

        Ok(token_data.claims)
    }
}

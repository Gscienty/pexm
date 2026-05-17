use auth_lib::claim;

pub struct Principal {
    id: u64,
    name: String,
    claim: claim::Claim,
    connection_id: u64,
}

impl Principal {
    pub fn new(id: u64, name: String, claim: claim::Claim, connection_id: u64) -> Self {
        Self {
            id,
            name,
            claim,
            connection_id,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn get_doc_name(&self) -> &str {
        &self.name
    }
    pub fn get_claim(&self) -> &claim::Claim {
        &self.claim
    }

    pub fn get_connection_id(&self) -> u64 {
        self.connection_id
    }
}

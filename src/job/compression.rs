use super::JobTrait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionJob {
    pub id: Uuid,
    pub auth: String,
}

impl JobTrait for CompressionJob {
    fn id(&self) -> Uuid {
        self.id
    }

    fn auth(&self) -> &str {
        &self.auth
    }
}

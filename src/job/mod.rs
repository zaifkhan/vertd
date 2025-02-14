pub mod compression;
pub mod conversion;

use compression::CompressionJob;
use conversion::ConversionJob;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub trait JobTrait {
    fn id(&self) -> Uuid;
    fn auth(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Job {
    Conversion(ConversionJob),
    Compression(CompressionJob),
}

impl JobTrait for Job {
    fn id(&self) -> Uuid {
        match self {
            Job::Conversion(job) => job.id(),
            Job::Compression(job) => job.id(),
        }
    }

    fn auth(&self) -> &str {
        match self {
            Job::Conversion(job) => job.auth(),
            Job::Compression(job) => job.auth(),
        }
    }
}

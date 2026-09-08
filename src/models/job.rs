use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Job {
    pub id: Uuid,
    pub r#type: String,
    pub payload: String,
    pub status: JobStatus,
    pub attempts: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    PENDING,
    PROCESSING,
    COMPLETED,
    FAILED,
}

impl Job {
    pub fn new(r#type: String, payload: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            r#type,
            payload,
            status: JobStatus::PENDING,
            attempts: 0,
        }
    }
}

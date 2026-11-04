use super::DatabaseError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthStatus {
    pub version: String,
    pub uptime_secs: u64,
}

#[async_trait]
pub trait Health: Send + Sync {
    /// Cheap liveness check.
    async fn ping(&self) -> Result<(), DatabaseError>;

    /// Server version and uptime.
    async fn status(&self) -> Result<HealthStatus, DatabaseError>;
}

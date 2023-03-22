use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ParticleDetail {
    Low,
    Medium,
    High,
}

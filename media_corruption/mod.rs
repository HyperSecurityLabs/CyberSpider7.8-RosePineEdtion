pub mod detector;
pub mod attacker;

pub use detector::MediaCorruptionDetector;
pub use attacker::MediaCorruptionAttacker;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaCorruptionFinding {
    pub url: String,
    pub file_type: String,
    pub corruption_type: CorruptionType,
    pub severity: MediaSeverity,
    pub description: String,
    pub evidence: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorruptionType {
    InvalidHeader,
    TruncatedContent,
    BrokenMagicBytes,
    MimeMismatch,
    ExecutableEmbedded,
    SuspiciousMetadata,
    CorruptImageData,
    MissingRequiredChunks,
    ZeroLength,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

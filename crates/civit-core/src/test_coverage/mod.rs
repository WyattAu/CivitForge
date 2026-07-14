#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::TestCoverageStore;
pub use types::{
    CoverageReport, CoverageSummary, CoverageTrend, CoverageEnforcementResult,
    CoverageUploadRequest, CoverageEnforcementConfig,
};

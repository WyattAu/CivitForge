#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::TestCoverageStore;
pub use types::{
    CoverageEnforcementConfig, CoverageEnforcementResult, CoverageReport, CoverageReportV2,
    CoverageSummary, CoverageTrend, CoverageTrendV2, CoverageUploadRequest,
    CoverageUploadRequestV2,
};

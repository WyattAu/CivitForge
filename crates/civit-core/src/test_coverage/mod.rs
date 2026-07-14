#![forbid(unsafe_code)]

pub mod store;
pub mod types;

pub use store::TestCoverageStore;
pub use types::{
    CoverageReport, CoverageReportV2, CoverageSummary, CoverageTrend, CoverageTrendV2,
    CoverageEnforcementResult, CoverageUploadRequest, CoverageUploadRequestV2,
    CoverageEnforcementConfig,
};

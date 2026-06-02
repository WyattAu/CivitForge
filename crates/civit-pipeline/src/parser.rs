//! Pipeline YAML parser.

use crate::error::{PipelineError, Result};
use crate::model::Pipeline;

/// Parse a `.civit/pipeline.yaml` string into a typed Pipeline.
pub fn parse_pipeline(yaml: &str) -> Result<Pipeline> {
    let pipeline: Pipeline =
        serde_yaml::from_str(yaml).map_err(|e| PipelineError::YamlParse(e.to_string()))?;

    if pipeline.version != "1" {
        return Err(PipelineError::InvalidVersion(pipeline.version));
    }

    Ok(pipeline)
}

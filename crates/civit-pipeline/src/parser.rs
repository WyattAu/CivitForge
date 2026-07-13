//! Pipeline YAML parser.

use crate::error::{PipelineError, Result};
use crate::model::Pipeline;

/// Parse a `.civit/pipeline.yaml` string into a typed Pipeline.
pub fn parse_pipeline(yaml: &str) -> Result<Pipeline> {
    let pipeline: Pipeline =
        serde_yaml::from_str(yaml).map_err(|e| PipelineError::YamlParse(e.to_string()))?;

    if pipeline.version != "1" && pipeline.version != "2" {
        return Err(PipelineError::InvalidVersion(pipeline.version));
    }

    // Validate v2-specific features
    if pipeline.version == "1" {
        // v1 should not have v2-only fields
        if pipeline.include.is_some() {
            return Err(PipelineError::Validation(
                "include is not supported in v1 pipelines".into(),
            ));
        }
    }

    Ok(pipeline)
}

/// Parse an included YAML file into a Pipeline.
pub fn parse_included_pipeline(yaml: &str) -> Result<Pipeline> {
    let pipeline: Pipeline =
        serde_yaml::from_str(yaml).map_err(|e| PipelineError::YamlParse(e.to_string()))?;
    Ok(pipeline)
}

/// Resolve include statements by reading and merging included files.
///
/// This function processes all `include` statements in a v2 pipeline,
/// reading the referenced files and merging their jobs into the main pipeline.
/// Context variables from the include statement are injected as environment variables.
pub fn resolve_includes(
    pipeline: &mut Pipeline,
    included_files: &[(crate::model::Include, String)],
) -> Result<()> {
    for (include, content) in included_files {
        let included_pipeline = parse_included_pipeline(content)?;

        // Inject context variables as environment variables in included jobs
        let mut jobs = included_pipeline.jobs;
        if let Some(context) = &include.context {
            for job in &mut jobs {
                let mut env = job.env.clone().unwrap_or_default();
                for (key, value) in context {
                    let env_name = format!("INCLUDE_{}", key.to_uppercase());
                    let value_str = match value {
                        serde_yaml::Value::String(s) => s.clone(),
                        serde_yaml::Value::Number(n) => n.to_string(),
                        serde_yaml::Value::Bool(b) => b.to_string(),
                        serde_yaml::Value::Null => String::new(),
                        _ => serde_json::to_string(value).unwrap_or_default(),
                    };
                    env.push(crate::model::EnvVar {
                        name: env_name,
                        value: Some(value_str),
                        from_secret: None,
                        description: Some("Context variable from include".to_string()),
                    });
                }
                if !env.is_empty() {
                    job.env = Some(env);
                }
            }
        }

        // Merge jobs from included pipeline
        pipeline.jobs.extend(jobs);
    }

    Ok(())
}

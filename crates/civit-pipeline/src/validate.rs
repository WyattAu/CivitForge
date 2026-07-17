//! Pipeline validation.

use crate::error::{PipelineError, Result};
use crate::model::{Job, Pipeline};

/// Validate a parsed pipeline for structural correctness.
pub fn validate_pipeline(pipeline: &Pipeline) -> Result<()> {
    // Check for duplicate job names
    let mut seen = std::collections::HashSet::new();
    for job in &pipeline.jobs {
        if !seen.insert(&job.name) {
            return Err(PipelineError::DuplicateJob(job.name.clone()));
        }
    }

    // Build name set for dependency validation
    let all_names: std::collections::HashSet<String> =
        pipeline.jobs.iter().map(|j| j.name.clone()).collect();

    // Validate each job
    for job in &pipeline.jobs {
        validate_job(job, &all_names)?;
    }

    // Check for circular dependencies (topological sort)
    detect_circular_deps(&pipeline.jobs)?;

    // Validate schedule triggers
    if let Some(triggers) = &pipeline.on
        && let Some(schedules) = &triggers.schedule
    {
        for sched in schedules {
            if !crate::trigger::validate_cron(&sched.cron) {
                return Err(PipelineError::InvalidCron(sched.cron.clone()));
            }
        }
    }

    // Validate v2-specific features
    if pipeline.version == "2" {
        validate_v2_features(pipeline)?;
    }

    Ok(())
}

fn validate_job(job: &Job, all_names: &std::collections::HashSet<String>) -> Result<()> {
    if job.steps.is_empty() {
        return Err(PipelineError::EmptyJob {
            name: job.name.clone(),
        });
    }

    // Validate dependencies exist
    if let Some(needs) = &job.needs {
        for dep in needs {
            if dep.is_empty() {
                return Err(PipelineError::Validation(format!(
                    "job '{}' has empty dependency name",
                    job.name
                )));
            }
            if !all_names.contains(dep) {
                return Err(PipelineError::UnknownDependency {
                    job: job.name.clone(),
                    dep: dep.clone(),
                });
            }
        }
    }

    // Validate timeout
    if let Some(timeout) = &job.timeout
        && timeout.to_duration().is_none()
    {
        return Err(PipelineError::InvalidTimeout(timeout.to_string()));
    }

    // Validate step-level timeouts
    for step in &job.steps {
        if let Some(timeout) = &step.timeout
            && timeout.to_duration().is_none()
        {
            return Err(PipelineError::InvalidTimeout(timeout.to_string()));
        }
    }

    // Validate service port ranges
    if let Some(services) = &job.services {
        for svc in services {
            if let Some(ports) = &svc.ports {
                for port in ports {
                    if port.port == 0 {
                        return Err(PipelineError::Validation(format!(
                            "service '{}' has port 0",
                            svc.name
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

/// Detect circular dependencies using DFS.
fn detect_circular_deps(jobs: &[Job]) -> Result<()> {
    let mut visiting = std::collections::HashSet::new();
    let mut in_stack = std::collections::HashSet::new();

    for job in jobs {
        if in_stack.contains(&job.name) {
            continue;
        }
        dfs(&job.name, jobs, &mut visiting, &mut in_stack)?;
    }

    Ok(())
}

fn dfs(
    current: &str,
    jobs: &[Job],
    visiting: &mut std::collections::HashSet<String>,
    in_stack: &mut std::collections::HashSet<String>,
) -> Result<()> {
    if in_stack.contains(current) {
        let chain: Vec<String> = in_stack.iter().cloned().collect();
        return Err(PipelineError::CircularDependency {
            name: current.to_string(),
            chain,
        });
    }

    if visiting.contains(current) {
        return Ok(());
    }

    visiting.insert(current.to_string());
    in_stack.insert(current.to_string());

    if let Some(job) = jobs.iter().find(|j| j.name == current)
        && let Some(needs) = &job.needs
    {
        for dep in needs {
            dfs(dep, jobs, visiting, in_stack)?;
        }
    }

    in_stack.remove(current);
    Ok(())
}

/// Validate v2-specific features.
fn validate_v2_features(pipeline: &Pipeline) -> Result<()> {
    let all_names: std::collections::HashSet<String> =
        pipeline.jobs.iter().map(|j| j.name.clone()).collect();

    // Validate include paths
    if let Some(includes) = &pipeline.include {
        for inc in includes {
            if inc.source.is_empty() {
                return Err(PipelineError::Validation(
                    "include source path cannot be empty".into(),
                ));
            }
        }
    }

    // Validate artifact dependencies
    for job in &pipeline.jobs {
        if let Some(artifacts_from) = &job.artifacts_from {
            for dep in artifacts_from {
                if !all_names.contains(dep) {
                    return Err(PipelineError::Validation(format!(
                        "job '{}' references artifact from unknown job '{}'",
                        job.name, dep
                    )));
                }
            }
        }

        // Validate before/after hooks
        if let Some(before) = &job.before
            && before.is_empty()
        {
            return Err(PipelineError::Validation(format!(
                "job '{}' has empty before hooks",
                job.name
            )));
        }

        if let Some(after) = &job.after
            && after.is_empty()
        {
            return Err(PipelineError::Validation(format!(
                "job '{}' has empty after hooks",
                job.name
            )));
        }
    }

    Ok(())
}

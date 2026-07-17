//! Matrix expansion logic.
//!
//! Takes a parsed Pipeline with `strategy.matrix` on jobs and expands each
//! such job into multiple concrete jobs — one per combination of matrix values.

use std::collections::HashMap;

use crate::error::{PipelineError, Result};
use crate::model::{Job, Pipeline, RunsOn, Step};

/// Expand matrix jobs in a pipeline.
///
/// For each job with `strategy.matrix`, the cross-product of all dimensions is
/// computed, then `include` entries are appended and `exclude` entries removed.
/// Each resulting combination produces a new job with:
/// - Name suffixed by the combination key (e.g. `build-linux-stable`)
/// - `runs-on` replaced by the `runs_on` value if present in the matrix
/// - Environment variables `MATRIX_<key>` injected for each dimension
///
/// Jobs without a strategy are returned unchanged.
pub fn expand_matrix(pipeline: &Pipeline) -> Result<Pipeline> {
    let mut expanded_jobs = Vec::new();

    for job in &pipeline.jobs {
        match &job.strategy {
            Some(strategy) => {
                let combinations = compute_combinations(&strategy.matrix)?;
                for combo in combinations {
                    expanded_jobs.push(apply_combination(job, &combo)?);
                }
            }
            None => {
                expanded_jobs.push(job.clone());
            }
        }
    }

    Ok(Pipeline {
        jobs: expanded_jobs,
        ..pipeline.clone()
    })
}

/// A single matrix combination: dimension name → chosen value.
type Combination = HashMap<String, String>;

/// Compute the cross-product of all matrix dimensions, then apply include/exclude.
fn compute_combinations(matrix: &crate::model::MatrixConfig) -> Result<Vec<Combination>> {
    // Start with the cross-product of all dimensions
    let dim_names: Vec<&String> = matrix.dimensions.keys().collect();
    let dim_values: Vec<&Vec<String>> = matrix.dimensions.values().collect();

    let mut combos: Vec<Combination> = if dim_values.is_empty() {
        vec![HashMap::new()]
    } else {
        cross_product(&dim_names, &dim_values)
    };

    // Apply exclude: remove matching combinations
    for excl in &matrix.exclude {
        combos.retain(|combo| !combo_matches_object(combo, excl));
    }

    // Apply include: add extra combinations
    for incl in &matrix.include {
        let mut combo = HashMap::new();
        if let serde_yaml::Value::Mapping(map) = incl {
            for (k, v) in map {
                if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                    combo.insert(key.to_string(), val.to_string());
                }
            }
        }
        if !combo.is_empty() {
            combos.push(combo);
        }
    }

    if combos.is_empty() {
        return Err(PipelineError::Validation(
            "matrix expansion produced no combinations".into(),
        ));
    }

    Ok(combos)
}

/// Compute the Cartesian product of named dimensions.
fn cross_product(names: &[&String], values: &[&Vec<String>]) -> Vec<Combination> {
    if names.is_empty() {
        return vec![HashMap::new()];
    }

    let first_name = names[0].clone();
    let first_vals = values[0];
    let rest_names = &names[1..];
    let rest_vals = &values[1..];

    let mut result = Vec::new();
    for val in first_vals {
        for mut rest in cross_product(rest_names, rest_vals) {
            rest.insert(first_name.clone(), val.clone());
            result.push(rest);
        }
    }
    result
}

/// Check if a matrix combination matches a YAML object (for exclude/include).
fn combo_matches_object(combo: &Combination, obj: &serde_yaml::Value) -> bool {
    if let serde_yaml::Value::Mapping(map) = obj {
        map.iter().all(|(k, v)| {
            if let (Some(key), Some(val)) = (k.as_str(), v.as_str()) {
                combo.get(key).map(|c| c.as_str()) == Some(val)
            } else {
                true
            }
        })
    } else {
        false
    }
}

/// Apply a matrix combination to a job template.
fn apply_combination(template: &Job, combo: &Combination) -> Result<Job> {
    // Build suffix from combination values (sorted for determinism)
    let mut sorted_keys: Vec<&String> = combo.keys().collect();
    sorted_keys.sort();
    let suffix: String = sorted_keys
        .iter()
        .map(|k| combo[*k].as_str())
        .collect::<Vec<_>>()
        .join("-");

    let new_name = format!("{}-{suffix}", template.name);

    // Resolve runs-on from matrix if present
    let runs_on = combo
        .get("runs-on")
        .map(|label| RunsOn {
            labels: Some(vec![label.clone()]),
            group: None,
        })
        .or_else(|| template.runs_on.clone());

    // Inject MATRIX_<KEY> env vars
    let mut env = template.env.clone().unwrap_or_default();
    for (key, value) in combo {
        let env_name = format!("MATRIX_{}", key.to_uppercase().replace('-', "_"));
        // Remove existing MATRIX_ var with same name if present
        env.retain(|e| e.name != env_name);
        env.push(crate::model::EnvVar {
            name: env_name,
            value: Some(value.clone()),
            from_secret: None,
            description: Some(format!("Matrix dimension value for '{key}'")),
        });
    }

    // Update step templates: replace `${{ matrix.<key> }}` expressions
    let resolved_steps: Vec<Step> = template
        .steps
        .iter()
        .map(|step| resolve_step_expressions(step, combo))
        .collect();

    Ok(Job {
        name: new_name,
        condition: template.condition.clone(),
        needs: template.needs.clone(),
        runs_on,
        strategy: None, // Expanded — no longer a matrix job
        timeout: template.timeout.clone(),
        env: if env.is_empty() { None } else { Some(env) },
        secrets: template.secrets.clone(),
        services: template.services.clone(),
        before: template.before.clone(),
        after: template.after.clone(),
        environment: template.environment.clone(),
        artifacts_from: template.artifacts_from.clone(),
        steps: resolved_steps,
    })
}

/// Replace `${{ matrix.<key> }}` in step run commands and shell commands.
fn resolve_step_expressions(step: &Step, combo: &Combination) -> Step {
    let run = step.run.as_ref().map(|cmds| {
        cmds.iter()
            .map(|cmd| substitute_matrix_expr(cmd, combo))
            .collect()
    });

    Step {
        name: step.name.clone(),
        description: step.description.clone(),
        image: step.image.clone(),
        run,
        shell: step.shell.clone(),
        workdir: substitute_matrix_expr(&step.workdir, combo),
        env: step.env.clone(),
        secrets: step.secrets.clone(),
        continue_on_error: step.continue_on_error,
        timeout: step.timeout.clone(),
        condition: step
            .condition
            .as_ref()
            .map(|c| substitute_matrix_expr(c, combo)),
        uses: step.uses.clone(),
        checkout: step.checkout.clone(),
        cache: step.cache.clone(),
        artifact: step.artifact.clone(),
        retry: step.retry.clone(),
    }
}

/// Substitute `${{ matrix.<key> }}` placeholders in a string.
fn substitute_matrix_expr(s: &str, combo: &Combination) -> String {
    let mut result = s.to_string();
    for (key, value) in combo {
        let placeholder = format!("${{{{ matrix.{key} }}}}");
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;
    use std::collections::HashMap;

    fn make_test_pipeline(yaml: &str) -> Pipeline {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn test_cross_product_two_dims() {
        let mut dims = HashMap::new();
        dims.insert(
            "os".to_string(),
            vec!["ubuntu-latest".to_string(), "macos-latest".to_string()],
        );
        dims.insert(
            "rust".to_string(),
            vec!["stable".to_string(), "nightly".to_string()],
        );

        let names: Vec<&String> = dims.keys().collect();
        let vals: Vec<&Vec<String>> = dims.values().collect();
        let combos = cross_product(&names, &vals);
        assert_eq!(combos.len(), 4);
    }

    #[test]
    fn test_cross_product_single_dim() {
        let mut dims = HashMap::new();
        dims.insert(
            "os".to_string(),
            vec!["ubuntu-latest".to_string(), "windows-latest".to_string()],
        );

        let names: Vec<&String> = dims.keys().collect();
        let vals: Vec<&Vec<String>> = dims.values().collect();
        let combos = cross_product(&names, &vals);
        assert_eq!(combos.len(), 2);
    }

    #[test]
    fn test_cross_product_empty() {
        let dims: HashMap<String, Vec<String>> = HashMap::new();
        let names: Vec<&String> = dims.keys().collect();
        let vals: Vec<&Vec<String>> = dims.values().collect();
        let combos = cross_product(&names, &vals);
        assert_eq!(combos.len(), 1);
        assert!(combos[0].is_empty());
    }

    #[test]
    fn test_substitute_matrix_expr() {
        let mut combo = HashMap::new();
        combo.insert("os".to_string(), "ubuntu-latest".to_string());
        combo.insert("rust".to_string(), "stable".to_string());

        let result = substitute_matrix_expr("cargo +${{ matrix.rust }} test", &combo);
        assert_eq!(result, "cargo +stable test");
    }

    #[test]
    fn test_substitute_multiple_exprs() {
        let mut combo = HashMap::new();
        combo.insert("os".to_string(), "ubuntu-latest".to_string());
        combo.insert("rust".to_string(), "stable".to_string());

        let result = substitute_matrix_expr("${{ matrix.os }}/${{ matrix.rust }}", &combo);
        assert_eq!(result, "ubuntu-latest/stable");
    }

    #[test]
    fn test_apply_combination_name() {
        let mut dims = HashMap::new();
        dims.insert("os".to_string(), vec!["ubuntu-latest".to_string()]);
        dims.insert("rust".to_string(), vec!["stable".to_string()]);

        let mut combo = HashMap::new();
        combo.insert("os".to_string(), "ubuntu-latest".to_string());
        combo.insert("rust".to_string(), "stable".to_string());

        let template = Job {
            name: "build".to_string(),
            condition: None,
            needs: None,
            runs_on: None,
            strategy: None,
            timeout: None,
            env: None,
            secrets: None,
            services: None,
            before: None,
            after: None,
            environment: None,
            artifacts_from: None,
            steps: vec![Step {
                name: "test".to_string(),
                description: None,
                image: None,
                run: Some(vec!["cargo test".to_string()]),
                shell: None,
                workdir: String::new(),
                env: None,
                secrets: None,
                continue_on_error: false,
                timeout: None,
                condition: None,
                uses: None,
                checkout: None,
                cache: None,
                artifact: None,
                retry: None,
            }],
        };

        let expanded = apply_combination(&template, &combo).unwrap();
        // Should contain both dimension values in sorted order
        assert!(expanded.name.starts_with("build-"));
        assert!(expanded.name.contains("stable"));
        assert!(expanded.name.contains("ubuntu-latest"));
    }

    #[test]
    fn test_apply_combination_env_injection() {
        let mut combo = HashMap::new();
        combo.insert("os".to_string(), "ubuntu-latest".to_string());
        combo.insert("rust".to_string(), "stable".to_string());

        let template = Job {
            name: "build".to_string(),
            condition: None,
            needs: None,
            runs_on: None,
            strategy: None,
            timeout: None,
            env: None,
            secrets: None,
            services: None,
            before: None,
            after: None,
            environment: None,
            artifacts_from: None,
            steps: vec![Step {
                name: "test".to_string(),
                description: None,
                image: None,
                run: Some(vec!["cargo test".to_string()]),
                shell: None,
                workdir: String::new(),
                env: None,
                secrets: None,
                continue_on_error: false,
                timeout: None,
                condition: None,
                uses: None,
                checkout: None,
                cache: None,
                artifact: None,
                retry: None,
            }],
        };

        let expanded = apply_combination(&template, &combo).unwrap();
        let env = expanded.env.unwrap();
        assert!(env.iter().any(|e| e.name == "MATRIX_OS"));
        assert!(env.iter().any(|e| e.name == "MATRIX_RUST"));
    }

    #[test]
    fn test_apply_combination_runs_on() {
        let mut combo = HashMap::new();
        combo.insert("runs-on".to_string(), "ubuntu-latest".to_string());

        let template = Job {
            name: "build".to_string(),
            condition: None,
            needs: None,
            runs_on: Some(RunsOn {
                labels: Some(vec!["original".to_string()]),
                group: None,
            }),
            strategy: None,
            timeout: None,
            env: None,
            secrets: None,
            services: None,
            before: None,
            after: None,
            environment: None,
            artifacts_from: None,
            steps: vec![Step {
                name: "test".to_string(),
                description: None,
                image: None,
                run: Some(vec!["cargo test".to_string()]),
                shell: None,
                workdir: String::new(),
                env: None,
                secrets: None,
                continue_on_error: false,
                timeout: None,
                condition: None,
                uses: None,
                checkout: None,
                cache: None,
                artifact: None,
                retry: None,
            }],
        };

        let expanded = apply_combination(&template, &combo).unwrap();
        let runs_on = expanded.runs_on.unwrap();
        assert_eq!(runs_on.labels.unwrap(), vec!["ubuntu-latest".to_string()]);
    }

    #[test]
    fn test_expand_full_pipeline() {
        let yaml = r#"
version: '1'
jobs:
  - name: build
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable]
    runs-on:
      labels: ["${{ matrix.os }}"]
    steps:
      - name: test
        run: ["cargo +${{ matrix.rust }} test"]
"#;
        let pipeline = make_test_pipeline(yaml);
        let expanded = expand_matrix(&pipeline).unwrap();
        assert_eq!(expanded.jobs.len(), 2);
        assert!(expanded.jobs[0].name.contains("ubuntu-latest"));
        assert!(expanded.jobs[1].name.contains("macos-latest"));
        // Strategy should be removed after expansion
        assert!(expanded.jobs[0].strategy.is_none());
    }

    #[test]
    fn test_expand_with_exclude() {
        let yaml = r#"
version: '1'
jobs:
  - name: build
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable, nightly]
        exclude:
          - os: macos-latest
            rust: nightly
    steps:
      - name: test
        run: ["cargo test"]
"#;
        let pipeline = make_test_pipeline(yaml);
        let expanded = expand_matrix(&pipeline).unwrap();
        // 2*2 = 4 - 1 excluded = 3
        assert_eq!(expanded.jobs.len(), 3);
    }

    #[test]
    fn test_expand_with_include() {
        let yaml = r#"
version: '1'
jobs:
  - name: build
    strategy:
      matrix:
        os: [ubuntu-latest]
        rust: [stable]
        include:
          - os: windows-latest
            rust: stable
    steps:
      - name: test
        run: ["cargo test"]
"#;
        let pipeline = make_test_pipeline(yaml);
        let expanded = expand_matrix(&pipeline).unwrap();
        // 1*1 = 1 + 1 included = 2
        assert_eq!(expanded.jobs.len(), 2);
        assert!(
            expanded
                .jobs
                .iter()
                .any(|j| j.name.contains("windows-latest"))
        );
    }

    #[test]
    fn test_expand_no_matrix_unchanged() {
        let yaml = r#"
version: '1'
jobs:
  - name: build
    steps:
      - name: test
        run: ["cargo test"]
"#;
        let pipeline = make_test_pipeline(yaml);
        let expanded = expand_matrix(&pipeline).unwrap();
        assert_eq!(expanded.jobs.len(), 1);
        assert_eq!(expanded.jobs[0].name, "build");
    }

    #[test]
    fn test_expand_preserves_needs() {
        let yaml = r#"
version: '1'
jobs:
  - name: lint
    steps:
      - name: lint
        run: ["cargo clippy"]
  - name: build
    needs: [lint]
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - name: build
        run: ["cargo build"]
"#;
        let pipeline = make_test_pipeline(yaml);
        let expanded = expand_matrix(&pipeline).unwrap();
        assert_eq!(expanded.jobs.len(), 3); // lint + 2 matrix builds
        // Matrix builds should inherit needs
        for job in &expanded.jobs[1..] {
            assert_eq!(job.needs.as_deref(), Some(&["lint".to_string()][..]));
        }
    }

    #[test]
    fn test_apply_combination_step_expr_resolution() {
        let mut combo = HashMap::new();
        combo.insert("rust".to_string(), "nightly".to_string());

        let template = Job {
            name: "build".to_string(),
            condition: None,
            needs: None,
            runs_on: None,
            strategy: None,
            timeout: None,
            env: None,
            secrets: None,
            services: None,
            before: None,
            after: None,
            environment: None,
            artifacts_from: None,
            steps: vec![Step {
                name: "test".to_string(),
                description: None,
                image: None,
                run: Some(vec!["cargo +${{ matrix.rust }} test".to_string()]),
                shell: None,
                workdir: String::new(),
                env: None,
                secrets: None,
                continue_on_error: false,
                timeout: None,
                condition: Some("${{ matrix.rust == 'nightly' }}".to_string()),
                uses: None,
                checkout: None,
                cache: None,
                artifact: None,
                retry: None,
            }],
        };

        let expanded = apply_combination(&template, &combo).unwrap();
        assert_eq!(
            expanded.steps[0].run.as_ref().unwrap()[0],
            "cargo +nightly test"
        );
        assert_eq!(
            expanded.steps[0].condition.as_deref(),
            Some("${{ matrix.rust == 'nightly' }}") // raw expr not resolved (only matrix refs)
        );
    }
}

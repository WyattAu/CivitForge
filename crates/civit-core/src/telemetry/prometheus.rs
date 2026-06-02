#![forbid(unsafe_code)]

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

#[derive(Debug, Clone)]
pub struct MetricSample {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub metric_type: MetricType,
    pub help: Option<String>,
}

pub struct PrometheusExporter {
    prefix: String,
}

impl PrometheusExporter {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    pub fn render(&self, samples: &[MetricSample]) -> String {
        let mut output = String::new();
        let mut seen_types: HashMap<String, (MetricType, Option<String>)> = HashMap::new();

        for sample in samples {
            if !seen_types.contains_key(&sample.name) {
                seen_types.insert(
                    sample.name.clone(),
                    (sample.metric_type.clone(), sample.help.clone()),
                );
            }
        }

        for (name, (mtype, help)) in &seen_types {
            let full_name = if self.prefix.is_empty() {
                name.clone()
            } else {
                format!("{}_{}", self.prefix, name)
            };

            if let Some(h) = help {
                output.push_str(&format!("# HELP {full_name} {h}\n"));
            }
            output.push_str(&format!(
                "# TYPE {} {}\n",
                full_name,
                match mtype {
                    MetricType::Counter => "counter",
                    MetricType::Gauge => "gauge",
                    MetricType::Histogram => "histogram",
                    MetricType::Summary => "summary",
                }
            ));
        }

        for sample in samples {
            let full_name = if self.prefix.is_empty() {
                sample.name.clone()
            } else {
                format!("{}_{}", self.prefix, sample.name)
            };

            if sample.labels.is_empty() {
                output.push_str(&format!("{} {}\n", full_name, sample.value));
            } else {
                let label_str: Vec<String> = sample
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{k}=\"{v}\""))
                    .collect();
                let labels_joined = label_str.join(",");
                output.push_str(&format!(
                    "{}{{{}}} {}\n",
                    full_name, labels_joined, sample.value
                ));
            }
        }

        output
    }

    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

impl Default for PrometheusExporter {
    fn default() -> Self {
        Self::new("civitforge")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(name: &str, value: f64, mtype: MetricType) -> MetricSample {
        MetricSample {
            name: name.to_string(),
            value,
            labels: HashMap::new(),
            metric_type: mtype,
            help: None,
        }
    }

    fn sample_with_help(name: &str, value: f64, mtype: MetricType, help: &str) -> MetricSample {
        MetricSample {
            name: name.to_string(),
            value,
            labels: HashMap::new(),
            metric_type: mtype,
            help: Some(help.to_string()),
        }
    }

    fn sample_with_labels(
        name: &str,
        value: f64,
        mtype: MetricType,
        labels: HashMap<String, String>,
    ) -> MetricSample {
        MetricSample {
            name: name.to_string(),
            value,
            labels,
            metric_type: mtype,
            help: None,
        }
    }

    #[test]
    fn test_default_prefix() {
        let exporter = PrometheusExporter::default();
        assert_eq!(exporter.prefix(), "civitforge");
    }

    #[test]
    fn test_custom_prefix() {
        let exporter = PrometheusExporter::new("myapp");
        assert_eq!(exporter.prefix(), "myapp");
    }

    #[test]
    fn test_empty_prefix() {
        let exporter = PrometheusExporter::new("");
        assert_eq!(exporter.prefix(), "");
    }

    #[test]
    fn test_render_single_counter() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample("requests_total", 42.0, MetricType::Counter)]);
        assert!(output.contains("# TYPE app_requests_total counter"));
        assert!(output.contains("app_requests_total 42"));
    }

    #[test]
    fn test_render_gauge() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample("temperature", 36.6, MetricType::Gauge)]);
        assert!(output.contains("# TYPE app_temperature gauge"));
        assert!(output.contains("app_temperature 36.6"));
    }

    #[test]
    fn test_render_histogram() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample("duration_seconds", 0.5, MetricType::Histogram)]);
        assert!(output.contains("# TYPE app_duration_seconds histogram"));
    }

    #[test]
    fn test_render_summary() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample("request_size", 1024.0, MetricType::Summary)]);
        assert!(output.contains("# TYPE app_request_size summary"));
    }

    #[test]
    fn test_render_with_help() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample_with_help(
            "up",
            1.0,
            MetricType::Gauge,
            "Server up status",
        )]);
        assert!(output.contains("# HELP app_up Server up status"));
        assert!(output.contains("# TYPE app_up gauge"));
    }

    #[test]
    fn test_render_no_prefix() {
        let exporter = PrometheusExporter::new("");
        let output = exporter.render(&[sample("cpu_usage", 75.5, MetricType::Gauge)]);
        assert!(output.contains("# TYPE cpu_usage gauge"));
        assert!(output.contains("cpu_usage 75.5"));
    }

    #[test]
    fn test_render_with_labels() {
        let exporter = PrometheusExporter::new("app");
        let mut labels = HashMap::new();
        labels.insert("method".to_string(), "GET".to_string());
        labels.insert("path".to_string(), "/api".to_string());
        let output = exporter.render(&[sample_with_labels(
            "http_requests",
            10.0,
            MetricType::Counter,
            labels,
        )]);
        assert!(output.contains("method=\"GET\""));
        assert!(output.contains("path=\"/api\""));
        assert!(output.contains("app_http_requests{"));
        assert!(output.contains("} 10"));
    }

    #[test]
    fn test_render_multiple_samples() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[
            sample("requests_total", 100.0, MetricType::Counter),
            sample("active_connections", 5.0, MetricType::Gauge),
        ]);
        assert!(output.contains("# TYPE app_requests_total counter"));
        assert!(output.contains("# TYPE app_active_connections gauge"));
        assert!(output.contains("app_requests_total 100"));
        assert!(output.contains("app_active_connections 5"));
    }

    #[test]
    fn test_render_duplicate_names_use_first_type() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[
            sample("requests_total", 100.0, MetricType::Counter),
            sample("requests_total", 200.0, MetricType::Counter),
        ]);
        let type_count = output.matches("# TYPE").count();
        assert_eq!(type_count, 1);
    }

    #[test]
    fn test_render_empty_samples() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[]);
        assert!(output.is_empty());
    }

    #[test]
    fn test_render_negative_value() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample("temperature_delta", -2.5, MetricType::Gauge)]);
        assert!(output.contains("app_temperature_delta -2.5"));
    }

    #[test]
    fn test_render_zero_value() {
        let exporter = PrometheusExporter::new("app");
        let output = exporter.render(&[sample("errors_total", 0.0, MetricType::Counter)]);
        assert!(output.contains("app_errors_total 0"));
    }

    #[test]
    fn test_metric_type_equality() {
        assert_eq!(MetricType::Counter, MetricType::Counter);
        assert_ne!(MetricType::Counter, MetricType::Gauge);
    }

    #[test]
    fn test_metric_sample_clone() {
        let s = sample("test", 1.0, MetricType::Counter);
        let s2 = s.clone();
        assert_eq!(s.name, s2.name);
        assert_eq!(s.value, s2.value);
    }

    #[test]
    fn test_exporter_clone_prefix() {
        let exporter = PrometheusExporter::new("test");
        assert_eq!(exporter.prefix().len(), 4);
    }

    #[test]
    fn test_render_label_ordering() {
        let exporter = PrometheusExporter::new("app");
        let mut labels = HashMap::new();
        labels.insert("z".to_string(), "last".to_string());
        labels.insert("a".to_string(), "first".to_string());
        let output = exporter.render(&[sample_with_labels(
            "ordered",
            1.0,
            MetricType::Gauge,
            labels,
        )]);
        assert!(output.contains("z=\"last\""));
        assert!(output.contains("a=\"first\""));
    }
}

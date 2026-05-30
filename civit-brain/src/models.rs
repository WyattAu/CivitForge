#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
}

impl CodeEntity {
    pub fn fully_qualified_name(&self) -> String {
        format!("{}:{}:{}", self.file_path, self.start_line, self.name)
    }

    pub fn is_within_range(&self, start: usize, end: usize) -> bool {
        self.start_line >= start && self.end_line <= end
    }

    pub fn overlaps(&self, start: usize, end: usize) -> bool {
        self.start_line <= end && self.end_line >= start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(id: &str, name: &str, start: usize, end: usize) -> CodeEntity {
        CodeEntity {
            id: id.into(),
            entity_type: "Function".into(),
            name: name.into(),
            file_path: "src/main.rs".into(),
            start_line: start,
            end_line: end,
        }
    }

    #[test]
    fn test_fully_qualified_name() {
        let entity = make_entity("e1", "main", 1, 10);
        let fqn = entity.fully_qualified_name();
        assert_eq!(fqn, "src/main.rs:1:main");
    }

    #[test]
    fn test_is_within_range() {
        let entity = make_entity("e1", "foo", 5, 15);
        assert!(entity.is_within_range(5, 15));
        assert!(entity.is_within_range(1, 20));
        assert!(!entity.is_within_range(1, 4));
        assert!(!entity.is_within_range(16, 20));
    }

    #[test]
    fn test_overlaps() {
        let entity = make_entity("e1", "foo", 5, 15);
        assert!(entity.overlaps(1, 10));
        assert!(entity.overlaps(10, 20));
        assert!(entity.overlaps(5, 15));
        assert!(!entity.overlaps(1, 4));
        assert!(!entity.overlaps(16, 20));
    }

    #[test]
    fn test_serialization() {
        let entity = make_entity("e1", "bar", 1, 5);
        let json = serde_json::to_string(&entity).unwrap();
        let de: CodeEntity = serde_json::from_str(&json).unwrap();
        assert_eq!(de.name, "bar");
        assert_eq!(de.start_line, 1);
        assert_eq!(de.end_line, 5);
    }
}

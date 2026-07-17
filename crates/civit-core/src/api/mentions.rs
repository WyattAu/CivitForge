#![forbid(unsafe_code)]

use regex::Regex;

fn mention_regex() -> Regex {
    Regex::new(r"(?m)@([A-Za-z0-9_]{1,39})").expect("valid regex pattern")
}

fn cross_ref_regex() -> Regex {
    Regex::new(r"(?m)#(\d{1,10})\b").expect("valid regex pattern")
}

/// Parse @username mentions from a comment body.
pub fn parse_mentions(body: &str) -> Vec<String> {
    mention_regex()
        .captures_iter(body)
        .map(|c| c[1].to_string())
        .collect()
}

/// Parse #NNN cross-references from a comment body.
pub fn parse_cross_references(body: &str) -> Vec<i32> {
    cross_ref_regex()
        .captures_iter(body)
        .filter_map(|c| c[1].parse::<i32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mentions_basic() {
        let mentions = parse_mentions("Hello @alice and @bob");
        assert_eq!(mentions, vec!["alice", "bob"]);
    }

    #[test]
    fn test_parse_mentions_none() {
        assert!(parse_mentions("no mentions here").is_empty());
    }

    #[test]
    fn test_parse_mentions_underscore() {
        let mentions = parse_mentions("cc @user_name_here");
        assert_eq!(mentions, vec!["user_name_here"]);
    }

    #[test]
    fn test_parse_mentions_multiline() {
        let body = "Hey @alice\nAlso @bob123";
        let mentions = parse_mentions(body);
        assert_eq!(mentions, vec!["alice", "bob123"]);
    }

    #[test]
    fn test_parse_cross_refs_basic() {
        let refs = parse_cross_references("Fixes #123 and #456");
        assert_eq!(refs, vec![123, 456]);
    }

    #[test]
    fn test_parse_cross_refs_none() {
        assert!(parse_cross_references("no refs here").is_empty());
    }

    #[test]
    fn test_parse_cross_refs_multiline() {
        let body = "Closes #1\nAlso #99";
        let refs = parse_cross_references(body);
        assert_eq!(refs, vec![1, 99]);
    }

    #[test]
    fn test_parse_cross_refs_boundary() {
        let refs = parse_cross_references("#12345678901 is too many digits");
        assert!(refs.is_empty());
    }

    #[test]
    fn test_parse_mentions_dedup() {
        let mentions = parse_mentions("@alice hi @alice again");
        assert_eq!(mentions, vec!["alice", "alice"]);
    }

    #[test]
    fn test_parse_cross_refs_dedup() {
        let refs = parse_cross_references("fixes #1 and also #1");
        assert_eq!(refs, vec![1, 1]);
    }
}

#![forbid(unsafe_code)]

use tantivy::Term;
use tantivy::query::{BooleanQuery, FuzzyTermQuery, Occur, PhraseQuery, Query, TermQuery};
use tantivy::schema::{Field, Schema};

#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    pub terms: Vec<String>,
    pub phrases: Vec<Vec<String>>,
    pub field_terms: Vec<(String, String)>,
    pub language_filter: Option<String>,
    pub repo_id_filter: Option<String>,
    pub negated_terms: Vec<String>,
}

#[derive(Debug, Clone)]
enum QueryToken {
    Word(String),
    Phrase(String),
    And,
    Or,
    Not,
}

fn tokenize(input: &str) -> Vec<QueryToken> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            chars.next();
            let mut phrase = String::new();
            while let Some(&c) = chars.peek() {
                if c == '"' {
                    chars.next();
                    break;
                }
                phrase.push(c);
                chars.next();
            }
            tokens.push(QueryToken::Phrase(phrase));
            continue;
        }
        if ch == '-'
            && let Some(next) = chars.clone().nth(1)
            && next.is_alphanumeric()
        {
            chars.next();
            continue;
        }
        if ch == '+' {
            chars.next();
            continue;
        }
        let mut word = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_whitespace() || c == '"' {
                break;
            }
            word.push(c);
            chars.next();
        }
        let trimmed = word.trim_end_matches(':');
        if trimmed == "AND" {
            tokens.push(QueryToken::And);
        } else if trimmed == "OR" {
            tokens.push(QueryToken::Or);
        } else if trimmed == "NOT" {
            tokens.push(QueryToken::Not);
        } else if let Some((field, value)) = word.split_once(':') {
            if !field.is_empty() && !value.is_empty() {
                let lower = field.to_lowercase();
                if lower == "lang" || lower == "language" {
                    tokens.push(QueryToken::Phrase(format!("__lang__:{value}")));
                } else if lower == "repo" || lower == "repo_id" {
                    tokens.push(QueryToken::Phrase(format!("__repo__:{value}")));
                } else {
                    tokens.push(QueryToken::Word(word));
                }
            } else {
                tokens.push(QueryToken::Word(word));
            }
        } else {
            tokens.push(QueryToken::Word(word));
        }
    }
    tokens
}

pub fn parse_search_query(input: &str) -> ParsedQuery {
    let tokens = tokenize(input);
    let mut parsed = ParsedQuery::default();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            QueryToken::Phrase(s) => {
                if let Some(rest) = s.strip_prefix("__lang__:") {
                    parsed.language_filter = Some(rest.to_string());
                } else if let Some(rest) = s.strip_prefix("__repo__:") {
                    parsed.repo_id_filter = Some(rest.to_string());
                } else {
                    let words: Vec<String> =
                        s.split_whitespace().map(|w| w.to_lowercase()).collect();
                    if words.len() > 1 {
                        parsed.phrases.push(words);
                    } else if !words.is_empty() {
                        parsed.terms.push(words.into_iter().next().unwrap());
                    }
                }
            }
            QueryToken::Word(w) => {
                parsed.terms.push(w.to_lowercase());
            }
            QueryToken::Not => {
                i += 1;
                if let Some(QueryToken::Word(w)) = tokens.get(i) {
                    parsed.negated_terms.push(w.to_lowercase());
                }
            }
            QueryToken::And | QueryToken::Or => {}
        }
        i += 1;
    }
    parsed
}

pub struct SearchQueryBuilder {
    query_str: String,
    repo_id_filter: Option<String>,
    language_filter: Option<String>,
    fuzzy: bool,
    fuzzy_distance: u8,
}

impl SearchQueryBuilder {
    pub fn new(query: &str) -> Self {
        Self {
            query_str: query.to_string(),
            repo_id_filter: None,
            language_filter: None,
            fuzzy: true,
            fuzzy_distance: 2,
        }
    }

    pub fn repo_id_filter(mut self, filter: Option<String>) -> Self {
        self.repo_id_filter = filter;
        self
    }

    pub fn language_filter(mut self, filter: Option<String>) -> Self {
        self.language_filter = filter;
        self
    }

    pub fn fuzzy(mut self, enabled: bool) -> Self {
        self.fuzzy = enabled;
        self
    }

    pub fn fuzzy_distance(mut self, distance: u8) -> Self {
        self.fuzzy_distance = distance;
        self
    }

    pub fn build(
        self,
        schema: &Schema,
        content_field: Field,
        repo_id_field: Field,
        language_field: Field,
    ) -> crate::error::Result<(Box<dyn Query>, String)> {
        let parsed = parse_search_query(&self.query_str);
        let repo_filter = self.repo_id_filter.or(parsed.repo_id_filter.clone());
        let lang_filter = self.language_filter.or(parsed.language_filter.clone());
        let has_filters = repo_filter.is_some() || lang_filter.is_some();

        // Split each term into sub-terms on non-alphanumeric chars for code search
        let mut content_clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

        for phrase in &parsed.phrases {
            if phrase.len() >= 2 {
                let terms: Vec<Term> = phrase
                    .iter()
                    .map(|w| Term::from_field_text(content_field, w))
                    .collect();
                let pq = PhraseQuery::new(terms);
                content_clauses.push((Occur::Should, Box::new(pq)));
            }
        }

        for term_str in &parsed.terms {
            let parts: Vec<&str> = term_str
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .collect();
            let was_split = parts.len() > 1;
            let sub_terms: Vec<String> = if was_split {
                parts.into_iter().map(|s| s.to_lowercase()).collect()
            } else {
                vec![term_str.clone()]
            };

            for sub_term in &sub_terms {
                if let Some((field_name, value)) = sub_term.split_once(':')
                    && let Ok(f) = schema.get_field(field_name)
                {
                    let term = Term::from_field_text(f, value);
                    let tq = TermQuery::new(term, Default::default());
                    content_clauses.push((Occur::Should, Box::new(tq)));
                    continue;
                }
                let term = Term::from_field_text(content_field, sub_term);
                if self.fuzzy && sub_term.len() > 3 {
                    let fq = FuzzyTermQuery::new(term, self.fuzzy_distance, true);
                    // When original term was split into sub-terms, all must match (Must)
                    // to ensure we find the specific multi-word identifier, not just one word
                    let occur = if was_split {
                        Occur::Must
                    } else {
                        Occur::Should
                    };
                    content_clauses.push((occur, Box::new(fq)));
                } else {
                    let tq = TermQuery::new(term, Default::default());
                    let occur = if was_split {
                        Occur::Must
                    } else {
                        Occur::Should
                    };
                    content_clauses.push((occur, Box::new(tq)));
                }
            }
        }

        for neg in &parsed.negated_terms {
            let term = Term::from_field_text(content_field, neg);
            let tq = TermQuery::new(term, Default::default());
            content_clauses.push((Occur::MustNot, Box::new(tq)));
        }

        if content_clauses.is_empty() && !has_filters {
            return Err(crate::error::CoreError::Search("empty query".into()));
        }

        let query: Box<dyn Query> = if has_filters {
            // When filters exist, wrap content clauses in a Must inner BooleanQuery
            // so content matching is required alongside filter matching
            let mut outer_clauses: Vec<(Occur, Box<dyn Query>)> = Vec::new();

            if !content_clauses.is_empty() {
                let content_query: Box<dyn Query> = if content_clauses.len() == 1 {
                    content_clauses.into_iter().next().unwrap().1
                } else {
                    Box::new(BooleanQuery::new(content_clauses))
                };
                outer_clauses.push((Occur::Must, content_query));
            }

            if let Some(ref repo) = repo_filter {
                let term = Term::from_field_text(repo_id_field, repo);
                let tq = TermQuery::new(term, Default::default());
                outer_clauses.push((Occur::Must, Box::new(tq)));
            }

            if let Some(ref lang) = lang_filter {
                let term = Term::from_field_text(language_field, lang);
                let tq = TermQuery::new(term, Default::default());
                outer_clauses.push((Occur::Must, Box::new(tq)));
            }

            if outer_clauses.len() == 1 {
                outer_clauses.into_iter().next().unwrap().1
            } else {
                Box::new(BooleanQuery::new(outer_clauses))
            }
        } else {
            // No filters: content clauses use Should (at least one must match)
            if content_clauses.len() == 1 {
                content_clauses.into_iter().next().unwrap().1
            } else {
                Box::new(BooleanQuery::new(content_clauses))
            }
        };

        Ok((query, self.query_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::schema::{STORED, TEXT};

    #[test]
    fn test_parse_simple_terms() {
        let parsed = parse_search_query("fn main hello");
        assert_eq!(parsed.terms, vec!["fn", "main", "hello"]);
        assert!(parsed.phrases.is_empty());
    }

    #[test]
    fn test_parse_phrase() {
        let parsed = parse_search_query("\"fn main\"");
        assert!(parsed.terms.is_empty());
        assert_eq!(parsed.phrases, vec![vec!["fn", "main"]]);
    }

    #[test]
    fn test_parse_mixed_terms_and_phrases() {
        let parsed = parse_search_query("hello \"world peace\" goodbye");
        assert_eq!(parsed.terms, vec!["hello", "goodbye"]);
        assert_eq!(parsed.phrases, vec![vec!["world", "peace"]]);
    }

    #[test]
    fn test_parse_negation() {
        let parsed = parse_search_query("hello NOT goodbye");
        assert_eq!(parsed.terms, vec!["hello"]);
        assert_eq!(parsed.negated_terms, vec!["goodbye"]);
    }

    #[test]
    fn test_parse_language_filter() {
        let parsed = parse_search_query("main lang:rust");
        assert_eq!(parsed.language_filter.as_deref(), Some("rust"));
        assert_eq!(parsed.terms, vec!["main"]);
    }

    #[test]
    fn test_parse_language_filter_long_form() {
        let parsed = parse_search_query("impl language:go");
        assert_eq!(parsed.language_filter.as_deref(), Some("go"));
    }

    #[test]
    fn test_parse_repo_filter() {
        let parsed = parse_search_query("main repo:myrepo");
        assert_eq!(parsed.repo_id_filter.as_deref(), Some("myrepo"));
        assert_eq!(parsed.terms, vec!["main"]);
    }

    #[test]
    fn test_parse_empty() {
        let parsed = parse_search_query("");
        assert!(parsed.terms.is_empty());
        assert!(parsed.phrases.is_empty());
    }

    #[test]
    fn test_parse_whitespace_only() {
        let parsed = parse_search_query("   ");
        assert!(parsed.terms.is_empty());
    }

    #[test]
    fn test_parse_field_term() {
        let parsed = parse_search_query("path:src/main.rs");
        assert_eq!(parsed.terms, vec!["path:src/main.rs"]);
    }

    #[test]
    fn test_parse_case_insensitive_terms() {
        let parsed = parse_search_query("Hello World FOO");
        assert_eq!(parsed.terms, vec!["hello", "world", "foo"]);
    }

    fn test_schema() -> Schema {
        let mut builder = Schema::builder();
        builder.add_text_field("content", TEXT);
        builder.add_text_field("repo_id", TEXT | STORED);
        builder.add_text_field("language", TEXT | STORED);
        builder.build()
    }

    fn test_schema_fields(schema: &Schema) -> (Field, Field, Field) {
        let content_f = schema.get_field("content").unwrap();
        let repo_f = schema.get_field("repo_id").unwrap();
        let lang_f = schema.get_field("language").unwrap();
        (content_f, repo_f, lang_f)
    }

    #[test]
    fn test_builder_basic() {
        let schema = test_schema();
        let (content_f, repo_f, lang_f) = test_schema_fields(&schema);

        let (query, _) = SearchQueryBuilder::new("hello world")
            .build(&schema, content_f, repo_f, lang_f)
            .unwrap();
        let _ = query;
    }

    #[test]
    fn test_builder_with_repo_filter() {
        let schema = test_schema();
        let (content_f, repo_f, lang_f) = test_schema_fields(&schema);

        let (query, _) = SearchQueryBuilder::new("test")
            .repo_id_filter(Some("repo-1".into()))
            .build(&schema, content_f, repo_f, lang_f)
            .unwrap();
        let _ = query;
    }

    #[test]
    fn test_builder_with_language_filter() {
        let schema = test_schema();
        let (content_f, repo_f, lang_f) = test_schema_fields(&schema);

        let (query, _) = SearchQueryBuilder::new("impl")
            .language_filter(Some("rust".into()))
            .build(&schema, content_f, repo_f, lang_f)
            .unwrap();
        let _ = query;
    }

    #[test]
    fn test_builder_empty_query_fails() {
        let schema = test_schema();
        let (content_f, repo_f, lang_f) = test_schema_fields(&schema);

        let result = SearchQueryBuilder::new("").build(&schema, content_f, repo_f, lang_f);
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_fuzzy_disabled() {
        let schema = test_schema();
        let (content_f, repo_f, lang_f) = test_schema_fields(&schema);

        let (query, _) = SearchQueryBuilder::new("hello")
            .fuzzy(false)
            .build(&schema, content_f, repo_f, lang_f)
            .unwrap();
        let _ = query;
    }

    #[test]
    fn test_parse_and_keywords_ignored() {
        let parsed = parse_search_query("hello AND world OR goodbye");
        assert_eq!(parsed.terms, vec!["hello", "world", "goodbye"]);
    }
}

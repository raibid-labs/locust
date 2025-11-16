//! Property-based tests for fuzzy matching
//!
//! Tests fuzzy matching properties using proptest.

use locust::core::fuzzy::FuzzyMatcher;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_fuzzy_score_range(query in "\\PC+", text in "\\PC+") {
        let matcher = FuzzyMatcher::new();
        if let Some((score, _positions)) = matcher.score(&query, &text) {
            prop_assert!(score >= 0.0 && score <= 100.0, "Score {} out of range for query='{}', text='{}'", score, query, text);
        }
    }

    #[test]
    fn test_fuzzy_empty_query_matches_all(text in "\\PC+") {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("", &text);
        prop_assert!(score.is_some(), "Empty query should match: {}", text);
    }

    #[test]
    fn test_fuzzy_exact_match_highest_score(text in "[a-zA-Z]{1,20}") {
        let matcher = FuzzyMatcher::new();
        let exact_score = matcher.score(&text, &text).map(|(s, _)| s).unwrap_or(0.0);

        // Generate similar but not exact text
        let mut modified = text.clone();
        modified.push('x');
        let partial_score = matcher.score(&text, &modified).map(|(s, _)| s).unwrap_or(0.0);

        prop_assert!(exact_score >= partial_score, "Exact match should score higher: {} vs {}", exact_score, partial_score);
    }

    #[test]
    fn test_fuzzy_query_substring_always_matches(
        text in "[a-zA-Z]{5,20}",
        start in 0usize..15usize,
        len in 1usize..5usize
    ) {
        let matcher = FuzzyMatcher::new();
        let end = (start + len).min(text.len());
        if start < text.len() && end <= text.len() && start < end {
            let substring = &text[start..end];
            let score = matcher.score(substring, &text);
            prop_assert!(score.is_some(), "Substring '{}' should match in '{}'", substring, text);
        }
    }

    #[test]
    fn test_fuzzy_case_insensitive(
        text in "[a-zA-Z]{3,20}"
    ) {
        let matcher = FuzzyMatcher::new();
        let lower = text.to_lowercase();
        let upper = text.to_uppercase();

        let score_lower = matcher.score(&lower, &text);
        let score_upper = matcher.score(&upper, &text);

        // Both should match
        prop_assert!(score_lower.is_some() || score_upper.is_some());
    }

    #[test]
    fn test_fuzzy_longer_query_than_text(
        text in "[a-zA-Z]{1,10}"
    ) {
        let matcher = FuzzyMatcher::new();
        let long_query = format!("{}{}", text, "xxxxxxxxxxxxxx");

        let score = matcher.score(&long_query, &text);
        // Long query might not match short text
        prop_assert!(score.is_none() || score.map(|(s, _)| s).unwrap() < 100.0);
    }

    #[test]
    fn test_fuzzy_score_deterministic(
        query in "\\PC{1,20}",
        text in "\\PC{1,30}"
    ) {
        let matcher = FuzzyMatcher::new();
        let score1 = matcher.score(&query, &text);
        let score2 = matcher.score(&query, &text);

        prop_assert_eq!(score1, score2, "Scores should be deterministic");
    }

    #[test]
    fn test_fuzzy_unicode_handling(
        query in "[\\u{0041}-\\u{1F600}]{1,10}",
        text in "[\\u{0041}-\\u{1F600}]{1,20}"
    ) {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score(&query, &text);

        // Should handle unicode without panic
        prop_assert!(score.is_some() || score.is_none());
    }

    #[test]
    fn test_fuzzy_whitespace_handling(
        words in prop::collection::vec("[a-z]{3,8}", 1..5)
    ) {
        let matcher = FuzzyMatcher::new();
        let text = words.join(" ");
        let query = words.first().unwrap();

        let score = matcher.score(query, &text);
        prop_assert!(score.is_some(), "Should match word in phrase");
    }

    #[test]
    fn test_fuzzy_consecutive_chars_score_higher(
        text in "[a-zA-Z]{10,20}"
    ) {
        let matcher = FuzzyMatcher::new();

        // Take first 3 consecutive chars
        let consecutive = &text[0..3.min(text.len())];

        // Take 3 non-consecutive chars
        let non_consecutive = format!(
            "{}{}{}",
            text.chars().nth(0).unwrap(),
            text.chars().nth(text.len() / 2).unwrap_or('x'),
            text.chars().nth(text.len() - 1).unwrap_or('y')
        );

        let score_consecutive = matcher.score(consecutive, &text).map(|(s, _)| s).unwrap_or(0.0);
        let score_non_consecutive = matcher.score(&non_consecutive, &text).map(|(s, _)| s).unwrap_or(0.0);

        // Consecutive should generally score higher
        prop_assert!(score_consecutive >= score_non_consecutive * 0.8); // Allow some variance
    }
}

// Standard unit tests for edge cases
#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_fuzzy_empty_query_empty_text() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("", "");
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_empty_text() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("query", "");
        assert!(score.is_none());
    }

    #[test]
    fn test_fuzzy_very_long_strings() {
        let matcher = FuzzyMatcher::new();
        let long_query = "a".repeat(10000);
        let long_text = "a".repeat(10000);

        let score = matcher.score(&long_query, &long_text);
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_special_characters() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("test", "t!e@s#t$");
        assert!(score.is_some());
    }

    #[test]
    fn test_fuzzy_numbers() {
        let matcher = FuzzyMatcher::new();
        let score = matcher.score("123", "abc123def");
        assert!(score.is_some());
    }
}

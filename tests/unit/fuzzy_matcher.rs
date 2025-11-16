//! Comprehensive unit tests for fuzzy matching.

use locust::core::fuzzy::{FuzzyMatcher, Match};

#[test]
fn test_matcher_creation() {
    let matcher = FuzzyMatcher::new();
    assert!(matcher.score("test", "test").is_some());
}

#[test]
fn test_default_matcher() {
    let matcher = FuzzyMatcher::default();
    assert!(matcher.score("test", "test").is_some());
}

#[test]
fn test_empty_scenarios() {
    let matcher = FuzzyMatcher::new();

    // Empty query matches everything with score 0
    let result = matcher.score("", "anything");
    assert!(result.is_some());
    assert_eq!(result.unwrap().0, 0.0);

    // Empty text doesn't match non-empty query
    assert!(matcher.score("query", "").is_none());

    // Both empty
    let result = matcher.score("", "");
    assert!(result.is_some());
    assert_eq!(result.unwrap().0, 0.0);
}

#[test]
fn test_exact_matches() {
    let matcher = FuzzyMatcher::new();

    let test_cases = vec![
        ("a", "a"),
        ("test", "test"),
        ("hello world", "hello world"),
        ("123", "123"),
    ];

    for (query, text) in test_cases {
        let result = matcher.score(query, text);
        assert!(
            result.is_some(),
            "Failed to match '{}' in '{}'",
            query,
            text
        );
        let (score, positions) = result.unwrap();
        assert!(score > 0.0, "Score should be positive for exact match");
        assert_eq!(
            positions.len(),
            query.len(),
            "Should have position for each character"
        );
    }
}

#[test]
fn test_fuzzy_matches() {
    let matcher = FuzzyMatcher::new();

    let test_cases = vec![
        ("fm", "file_manager", true),
        ("fzm", "fuzzy_matcher", true),
        ("ht", "http_transport", true),
        ("ctrl", "controller", true),
        ("xyz", "abc", false),
        ("aaa", "ab", false),
    ];

    for (query, text, should_match) in test_cases {
        let result = matcher.score(query, text);
        assert_eq!(
            result.is_some(),
            should_match,
            "Match result for '{}' in '{}' should be {}",
            query,
            text,
            should_match
        );
    }
}

#[test]
fn test_case_sensitivity() {
    let case_insensitive = FuzzyMatcher::new();
    let case_sensitive = FuzzyMatcher::new().with_case_sensitive(true);

    // Case insensitive (default)
    assert!(case_insensitive.score("test", "TEST").is_some());
    assert!(case_insensitive.score("TeSt", "test").is_some());

    // Case sensitive
    assert!(case_sensitive.score("test", "TEST").is_none());
    assert!(case_sensitive.score("test", "test").is_some());
}

#[test]
fn test_scoring_consecutive_matches() {
    let matcher = FuzzyMatcher::new();

    // "abc" in "abc" (consecutive) vs "aXbXc" (non-consecutive)
    let (score_consecutive, _) = matcher.score("abc", "abc").unwrap();
    let (score_separated, _) = matcher.score("abc", "aXbXc").unwrap();

    assert!(
        score_consecutive > score_separated,
        "Consecutive matches should score higher: {} vs {}",
        score_consecutive,
        score_separated
    );
}

#[test]
fn test_scoring_first_character() {
    let matcher = FuzzyMatcher::new();

    // Match at start vs match in middle
    let (score_start, _) = matcher.score("t", "test").unwrap();
    let (score_middle, _) = matcher.score("t", "best").unwrap();

    assert!(
        score_start > score_middle,
        "First character match should score higher: {} vs {}",
        score_start,
        score_middle
    );
}

#[test]
fn test_scoring_word_boundaries() {
    let matcher = FuzzyMatcher::new();

    // Matches earlier in text should score higher
    let (score_early, _) = matcher.score("a", "abc").unwrap();
    let (score_late, _) = matcher.score("a", "xyzabcdefghijklmnop").unwrap();

    assert!(
        score_early > score_late,
        "Early matches should score higher: {} vs {}",
        score_early,
        score_late
    );
}

#[test]
fn test_position_accuracy() {
    let matcher = FuzzyMatcher::new();

    let (_, positions) = matcher.score("ace", "abcdef").unwrap();
    assert_eq!(positions.len(), 3);
    // Should match positions 0, 2, 4 (a, c, e)
    assert_eq!(positions, vec![0, 2, 4]);
}

#[test]
fn test_unicode_handling() {
    let matcher = FuzzyMatcher::new();

    // Single emoji match
    let result = matcher.score("🦀", "🦀🚀");
    assert!(result.is_some());
    let (_, positions) = result.unwrap();
    assert_eq!(positions.len(), 1);

    // Mixed ASCII and emoji
    let result = matcher.score("h🦀", "hello🦀world");
    assert!(result.is_some());
    let (_, positions) = result.unwrap();
    assert_eq!(positions.len(), 2);

    // Multi-byte characters
    let result = matcher.score("日本", "日本語");
    assert!(result.is_some());
    let (_, positions) = result.unwrap();
    assert_eq!(positions.len(), 2);
}

#[test]
fn test_find_matches_basic() {
    let matcher = FuzzyMatcher::new();
    let candidates = vec!["apple", "application", "apply", "orange"];

    let matches = matcher.find_matches("app", &candidates);

    // Should match "apple", "application", "apply"
    assert_eq!(matches.len(), 3);
    assert!(matches.iter().all(|m| m.score > 0.0));
}

#[test]
fn test_find_matches_sorting() {
    let matcher = FuzzyMatcher::new();
    let candidates = vec!["test", "best", "testing", "rest"];

    let matches = matcher.find_matches("test", &candidates);

    assert!(!matches.is_empty());
    // Exact match should be first
    assert_eq!(matches[0].text, "test");
    // Better matches should come before worse matches
    for i in 0..matches.len() - 1 {
        assert!(matches[i].score >= matches[i + 1].score);
    }
}

#[test]
fn test_find_matches_no_results() {
    let matcher = FuzzyMatcher::new();
    let candidates = vec!["apple", "banana", "cherry"];

    let matches = matcher.find_matches("xyz", &candidates);

    assert!(matches.is_empty());
}

#[test]
fn test_find_matches_empty_query() {
    let matcher = FuzzyMatcher::new();
    let candidates = vec!["apple", "banana", "cherry"];

    let matches = matcher.find_matches("", &candidates);

    // Empty query should match all with score 0
    assert_eq!(matches.len(), 3);
    assert!(matches.iter().all(|m| m.score == 0.0));
}

#[test]
fn test_match_struct_creation() {
    let match_result = Match::new(0, 42.5, vec![0, 1, 2], "test".to_string());

    assert_eq!(match_result.index, 0);
    assert_eq!(match_result.score, 42.5);
    assert_eq!(match_result.positions, vec![0, 1, 2]);
    assert_eq!(match_result.text, "test");
}

#[test]
fn test_custom_configuration() {
    let matcher = FuzzyMatcher::new()
        .with_consecutive_bonus(20.0)
        .with_word_boundary_bonus(10.0)
        .with_first_char_bonus(15.0);

    let (score, _) = matcher.score("test", "test").unwrap();
    // With higher bonuses, score should be higher
    assert!(score > 30.0);
}

#[test]
fn test_performance_many_candidates() {
    let matcher = FuzzyMatcher::new();

    // Generate 1000 candidates
    let candidates: Vec<String> = (0..1000).map(|i| format!("candidate_{:04}", i)).collect();
    let candidate_refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();

    let start = std::time::Instant::now();
    let matches = matcher.find_matches("cand", &candidate_refs);
    let elapsed = start.elapsed();

    // Should match all 1000 candidates
    assert_eq!(matches.len(), 1000);
    // Should complete in less than 10ms (target is <1ms, but being generous)
    assert!(
        elapsed.as_millis() < 10,
        "Performance regression: took {}ms",
        elapsed.as_millis()
    );
}

#[test]
fn test_edge_cases() {
    let matcher = FuzzyMatcher::new();

    // Single character
    assert!(matcher.score("a", "a").is_some());
    assert!(matcher.score("a", "b").is_none());

    // Query longer than text
    assert!(matcher.score("toolong", "short").is_none());

    // Repeated characters
    let result = matcher.score("aaa", "aaabbb");
    assert!(result.is_some());
    let (_, positions) = result.unwrap();
    assert_eq!(positions, vec![0, 1, 2]);

    // Special characters
    assert!(matcher.score("@#", "@#$%").is_some());

    // Numbers
    assert!(matcher.score("123", "123456").is_some());
}

#[test]
fn test_real_world_scenarios() {
    let matcher = FuzzyMatcher::new();

    // File paths
    let candidates = vec![
        "src/core/fuzzy.rs",
        "src/core/mod.rs",
        "tests/unit/fuzzy_matcher.rs",
        "benches/fuzzy_matching.rs",
    ];

    let matches = matcher.find_matches("fuzz", &candidates);
    assert_eq!(matches.len(), 3);
    assert!(matches.iter().any(|m| m.text.contains("fuzzy.rs")));

    // Commands
    let commands = vec!["open_file", "save_file", "close_window", "find_replace"];
    let matches = matcher.find_matches("of", &commands);
    assert!(matches.iter().any(|m| m.text == "open_file"));

    // Variable names
    let vars = vec!["userController", "userConfig", "userData", "systemConfig"];
    let matches = matcher.find_matches("uc", &vars);
    assert!(matches.len() >= 2);
}

#[test]
fn test_match_positions_for_highlighting() {
    let matcher = FuzzyMatcher::new();
    let text = "file_manager";
    let (_, positions) = matcher.score("fm", text).unwrap();

    // Verify we can use positions for highlighting
    for &pos in &positions {
        assert!(pos < text.len(), "Position out of bounds");
        // Check that we're pointing to valid UTF-8 boundaries
        assert!(text.is_char_boundary(pos));
    }
}

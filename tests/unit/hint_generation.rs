//! Unit tests for hint generation algorithm.

use locust::core::targets::{NavTarget, TargetPriority};
use locust::plugins::nav::hints::{Hint, HintGenerator, HintMatcher};
use ratatui::layout::Rect;

#[test]
fn test_hint_generation_sequential() {
    let mut generator = HintGenerator::new("asdf".to_string());

    let targets = vec![
        NavTarget::new(1, Rect::new(0, 0, 10, 1)),
        NavTarget::new(2, Rect::new(0, 2, 10, 1)),
        NavTarget::new(3, Rect::new(0, 4, 10, 1)),
        NavTarget::new(4, Rect::new(0, 6, 10, 1)),
    ];

    let hints = generator.generate(&targets);

    assert_eq!(hints.len(), 4);
    assert_eq!(hints[0].text, "a");
    assert_eq!(hints[1].text, "s");
    assert_eq!(hints[2].text, "d");
    assert_eq!(hints[3].text, "f");
}

#[test]
fn test_hint_generation_two_char_hints() {
    let mut generator = HintGenerator::new("ab".to_string());

    let targets = vec![
        NavTarget::new(1, Rect::new(0, 0, 10, 1)),
        NavTarget::new(2, Rect::new(0, 2, 10, 1)),
        NavTarget::new(3, Rect::new(0, 4, 10, 1)),
        NavTarget::new(4, Rect::new(0, 6, 10, 1)),
        NavTarget::new(5, Rect::new(0, 8, 10, 1)),
    ];

    let hints = generator.generate(&targets);

    assert_eq!(hints.len(), 5);
    assert_eq!(hints[0].text, "a");
    assert_eq!(hints[1].text, "b");
    assert_eq!(hints[2].text, "aa");
    assert_eq!(hints[3].text, "ab");
    assert_eq!(hints[4].text, "ba");
}

#[test]
fn test_hint_generation_respects_priority() {
    let mut generator = HintGenerator::new("asdf".to_string());

    let targets = vec![
        NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_priority(TargetPriority::Low),
        NavTarget::new(2, Rect::new(0, 2, 10, 1)).with_priority(TargetPriority::Critical),
        NavTarget::new(3, Rect::new(0, 4, 10, 1)).with_priority(TargetPriority::High),
        NavTarget::new(4, Rect::new(0, 6, 10, 1)).with_priority(TargetPriority::Normal),
    ];

    let hints = generator.generate(&targets);

    // Highest priority should get shortest hints
    assert_eq!(hints[0].target_id, 2); // Critical
    assert_eq!(hints[0].text, "a");
    assert_eq!(hints[1].target_id, 3); // High
    assert_eq!(hints[1].text, "s");
    assert_eq!(hints[2].target_id, 4); // Normal
    assert_eq!(hints[2].text, "d");
    assert_eq!(hints[3].target_id, 1); // Low
    assert_eq!(hints[3].text, "f");
}

#[test]
fn test_hint_generation_respects_position() {
    let mut generator = HintGenerator::new("asdf".to_string());

    let targets = vec![
        NavTarget::new(1, Rect::new(20, 10, 10, 1)),
        NavTarget::new(2, Rect::new(0, 10, 10, 1)),
        NavTarget::new(3, Rect::new(0, 0, 10, 1)),
        NavTarget::new(4, Rect::new(20, 0, 10, 1)),
    ];

    let hints = generator.generate(&targets);

    // Should be ordered top-left first: 3, 4, 2, 1
    assert_eq!(hints[0].target_id, 3); // (0, 0)
    assert_eq!(hints[1].target_id, 4); // (20, 0)
    assert_eq!(hints[2].target_id, 2); // (0, 10)
    assert_eq!(hints[3].target_id, 1); // (20, 10)
}

#[test]
fn test_hint_matching_simple() {
    let mut hint = Hint::new("asd".to_string(), 1);

    assert_eq!(hint.matched_chars, 0);
    assert!(!hint.is_complete());

    hint.update_match("a");
    assert_eq!(hint.matched_chars, 1);
    assert_eq!(hint.matched(), "a");
    assert_eq!(hint.unmatched(), "sd");
    assert!(!hint.is_complete());

    hint.update_match("as");
    assert_eq!(hint.matched_chars, 2);
    assert!(!hint.is_complete());

    hint.update_match("asd");
    assert_eq!(hint.matched_chars, 3);
    assert!(hint.is_complete());
}

#[test]
fn test_hint_matching_no_match() {
    let mut hint = Hint::new("asd".to_string(), 1);

    hint.update_match("xyz");
    assert_eq!(hint.matched_chars, 0);
    assert!(!hint.matches_input("xyz"));
}

#[test]
fn test_hint_matcher_basic() {
    let mut matcher = HintMatcher::new();

    let hints = vec![
        Hint::new("a".to_string(), 1),
        Hint::new("s".to_string(), 2),
        Hint::new("d".to_string(), 3),
    ];

    matcher.set_hints(hints);
    assert_eq!(matcher.hints().len(), 3);
    assert_eq!(matcher.input(), "");
}

#[test]
fn test_hint_matcher_push_char() {
    let mut matcher = HintMatcher::new();

    let hints = vec![
        Hint::new("a".to_string(), 1),
        Hint::new("s".to_string(), 2),
        Hint::new("as".to_string(), 3),
    ];

    matcher.set_hints(hints);

    // Type 'a' - should complete hint "a"
    let result = matcher.push_char('a');
    assert_eq!(result, Some(1));
    assert_eq!(matcher.input(), "a");
}

#[test]
fn test_hint_matcher_progressive_matching() {
    let mut matcher = HintMatcher::new();

    let hints = vec![
        Hint::new("aa".to_string(), 1),
        Hint::new("as".to_string(), 2),
        Hint::new("ad".to_string(), 3),
        Hint::new("sa".to_string(), 4),
    ];

    matcher.set_hints(hints);

    // Type 'a' - multiple matches
    let result = matcher.push_char('a');
    assert_eq!(result, None); // No unique match yet
    assert_eq!(matcher.matching_hints().len(), 3); // aa, as, ad

    // Type 's' - unique match "as"
    let result = matcher.push_char('s');
    assert_eq!(result, Some(2));
}

#[test]
fn test_hint_matcher_backspace() {
    let mut matcher = HintMatcher::new();

    let hints = vec![
        Hint::new("aa".to_string(), 1),
        Hint::new("as".to_string(), 2),
    ];

    matcher.set_hints(hints);

    matcher.push_char('a');
    matcher.push_char('s');
    assert_eq!(matcher.input(), "as");

    matcher.pop_char();
    assert_eq!(matcher.input(), "a");
    assert_eq!(matcher.matching_hints().len(), 2);
}

#[test]
fn test_hint_matcher_non_matching() {
    let mut matcher = HintMatcher::new();

    let hints = vec![
        Hint::new("aa".to_string(), 1),
        Hint::new("as".to_string(), 2),
        Hint::new("sa".to_string(), 3),
    ];

    matcher.set_hints(hints);
    matcher.push_char('a');

    let matching = matcher.matching_hints();
    let non_matching = matcher.non_matching_hints();

    assert_eq!(matching.len(), 2); // aa, as
    assert_eq!(non_matching.len(), 1); // sa
}

#[test]
fn test_hint_matcher_clear() {
    let mut matcher = HintMatcher::new();

    let hints = vec![Hint::new("a".to_string(), 1)];

    matcher.set_hints(hints);
    matcher.push_char('a');

    assert!(!matcher.hints().is_empty());
    assert!(!matcher.input().is_empty());

    matcher.clear();

    assert!(matcher.hints().is_empty());
    assert!(matcher.input().is_empty());
}

#[test]
fn test_hint_for_target() {
    let mut matcher = HintMatcher::new();

    let hints = vec![
        Hint::new("a".to_string(), 10),
        Hint::new("s".to_string(), 20),
    ];

    matcher.set_hints(hints);

    let hint = matcher.hint_for_target(10);
    assert!(hint.is_some());
    assert_eq!(hint.unwrap().text, "a");

    let hint = matcher.hint_for_target(20);
    assert!(hint.is_some());
    assert_eq!(hint.unwrap().text, "s");

    let hint = matcher.hint_for_target(99);
    assert!(hint.is_none());
}

#[test]
fn test_large_hint_set() {
    let mut generator = HintGenerator::new("asdfghjkl".to_string());

    // Generate 100 targets
    let targets: Vec<_> = (0..100)
        .map(|i| NavTarget::new(i, Rect::new(0, i as u16, 10, 1)))
        .collect();

    let hints = generator.generate(&targets);

    assert_eq!(hints.len(), 100);

    // All hints should be unique
    let unique_hints: std::collections::HashSet<_> = hints.iter().map(|h| &h.text).collect();
    assert_eq!(unique_hints.len(), 100);

    // First 9 should be single character
    for i in 0..9 {
        assert_eq!(hints[i].text.len(), 1);
    }

    // Next should be two characters
    assert_eq!(hints[9].text.len(), 2);
}

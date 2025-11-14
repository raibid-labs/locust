//! Hint generation algorithm for Vimium-style navigation.
//!
//! This module implements the core hint generation algorithm that assigns
//! unique, short character sequences to navigation targets.

use crate::core::targets::NavTarget;
use std::collections::HashMap;

/// A hint assigned to a navigation target.
///
/// Contains the hint string (e.g., "as", "df") and tracks which
/// characters have been matched by user input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hint {
    /// The full hint string (e.g., "as", "df", "jk")
    pub text: String,

    /// Target ID this hint is assigned to
    pub target_id: u64,

    /// Number of characters from the hint that have been matched
    pub matched_chars: usize,
}

impl Hint {
    /// Creates a new hint.
    pub fn new(text: String, target_id: u64) -> Self {
        Self {
            text,
            target_id,
            matched_chars: 0,
        }
    }

    /// Returns true if this hint is fully matched.
    pub fn is_complete(&self) -> bool {
        self.matched_chars == self.text.len()
    }

    /// Returns true if this hint matches the given input prefix.
    pub fn matches_input(&self, input: &str) -> bool {
        self.text.starts_with(input)
    }

    /// Returns the unmatched portion of the hint.
    pub fn unmatched(&self) -> &str {
        &self.text[self.matched_chars..]
    }

    /// Returns the matched portion of the hint.
    pub fn matched(&self) -> &str {
        &self.text[..self.matched_chars]
    }

    /// Updates the number of matched characters based on input.
    pub fn update_match(&mut self, input: &str) {
        let matching = input
            .chars()
            .zip(self.text.chars())
            .take_while(|(a, b)| a == b)
            .count();
        self.matched_chars = matching;
    }
}

/// Generates hints for a list of navigation targets.
///
/// This struct implements the Vimium-style hint generation algorithm:
/// 1. Sort targets by priority and position
/// 2. Generate shortest unique hints from charset
/// 3. Assign hints to targets in order
///
/// The algorithm generates hints in this order:
/// - 1 character: a, s, d, f, ...
/// - 2 characters: aa, as, ad, af, sa, ss, sd, ...
/// - 3 characters: aaa, aas, aad, ...
///
/// # Examples
///
/// ```rust
/// use locust::plugins::nav::hints::HintGenerator;
/// use locust::core::targets::NavTarget;
/// use ratatui::layout::Rect;
///
/// let mut generator = HintGenerator::new("asdf".to_string());
/// let targets = vec![
///     NavTarget::new(1, Rect::new(0, 0, 10, 1)),
///     NavTarget::new(2, Rect::new(0, 2, 10, 1)),
///     NavTarget::new(3, Rect::new(0, 4, 10, 1)),
/// ];
///
/// let hints = generator.generate(&targets);
/// assert_eq!(hints.len(), 3);
/// assert_eq!(hints[0].text, "a");
/// assert_eq!(hints[1].text, "s");
/// assert_eq!(hints[2].text, "d");
/// ```
pub struct HintGenerator {
    /// Character set used for generating hints
    charset: String,

    /// Cached charset length for performance
    charset_len: usize,
}

impl HintGenerator {
    /// Creates a new hint generator with the given character set.
    ///
    /// # Panics
    ///
    /// Panics if the charset is empty.
    pub fn new(charset: String) -> Self {
        assert!(!charset.is_empty(), "Charset cannot be empty");
        let charset_len = charset.chars().count();
        Self {
            charset,
            charset_len,
        }
    }

    /// Generates hints for the given targets.
    ///
    /// Targets are sorted by priority (highest first) and then by
    /// vertical then horizontal position. The most prominent targets
    /// receive the shortest hints.
    pub fn generate(&mut self, targets: &[NavTarget]) -> Vec<Hint> {
        if targets.is_empty() {
            return Vec::new();
        }

        // Sort targets by priority (descending) and position (top-left first)
        let mut sorted_targets: Vec<&NavTarget> = targets.iter().collect();
        sorted_targets.sort_by(|a, b| {
            // First by priority (higher is better)
            match b.priority.cmp(&a.priority) {
                std::cmp::Ordering::Equal => {
                    // Then by vertical position (top first)
                    match a.rect.y.cmp(&b.rect.y) {
                        std::cmp::Ordering::Equal => {
                            // Finally by horizontal position (left first)
                            a.rect.x.cmp(&b.rect.x)
                        }
                        other => other,
                    }
                }
                other => other,
            }
        });

        // Generate hints in order
        sorted_targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let hint_text = self.generate_hint_string(index);
                Hint::new(hint_text, target.id)
            })
            .collect()
    }

    /// Generates the hint string for the given index.
    ///
    /// Uses a base-N encoding where N is the charset length.
    /// Index 0 -> "a", 1 -> "s", ..., N -> "aa", N+1 -> "as", etc.
    fn generate_hint_string(&self, index: usize) -> String {
        let chars: Vec<char> = self.charset.chars().collect();
        let mut result = String::new();
        let mut n = index;

        // Convert index to base-N representation
        loop {
            let digit = n % self.charset_len;
            result.push(chars[digit]);
            n /= self.charset_len;

            if n == 0 {
                break;
            }
            // Adjust for 0-based indexing
            n -= 1;
        }

        // Reverse because we built it backwards
        result.chars().rev().collect()
    }

    /// Returns the character set used by this generator.
    pub fn charset(&self) -> &str {
        &self.charset
    }
}

/// Manages active hints and input matching.
///
/// This struct tracks the current set of hints and handles user input
/// to progressively narrow down matches until a single hint is selected.
#[derive(Debug, Default)]
pub struct HintMatcher {
    /// Current user input
    input: String,

    /// All active hints
    hints: Vec<Hint>,

    /// Map from target ID to hint index
    target_to_hint: HashMap<u64, usize>,
}

impl HintMatcher {
    /// Creates a new hint matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the active hints.
    pub fn set_hints(&mut self, hints: Vec<Hint>) {
        self.target_to_hint.clear();
        for (idx, hint) in hints.iter().enumerate() {
            self.target_to_hint.insert(hint.target_id, idx);
        }
        self.hints = hints;
        self.input.clear();
    }

    /// Returns all hints.
    pub fn hints(&self) -> &[Hint] {
        &self.hints
    }

    /// Returns the current input string.
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Clears all hints and input.
    pub fn clear(&mut self) {
        self.hints.clear();
        self.target_to_hint.clear();
        self.input.clear();
    }

    /// Adds a character to the input and updates hint matching.
    ///
    /// Returns `Some(target_id)` if a hint was fully matched.
    pub fn push_char(&mut self, c: char) -> Option<u64> {
        self.input.push(c);
        self.update_matches();
        self.check_complete_match()
    }

    /// Removes the last character from input.
    pub fn pop_char(&mut self) {
        self.input.pop();
        self.update_matches();
    }

    /// Updates match counts for all hints based on current input.
    fn update_matches(&mut self) {
        for hint in &mut self.hints {
            hint.update_match(&self.input);
        }
    }

    /// Checks if exactly one hint is fully matched.
    ///
    /// Returns the target ID if found.
    fn check_complete_match(&self) -> Option<u64> {
        let complete_hints: Vec<_> = self.hints.iter().filter(|h| h.is_complete()).collect();

        if complete_hints.len() == 1 {
            Some(complete_hints[0].target_id)
        } else {
            None
        }
    }

    /// Returns hints that match the current input.
    pub fn matching_hints(&self) -> Vec<&Hint> {
        self.hints
            .iter()
            .filter(|h| h.matches_input(&self.input))
            .collect()
    }

    /// Returns hints that don't match the current input (should be dimmed).
    pub fn non_matching_hints(&self) -> Vec<&Hint> {
        self.hints
            .iter()
            .filter(|h| !h.matches_input(&self.input))
            .collect()
    }

    /// Returns the hint for a given target ID.
    pub fn hint_for_target(&self, target_id: u64) -> Option<&Hint> {
        self.target_to_hint
            .get(&target_id)
            .and_then(|&idx| self.hints.get(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::targets::{NavTarget, TargetPriority};
    use ratatui::layout::Rect;

    #[test]
    fn test_hint_creation() {
        let hint = Hint::new("as".to_string(), 42);
        assert_eq!(hint.text, "as");
        assert_eq!(hint.target_id, 42);
        assert_eq!(hint.matched_chars, 0);
        assert!(!hint.is_complete());
    }

    #[test]
    fn test_hint_matching() {
        let mut hint = Hint::new("asd".to_string(), 1);

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
    fn test_hint_generation_simple() {
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
    fn test_hint_generation_two_char() {
        let mut generator = HintGenerator::new("as".to_string());
        let targets = vec![
            NavTarget::new(1, Rect::new(0, 0, 10, 1)),
            NavTarget::new(2, Rect::new(0, 2, 10, 1)),
            NavTarget::new(3, Rect::new(0, 4, 10, 1)),
        ];

        let hints = generator.generate(&targets);
        assert_eq!(hints.len(), 3);
        assert_eq!(hints[0].text, "a");
        assert_eq!(hints[1].text, "s");
        assert_eq!(hints[2].text, "aa");
    }

    #[test]
    fn test_hint_generation_priority() {
        let mut generator = HintGenerator::new("asdf".to_string());
        let targets = vec![
            NavTarget::new(1, Rect::new(0, 0, 10, 1)).with_priority(TargetPriority::Normal),
            NavTarget::new(2, Rect::new(0, 2, 10, 1)).with_priority(TargetPriority::High),
            NavTarget::new(3, Rect::new(0, 4, 10, 1)).with_priority(TargetPriority::Critical),
        ];

        let hints = generator.generate(&targets);
        assert_eq!(hints.len(), 3);
        // Highest priority should get shortest hint
        assert_eq!(hints[0].target_id, 3); // Critical
        assert_eq!(hints[0].text, "a");
        assert_eq!(hints[1].target_id, 2); // High
        assert_eq!(hints[1].text, "s");
        assert_eq!(hints[2].target_id, 1); // Normal
        assert_eq!(hints[2].text, "d");
    }

    #[test]
    fn test_hint_matcher() {
        let mut matcher = HintMatcher::new();
        let hints = vec![
            Hint::new("a".to_string(), 1),
            Hint::new("s".to_string(), 2),
            Hint::new("as".to_string(), 3),
        ];
        matcher.set_hints(hints);

        assert_eq!(matcher.matching_hints().len(), 3);

        // Type 'a' - should match hints "a" and "as"
        let result = matcher.push_char('a');
        assert_eq!(result, Some(1)); // Hint "a" is complete
    }

    #[test]
    fn test_hint_matcher_progressive() {
        let mut matcher = HintMatcher::new();
        let hints = vec![
            Hint::new("aa".to_string(), 1),
            Hint::new("as".to_string(), 2),
            Hint::new("ad".to_string(), 3),
        ];
        matcher.set_hints(hints);

        // Type 'a' - all three match
        let result = matcher.push_char('a');
        assert_eq!(result, None);
        assert_eq!(matcher.matching_hints().len(), 3);

        // Type 's' - only "as" matches
        let result = matcher.push_char('s');
        assert_eq!(result, Some(2));
    }
}

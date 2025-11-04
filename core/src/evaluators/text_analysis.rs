// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Text analysis utilities for evaluators
//!
//! This module provides shared utilities for analyzing text properties:
//! - Syllable counting
//! - Sentence splitting
//! - Word counting
//! - Readability metrics (Flesch-Kincaid)
//! - Discourse marker detection

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use unicode_segmentation::UnicodeSegmentation;

/// A discourse marker indicating logical connections in text
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscourseMarker {
    /// The marker text (e.g., "however", "therefore")
    pub marker: String,
    /// Type of discourse relation
    pub marker_type: DiscourseMarkerType,
    /// Position in the text (character index)
    pub position: usize,
}

/// Types of discourse markers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscourseMarkerType {
    /// Contrast/opposition (however, but, although)
    Contrast,
    /// Addition/continuation (furthermore, moreover, additionally)
    Addition,
    /// Causation/result (therefore, thus, consequently)
    Causation,
    /// Exemplification (for example, for instance)
    Example,
    /// Temporal sequence (then, next, finally)
    Temporal,
    /// Conclusion (in conclusion, to sum up)
    Conclusion,
}

/// Count the number of syllables in a word using a heuristic approach
///
/// This is a simplified English syllable counter based on vowel groups.
/// While not perfect, it provides reasonable estimates for readability metrics.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::count_syllables;
///
/// assert_eq!(count_syllables("hello"), 2);
/// assert_eq!(count_syllables("world"), 1);
/// assert_eq!(count_syllables("beautiful"), 3);
/// assert_eq!(count_syllables("a"), 1);
/// ```
pub fn count_syllables(word: &str) -> usize {
    let word = word.to_lowercase();
    let word = word.trim();

    if word.is_empty() {
        return 0;
    }

    // Single letter words are one syllable
    if word.len() == 1 {
        return 1;
    }

    let mut count = 0;
    let chars: Vec<char> = word.chars().collect();
    let mut prev_was_vowel = false;

    for (i, &ch) in chars.iter().enumerate() {
        let is_vowel = matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');

        if is_vowel && !prev_was_vowel {
            count += 1;
        }

        prev_was_vowel = is_vowel;
    }

    // Adjust for silent 'e' at the end
    if word.ends_with('e') && count > 1 {
        count -= 1;
    }

    // Adjust for words ending in 'le' (like "table")
    if word.len() >= 2 && word.ends_with("le") && count > 1 {
        let before_le = chars[chars.len() - 3];
        if !matches!(before_le, 'a' | 'e' | 'i' | 'o' | 'u') {
            count += 1;
        }
    }

    // Every word has at least one syllable
    count.max(1)
}

/// Split text into sentences
///
/// Uses a simple heuristic based on sentence-ending punctuation followed by
/// whitespace and capital letters.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::split_sentences;
///
/// let text = "Hello world. This is a test. How are you?";
/// let sentences = split_sentences(text);
/// assert_eq!(sentences.len(), 3);
/// ```
pub fn split_sentences(text: &str) -> Vec<String> {
    static SENTENCE_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = SENTENCE_REGEX.get_or_init(|| {
        // Split on . ! ? followed by whitespace (but not in abbreviations like "Dr.")
        Regex::new(r"(?<=[.!?])\s+(?=[A-Z])").unwrap()
    });

    regex
        .split(text)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Count words in text
///
/// Uses Unicode word boundaries for accurate counting across languages.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::count_words;
///
/// assert_eq!(count_words("Hello world"), 2);
/// assert_eq!(count_words("The quick brown fox"), 4);
/// assert_eq!(count_words(""), 0);
/// ```
pub fn count_words(text: &str) -> usize {
    text.unicode_words().count()
}

/// Calculate Flesch Reading Ease score
///
/// Formula: 206.835 - 1.015(total words / total sentences) - 84.6(total syllables / total words)
///
/// Score interpretation:
/// - 90-100: Very Easy (5th grade)
/// - 80-89: Easy (6th grade)
/// - 70-79: Fairly Easy (7th grade)
/// - 60-69: Standard (8th-9th grade)
/// - 50-59: Fairly Difficult (10th-12th grade)
/// - 30-49: Difficult (College)
/// - 0-29: Very Difficult (College graduate)
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::flesch_reading_ease;
///
/// let text = "The cat sat on the mat. It was a sunny day.";
/// let score = flesch_reading_ease(text);
/// assert!(score > 80.0); // Should be easy to read
/// ```
pub fn flesch_reading_ease(text: &str) -> f64 {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return 0.0;
    }

    let total_words = count_words(text);
    if total_words == 0 {
        return 0.0;
    }

    let total_syllables: usize = text
        .unicode_words()
        .map(count_syllables)
        .sum();

    let avg_sentence_length = total_words as f64 / sentences.len() as f64;
    let avg_syllables_per_word = total_syllables as f64 / total_words as f64;

    206.835 - 1.015 * avg_sentence_length - 84.6 * avg_syllables_per_word
}

/// Calculate Flesch-Kincaid Grade Level
///
/// Formula: 0.39(total words / total sentences) + 11.8(total syllables / total words) - 15.59
///
/// The result represents the U.S. grade level needed to understand the text.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::flesch_kincaid_grade;
///
/// let text = "The cat sat on the mat. It was a sunny day.";
/// let grade = flesch_kincaid_grade(text);
/// assert!(grade < 5.0); // Should be elementary school level
/// ```
pub fn flesch_kincaid_grade(text: &str) -> f64 {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return 0.0;
    }

    let total_words = count_words(text);
    if total_words == 0 {
        return 0.0;
    }

    let total_syllables: usize = text
        .unicode_words()
        .map(count_syllables)
        .sum();

    let avg_sentence_length = total_words as f64 / sentences.len() as f64;
    let avg_syllables_per_word = total_syllables as f64 / total_words as f64;

    0.39 * avg_sentence_length + 11.8 * avg_syllables_per_word - 15.59
}

/// Detect discourse markers in text
///
/// Identifies logical connectors that indicate text structure and coherence.
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::detect_discourse_markers;
///
/// let text = "However, the results were different. Therefore, we conclude...";
/// let markers = detect_discourse_markers(text);
/// assert!(markers.len() >= 2);
/// ```
pub fn detect_discourse_markers(text: &str) -> Vec<DiscourseMarker> {
    let text_lower = text.to_lowercase();
    let mut markers = Vec::new();

    // Define marker patterns with their types
    let marker_patterns: &[(&str, DiscourseMarkerType)] = &[
        // Contrast
        ("however", DiscourseMarkerType::Contrast),
        ("but", DiscourseMarkerType::Contrast),
        ("although", DiscourseMarkerType::Contrast),
        ("though", DiscourseMarkerType::Contrast),
        ("nevertheless", DiscourseMarkerType::Contrast),
        ("nonetheless", DiscourseMarkerType::Contrast),
        ("on the other hand", DiscourseMarkerType::Contrast),
        ("in contrast", DiscourseMarkerType::Contrast),
        ("conversely", DiscourseMarkerType::Contrast),
        ("yet", DiscourseMarkerType::Contrast),
        ("still", DiscourseMarkerType::Contrast),

        // Addition
        ("furthermore", DiscourseMarkerType::Addition),
        ("moreover", DiscourseMarkerType::Addition),
        ("additionally", DiscourseMarkerType::Addition),
        ("also", DiscourseMarkerType::Addition),
        ("besides", DiscourseMarkerType::Addition),
        ("in addition", DiscourseMarkerType::Addition),
        ("as well", DiscourseMarkerType::Addition),
        ("similarly", DiscourseMarkerType::Addition),
        ("likewise", DiscourseMarkerType::Addition),

        // Causation
        ("therefore", DiscourseMarkerType::Causation),
        ("thus", DiscourseMarkerType::Causation),
        ("consequently", DiscourseMarkerType::Causation),
        ("as a result", DiscourseMarkerType::Causation),
        ("hence", DiscourseMarkerType::Causation),
        ("accordingly", DiscourseMarkerType::Causation),
        ("because", DiscourseMarkerType::Causation),
        ("since", DiscourseMarkerType::Causation),
        ("so", DiscourseMarkerType::Causation),

        // Example
        ("for example", DiscourseMarkerType::Example),
        ("for instance", DiscourseMarkerType::Example),
        ("such as", DiscourseMarkerType::Example),
        ("specifically", DiscourseMarkerType::Example),
        ("namely", DiscourseMarkerType::Example),
        ("to illustrate", DiscourseMarkerType::Example),

        // Temporal
        ("then", DiscourseMarkerType::Temporal),
        ("next", DiscourseMarkerType::Temporal),
        ("finally", DiscourseMarkerType::Temporal),
        ("first", DiscourseMarkerType::Temporal),
        ("second", DiscourseMarkerType::Temporal),
        ("third", DiscourseMarkerType::Temporal),
        ("subsequently", DiscourseMarkerType::Temporal),
        ("afterward", DiscourseMarkerType::Temporal),
        ("meanwhile", DiscourseMarkerType::Temporal),

        // Conclusion
        ("in conclusion", DiscourseMarkerType::Conclusion),
        ("to sum up", DiscourseMarkerType::Conclusion),
        ("to summarize", DiscourseMarkerType::Conclusion),
        ("in summary", DiscourseMarkerType::Conclusion),
        ("overall", DiscourseMarkerType::Conclusion),
        ("ultimately", DiscourseMarkerType::Conclusion),
    ];

    // Search for each marker pattern
    for (pattern, marker_type) in marker_patterns {
        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(pattern) {
            let absolute_pos = start + pos;

            // Check if it's a word boundary match (not part of another word)
            let is_word_boundary = {
                let before_ok = absolute_pos == 0 ||
                    !text_lower.chars().nth(absolute_pos - 1)
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false);

                let after_pos = absolute_pos + pattern.len();
                let after_ok = after_pos >= text_lower.len() ||
                    !text_lower.chars().nth(after_pos)
                        .map(|c| c.is_alphanumeric())
                        .unwrap_or(false);

                before_ok && after_ok
            };

            if is_word_boundary {
                markers.push(DiscourseMarker {
                    marker: pattern.to_string(),
                    marker_type: *marker_type,
                    position: absolute_pos,
                });
            }

            start = absolute_pos + pattern.len();
        }
    }

    // Sort by position
    markers.sort_by_key(|m| m.position);
    markers
}

/// Calculate average sentence length in words
///
/// # Examples
///
/// ```
/// use llm_test_bench_core::evaluators::text_analysis::average_sentence_length;
///
/// let text = "Hello world. This is a test.";
/// let avg = average_sentence_length(text);
/// assert_eq!(avg, 3.0); // (2 + 4) / 2
/// ```
pub fn average_sentence_length(text: &str) -> f64 {
    let sentences = split_sentences(text);
    if sentences.is_empty() {
        return 0.0;
    }

    let total_words = count_words(text);
    total_words as f64 / sentences.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_syllables() {
        assert_eq!(count_syllables("hello"), 2);
        assert_eq!(count_syllables("world"), 1);
        assert_eq!(count_syllables("beautiful"), 3);
        assert_eq!(count_syllables("a"), 1);
        assert_eq!(count_syllables("the"), 1);
        assert_eq!(count_syllables("table"), 2);
        assert_eq!(count_syllables("simple"), 2);
        assert_eq!(count_syllables(""), 0);
        assert_eq!(count_syllables("I"), 1);
    }

    #[test]
    fn test_split_sentences() {
        let text = "Hello world. This is a test. How are you?";
        let sentences = split_sentences(text);
        assert_eq!(sentences.len(), 3);
        assert_eq!(sentences[0], "Hello world.");
        assert_eq!(sentences[1], "This is a test.");
        assert_eq!(sentences[2], "How are you?");
    }

    #[test]
    fn test_split_sentences_edge_cases() {
        assert_eq!(split_sentences(""), Vec::<String>::new());
        assert_eq!(split_sentences("No punctuation"), vec!["No punctuation"]);
        assert_eq!(split_sentences("One sentence."), vec!["One sentence."]);
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("Hello world"), 2);
        assert_eq!(count_words("The quick brown fox"), 4);
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("   spaces   "), 1);
        assert_eq!(count_words("One"), 1);
    }

    #[test]
    fn test_flesch_reading_ease() {
        // Simple text should have high score
        let simple = "The cat sat. The dog ran.";
        let score = flesch_reading_ease(simple);
        assert!(score > 80.0, "Simple text should be easy to read: {}", score);

        // Complex text should have lower score
        let complex = "The implementation necessitates comprehensive understanding of multifaceted algorithmic considerations.";
        let score = flesch_reading_ease(complex);
        assert!(score < 50.0, "Complex text should be harder to read: {}", score);

        // Empty text
        assert_eq!(flesch_reading_ease(""), 0.0);
    }

    #[test]
    fn test_flesch_kincaid_grade() {
        // Simple text should be low grade level
        let simple = "The cat sat. The dog ran.";
        let grade = flesch_kincaid_grade(simple);
        assert!(grade < 5.0, "Simple text should be low grade: {}", grade);

        // Complex text should be higher grade level
        let complex = "The implementation necessitates comprehensive understanding of multifaceted algorithmic considerations.";
        let grade = flesch_kincaid_grade(complex);
        assert!(grade > 10.0, "Complex text should be high grade: {}", grade);

        // Empty text
        assert_eq!(flesch_kincaid_grade(""), 0.0);
    }

    #[test]
    fn test_detect_discourse_markers() {
        let text = "However, the results were different. Therefore, we conclude that the hypothesis was incorrect. For example, in the first trial...";
        let markers = detect_discourse_markers(text);

        assert!(markers.len() >= 3);
        assert!(markers.iter().any(|m| m.marker == "however"));
        assert!(markers.iter().any(|m| m.marker == "therefore"));
        assert!(markers.iter().any(|m| m.marker == "for example"));

        // Check types
        let however = markers.iter().find(|m| m.marker == "however").unwrap();
        assert_eq!(however.marker_type, DiscourseMarkerType::Contrast);

        let therefore = markers.iter().find(|m| m.marker == "therefore").unwrap();
        assert_eq!(therefore.marker_type, DiscourseMarkerType::Causation);
    }

    #[test]
    fn test_detect_discourse_markers_word_boundaries() {
        // Should not match "however" in "whatsoever"
        let text = "Whatsoever the case may be.";
        let markers = detect_discourse_markers(text);
        assert!(markers.is_empty());

        // Should match standalone "however"
        let text = "However, we proceed.";
        let markers = detect_discourse_markers(text);
        assert_eq!(markers.len(), 1);
        assert_eq!(markers[0].marker, "however");
    }

    #[test]
    fn test_detect_discourse_markers_empty() {
        assert_eq!(detect_discourse_markers(""), Vec::<DiscourseMarker>::new());
        assert_eq!(detect_discourse_markers("No markers here just plain text"),
                   Vec::<DiscourseMarker>::new());
    }

    #[test]
    fn test_average_sentence_length() {
        let text = "Hello world. This is a test sentence.";
        let avg = average_sentence_length(text);
        assert_eq!(avg, 3.5); // (2 + 5) / 2

        assert_eq!(average_sentence_length(""), 0.0);
        assert_eq!(average_sentence_length("One sentence here"), 3.0);
    }

    #[test]
    fn test_discourse_marker_positions_sorted() {
        let text = "First, we start. Then we continue. Finally, we conclude.";
        let markers = detect_discourse_markers(text);

        // Markers should be in order of appearance
        for i in 1..markers.len() {
            assert!(markers[i].position > markers[i-1].position);
        }
    }

    #[test]
    fn test_multi_word_discourse_markers() {
        let text = "On the other hand, the data suggests otherwise. In conclusion, we find the results compelling.";
        let markers = detect_discourse_markers(text);

        assert!(markers.iter().any(|m| m.marker == "on the other hand"));
        assert!(markers.iter().any(|m| m.marker == "in conclusion"));
    }
}

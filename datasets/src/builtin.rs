// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Built-in benchmark datasets.
//!
//! This module provides factory functions for creating ready-to-use benchmark datasets.
//! All built-in datasets include comprehensive test cases with proper categorization,
//! expected outputs, and template variables where applicable.
//!
//! # Available Datasets
//!
//! - **coding-tasks**: Programming challenges (FizzBuzz, string manipulation, algorithms)
//! - **reasoning-tasks**: Logic puzzles and math word problems
//! - **summarization-tasks**: Text summarization and compression
//! - **instruction-following**: Instruction adherence and format compliance
//! - **creative-writing**: Creative generation and storytelling

use crate::schema::{Dataset, DefaultConfig, TestCase};

/// Get all built-in datasets.
///
/// Returns a vector containing all 5 built-in benchmark datasets.
///
/// # Example
///
/// ```
/// use llm_test_bench_datasets::builtin::get_builtin_datasets;
///
/// let datasets = get_builtin_datasets();
/// assert_eq!(datasets.len(), 5);
/// ```
pub fn get_builtin_datasets() -> Vec<Dataset> {
    vec![
        coding_tasks(),
        reasoning_tasks(),
        summarization_tasks(),
        instruction_following(),
        creative_writing(),
    ]
}

/// Coding tasks dataset.
///
/// Programming challenges including FizzBuzz, string manipulation,
/// and basic algorithms. Uses temperature 0.0 for deterministic outputs.
///
/// # Example
///
/// ```
/// use llm_test_bench_datasets::builtin::coding_tasks;
///
/// let dataset = coding_tasks();
/// assert_eq!(dataset.name, "coding-tasks");
/// assert!(dataset.test_cases.len() >= 5);
/// ```
pub fn coding_tasks() -> Dataset {
    let defaults = DefaultConfig::new()
        .with_temperature(0.0)
        .with_max_tokens(500);

    let mut dataset = Dataset::new("coding-tasks", "1.0.0")
        .with_description("Programming challenges in multiple languages")
        .with_defaults(defaults);

    // FizzBuzz with template variable
    dataset.add_test_case(
        TestCase::new(
            "fizzbuzz-python",
            "Write a Python function that implements FizzBuzz for numbers 1 to {{n}}."
        )
        .with_category("coding")
        .add_variable("n", "100")
        .with_expected("def fizzbuzz")
        .with_references(vec![
            "for i in range".to_string(),
            "if i % 15".to_string(),
            "FizzBuzz".to_string(),
        ])
    );

    // String reverse
    dataset.add_test_case(
        TestCase::new(
            "reverse-string-rust",
            "Write a Rust function to reverse a string in-place."
        )
        .with_category("coding")
        .with_expected("fn reverse")
        .with_references(vec![
            "chars()".to_string(),
            "rev()".to_string(),
            "collect()".to_string(),
        ])
    );

    // Fibonacci with template
    dataset.add_test_case(
        TestCase::new(
            "fibonacci-javascript",
            "Implement a {{lang}} function to calculate the nth Fibonacci number using recursion."
        )
        .with_category("coding")
        .add_variable("lang", "JavaScript")
        .with_expected("function")
        .with_references(vec![
            "fibonacci".to_string(),
            "return".to_string(),
        ])
    );

    // Palindrome checker
    dataset.add_test_case(
        TestCase::new(
            "palindrome-checker",
            "Write a {{lang}} function that checks if a string is a palindrome (case-insensitive)."
        )
        .with_category("coding")
        .add_variable("lang", "Python")
        .with_expected("def")
        .with_references(vec![
            "lower()".to_string(),
            "==".to_string(),
        ])
    );

    // Array sum
    dataset.add_test_case(
        TestCase::new(
            "array-sum",
            "Create a function in {{lang}} that sums all elements in an array."
        )
        .with_category("coding")
        .add_variable("lang", "TypeScript")
        .with_expected("function")
    );

    // Binary search
    dataset.add_test_case(
        TestCase::new(
            "binary-search",
            "Implement a binary search algorithm in {{lang}}. Include comments explaining the logic."
        )
        .with_category("coding")
        .add_variable("lang", "Python")
        .with_expected("def binary_search")
        .with_references(vec![
            "while".to_string(),
            "mid".to_string(),
            "left".to_string(),
            "right".to_string(),
        ])
    );

    // Find duplicates
    dataset.add_test_case(
        TestCase::new(
            "find-duplicates",
            "Write a {{lang}} function to find all duplicate elements in an array."
        )
        .with_category("coding")
        .add_variable("lang", "JavaScript")
        .with_expected("function")
    );

    dataset
}

/// Reasoning tasks dataset.
///
/// Logic puzzles and math word problems. Uses temperature 0.7
/// for more natural reasoning.
pub fn reasoning_tasks() -> Dataset {
    let defaults = DefaultConfig::new()
        .with_temperature(0.7)
        .with_max_tokens(300);

    let mut dataset = Dataset::new("reasoning-tasks", "1.0.0")
        .with_description("Logical reasoning and problem-solving tasks")
        .with_defaults(defaults);

    // Logic puzzle
    dataset.add_test_case(
        TestCase::new(
            "logic-puzzle-truthtellers",
            r#"Three people are in a room: Alice, Bob, and Carol.
- Alice always tells the truth
- Bob always lies
- Carol sometimes tells the truth and sometimes lies

Alice says: "Bob is lying."
Bob says: "Carol is telling the truth."
Carol says: "I am lying."

Who is telling the truth? Explain your reasoning."#
        )
        .with_category("reasoning")
        .with_expected("Alice")
        .with_references(vec![
            "Alice tells the truth".to_string(),
            "logical contradiction".to_string(),
        ])
    );

    // Math word problem with template
    dataset.add_test_case(
        TestCase::new(
            "math-word-problem-distance",
            "If a train travels {{distance}} km at {{speed}} km/h, how long does the journey take in hours? Show your calculation."
        )
        .with_category("reasoning")
        .add_variable("distance", "240")
        .add_variable("speed", "80")
        .with_expected("3 hours")
    );

    // Pattern recognition
    dataset.add_test_case(
        TestCase::new(
            "pattern-recognition",
            "What comes next in this sequence: 2, 4, 8, 16, 32, ?"
        )
        .with_category("reasoning")
        .with_expected("64")
    );

    // Age problem
    dataset.add_test_case(
        TestCase::new(
            "age-problem",
            "Alice is twice as old as Bob. In 5 years, Alice will be 1.5 times as old as Bob. How old are they now?"
        )
        .with_category("reasoning")
        .with_expected("10")
        .with_references(vec![
            "Alice: 10".to_string(),
            "Bob: 5".to_string(),
        ])
    );

    // River crossing puzzle
    dataset.add_test_case(
        TestCase::new(
            "river-crossing",
            "A farmer needs to cross a river with a fox, a chicken, and a bag of grain. The boat can only carry the farmer and one item. If left alone, the fox will eat the chicken, and the chicken will eat the grain. How can the farmer get everything across safely?"
        )
        .with_category("reasoning")
        .with_references(vec![
            "chicken first".to_string(),
            "come back alone".to_string(),
        ])
    );

    dataset
}

/// Summarization tasks dataset.
///
/// Text summarization and compression challenges.
pub fn summarization_tasks() -> Dataset {
    let defaults = DefaultConfig::new()
        .with_temperature(0.5)
        .with_max_tokens(200);

    let mut dataset = Dataset::new("summarization-tasks", "1.0.0")
        .with_description("Text summarization tasks")
        .with_defaults(defaults);

    dataset.add_test_case(
        TestCase::new(
            "summarize-article-short",
            r#"Summarize the following text in one sentence:

"The Amazon rainforest, also known as Amazonia, is a moist broadleaf tropical rainforest in the Amazon biome that covers most of the Amazon basin of South America. This basin encompasses 7,000,000 km2, of which 5,500,000 km2 are covered by the rainforest. The majority of the forest is contained within Brazil, with 60% of the rainforest, followed by Peru with 13%, Colombia with 10%, and minor amounts in other countries."

Summary:"#
        )
        .with_category("summarization")
        .with_references(vec![
            "Amazon".to_string(),
            "South America".to_string(),
            "Brazil".to_string(),
        ])
    );

    dataset.add_test_case(
        TestCase::new(
            "summarize-bullet-points",
            "Convert the following paragraph into 3 bullet points:\n\n\"Rust is a multi-paradigm programming language focused on performance and safety. It prevents segmentation faults and guarantees thread safety. Rust achieves memory safety without garbage collection, and reference counting is optional.\""
        )
        .with_category("summarization")
        .with_references(vec![
            "performance".to_string(),
            "safety".to_string(),
            "no garbage collection".to_string(),
        ])
    );

    dataset.add_test_case(
        TestCase::new(
            "extract-key-points",
            "List the {{n}} most important points from this text:\n\n\"Climate change is causing global temperatures to rise. This leads to melting ice caps, rising sea levels, and more extreme weather events. Scientists recommend reducing carbon emissions through renewable energy, sustainable transportation, and energy-efficient buildings.\""
        )
        .with_category("summarization")
        .add_variable("n", "3")
    );

    dataset.add_test_case(
        TestCase::new(
            "tldr-generation",
            "Provide a TL;DR (Too Long; Didn't Read) summary of this GitHub pull request description:\n\n\"This PR adds comprehensive error handling to the authentication module. Previously, failed login attempts would crash the application. Now, all errors are properly caught and logged. Users receive clear error messages instead of cryptic stack traces. Tests have been added to cover all error cases.\""
        )
        .with_category("summarization")
    );

    dataset
}

/// Instruction following dataset.
///
/// Tests for instruction adherence and format compliance.
pub fn instruction_following() -> Dataset {
    let defaults = DefaultConfig::new()
        .with_temperature(0.3)
        .with_max_tokens(400);

    let mut dataset = Dataset::new("instruction-following", "1.0.0")
        .with_description("Instruction adherence tests")
        .with_defaults(defaults);

    dataset.add_test_case(
        TestCase::new(
            "format-numbered-list",
            "List {{n}} fruits. Format your response as a numbered list with no additional text."
        )
        .with_category("instruction-following")
        .add_variable("n", "three")
        .with_references(vec![
            "1.".to_string(),
            "2.".to_string(),
            "3.".to_string(),
        ])
    );

    dataset.add_test_case(
        TestCase::new(
            "format-json",
            r#"Provide information about a cat in JSON format with exactly these fields: name (string), age (number), color (string). Do not include any text before or after the JSON."#
        )
        .with_category("instruction-following")
        .with_references(vec![
            "{".to_string(),
            "\"name\"".to_string(),
            "\"age\"".to_string(),
            "\"color\"".to_string(),
        ])
    );

    dataset.add_test_case(
        TestCase::new(
            "multi-step-instructions",
            "Follow these steps exactly:\n1. Write a haiku about coding\n2. Explain what a haiku is\n3. Count the syllables in your haiku\n\nNumber each section (1, 2, 3)."
        )
        .with_category("instruction-following")
    );

    dataset.add_test_case(
        TestCase::new(
            "word-limit",
            "Explain quantum computing in exactly {{words}} words. No more, no less."
        )
        .with_category("instruction-following")
        .add_variable("words", "50")
    );

    dataset.add_test_case(
        TestCase::new(
            "avoid-word",
            "Explain how to make a sandwich without using the word 'bread'."
        )
        .with_category("instruction-following")
    );

    dataset.add_test_case(
        TestCase::new(
            "format-csv",
            "Create a CSV table with 3 rows of sample employee data. Columns: id, name, department, salary. Include the header row."
        )
        .with_category("instruction-following")
        .with_references(vec![
            "id,name,department,salary".to_string(),
            ",".to_string(),
        ])
    );

    dataset
}

/// Creative writing dataset.
///
/// Creative generation and storytelling tasks.
pub fn creative_writing() -> Dataset {
    let defaults = DefaultConfig::new()
        .with_temperature(0.9)
        .with_max_tokens(600);

    let mut dataset = Dataset::new("creative-writing", "1.0.0")
        .with_description("Creative generation tasks")
        .with_defaults(defaults);

    dataset.add_test_case(
        TestCase::new(
            "story-opening",
            "Write an engaging opening paragraph for a science fiction story about {{topic}}."
        )
        .with_category("creative-writing")
        .add_variable("topic", "time travel")
    );

    dataset.add_test_case(
        TestCase::new(
            "haiku-nature",
            "Write a haiku about {{subject}}. Follow the traditional 5-7-5 syllable pattern."
        )
        .with_category("creative-writing")
        .add_variable("subject", "autumn leaves")
        .with_references(vec![
            "5 syllables".to_string(),
            "7 syllables".to_string(),
        ])
    );

    dataset.add_test_case(
        TestCase::new(
            "product-description",
            "Write a creative product description for a {{product}} that emphasizes its unique features."
        )
        .with_category("creative-writing")
        .add_variable("product", "smart coffee mug")
    );

    dataset.add_test_case(
        TestCase::new(
            "dialogue-scene",
            "Write a short dialogue between two characters meeting at a coffee shop. One character has exciting news to share."
        )
        .with_category("creative-writing")
    );

    dataset.add_test_case(
        TestCase::new(
            "metaphor-explanation",
            "Explain the concept of '{{concept}}' using a creative metaphor or analogy."
        )
        .with_category("creative-writing")
        .add_variable("concept", "cloud computing")
    );

    dataset.add_test_case(
        TestCase::new(
            "limerick",
            "Write a limerick about {{subject}}."
        )
        .with_category("creative-writing")
        .add_variable("subject", "programming bugs")
    );

    dataset
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_valid::Validate;

    #[test]
    fn test_get_builtin_datasets() {
        let datasets = get_builtin_datasets();
        assert_eq!(datasets.len(), 5);
    }

    #[test]
    fn test_coding_tasks() {
        let dataset = coding_tasks();
        assert_eq!(dataset.name, "coding-tasks");
        assert!(dataset.test_cases.len() >= 5);
        assert!(dataset.defaults.is_some());

        // Validate dataset
        assert!(dataset.validate().is_ok());

        // Check for template variables
        let has_templates = dataset.test_cases.iter()
            .any(|tc| tc.variables.is_some());
        assert!(has_templates, "Coding tasks should have template variables");
    }

    #[test]
    fn test_reasoning_tasks() {
        let dataset = reasoning_tasks();
        assert_eq!(dataset.name, "reasoning-tasks");
        assert!(dataset.test_cases.len() >= 5);
        assert!(dataset.validate().is_ok());
    }

    #[test]
    fn test_summarization_tasks() {
        let dataset = summarization_tasks();
        assert_eq!(dataset.name, "summarization-tasks");
        assert!(dataset.test_cases.len() >= 3);
        assert!(dataset.validate().is_ok());
    }

    #[test]
    fn test_instruction_following() {
        let dataset = instruction_following();
        assert_eq!(dataset.name, "instruction-following");
        assert!(dataset.test_cases.len() >= 5);
        assert!(dataset.validate().is_ok());
    }

    #[test]
    fn test_creative_writing() {
        let dataset = creative_writing();
        assert_eq!(dataset.name, "creative-writing");
        assert!(dataset.test_cases.len() >= 5);
        assert!(dataset.validate().is_ok());
    }

    #[test]
    fn test_all_datasets_valid() {
        for dataset in get_builtin_datasets() {
            assert!(
                dataset.validate().is_ok(),
                "Dataset {} failed validation",
                dataset.name
            );
            assert!(!dataset.test_cases.is_empty(), "Dataset {} has no test cases", dataset.name);
        }
    }
}

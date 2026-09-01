/// Integration tests for the dictionary module.
///
/// These tests verify the Dictionary, Trie, Levenshtein distance,
/// and auto-correction work correctly as a whole.

use keybroad_core::{levenshtein_distance, BengaliEngine, Dictionary, LayoutType, WordSource};

// === Dictionary Trie Tests ===

#[test]
fn test_dictionary_load_embedded() {
    let dict = Dictionary::load_embedded();
    assert!(dict.word_count() > 0);
}

#[test]
fn test_dictionary_is_word_bangla() {
    let dict = Dictionary::load_embedded();
    assert!(dict.is_word("বাংলা"));
}

#[test]
fn test_dictionary_is_word_not_found() {
    let dict = Dictionary::load_embedded();
    assert!(!dict.is_word("বাংল"));  // Incomplete word
    assert!(!dict.is_word("xyz"));   // Non-existent word
}

#[test]
fn test_dictionary_insert_and_lookup() {
    let mut dict = Dictionary::new();
    dict.insert("কীবোর্ড");
    dict.insert("প্রযুক্তি");

    assert!(dict.is_word("কীবোর্ড"));
    assert!(dict.is_word("প্রযুক্তি"));
    assert!(!dict.is_word("কীবোর্ডি"));
}

#[test]
fn test_dictionary_word_count() {
    let mut dict = Dictionary::new();
    dict.insert("এক");
    dict.insert("দুই");
    dict.insert("এক"); // duplicate
    assert_eq!(dict.word_count(), 2);
}

// === Prefix Matching Tests ===

#[test]
fn test_prefix_matches_bangla() {
    let dict = Dictionary::load_embedded();
    let matches = dict.get_prefix_matches("বাং", 10);
    assert!(!matches.is_empty());
    // Should contain words starting with "বাং"
    for word in &matches {
        assert!(word.starts_with("বাং"));
    }
}

#[test]
fn test_prefix_matches_limit() {
    let dict = Dictionary::load_embedded();
    let matches = dict.get_prefix_matches("ক", 3);
    assert!(matches.len() <= 3);
}

#[test]
fn test_prefix_matches_no_match() {
    let dict = Dictionary::load_embedded();
    let matches = dict.get_prefix_matches("xyz", 10);
    assert!(matches.is_empty());
}

#[test]
fn test_prefix_matches_empty_prefix() {
    let dict = Dictionary::load_embedded();
    let matches = dict.get_prefix_matches("", 10);
    // Empty prefix should match all words (up to limit)
    assert_eq!(matches.len(), 10);
}

// === Levenshtein Distance Tests ===

#[test]
fn test_levenshtein_identical_strings() {
    assert_eq!(levenshtein_distance("abc", "abc"), 0);
}

#[test]
fn test_levenshtein_single_insert() {
    assert_eq!(levenshtein_distance("abc", "abcd"), 1);
}

#[test]
fn test_levenshtein_single_delete() {
    assert_eq!(levenshtein_distance("abcd", "abc"), 1);
}

#[test]
fn test_levenshtein_single_replace() {
    assert_eq!(levenshtein_distance("abc", "axc"), 1);
}

#[test]
fn test_levenshtein_empty_strings() {
    assert_eq!(levenshtein_distance("", ""), 0);
    assert_eq!(levenshtein_distance("", "abc"), 3);
    assert_eq!(levenshtein_distance("abc", ""), 3);
}

#[test]
fn test_levenshtein_bengali_chars() {
    assert_eq!(levenshtein_distance("বাংলা", "বাংল"), 1);
    assert_eq!(levenshtein_distance("ভাষা", "ভাষা"), 0);
    assert_eq!(levenshtein_distance("কীবোর্ড", "কীবোর্ডি"), 1);
}

#[test]
fn test_levenshtein_multiple_edits() {
    assert_eq!(levenshtein_distance("abc", "def"), 3);
    assert_eq!(levenshtein_distance("abc", "axcy"), 2);
}

// === Auto-Correction Tests ===

#[test]
fn test_corrections_close_match() {
    let mut dict = Dictionary::new();
    dict.insert("বাংলা");
    dict.insert("বাংলাদেশ");

    let corrections = dict.get_corrections("বাংল", 2);
    assert!(corrections.contains(&"বাংলা".to_string()));
}

#[test]
fn test_corrections_no_match() {
    let mut dict = Dictionary::new();
    dict.insert("বাংলা");

    let corrections = dict.get_corrections("xyz", 2);
    assert!(corrections.is_empty());
}

#[test]
fn test_corrections_respects_max_distance() {
    let mut dict = Dictionary::new();
    dict.insert("বাংলা");

    // Distance 1: should find "বাংলা" from "বাংল"
    let corrections_d1 = dict.get_corrections("বাংল", 1);
    assert!(corrections_d1.contains(&"বাংলা".to_string()));

    // Distance 0: should NOT find "বাংলা" from "বাংল"
    let corrections_d0 = dict.get_corrections("বাংল", 0);
    assert!(!corrections_d0.contains(&"বাংলা".to_string()));
}

#[test]
fn test_corrections_sorted_by_distance() {
    let mut dict = Dictionary::new();
    dict.insert("বাংলা");
    dict.insert("বাংলাদেশ");

    let corrections = dict.get_corrections("বাংল", 5);
    // "বাংলা" (distance 1) should come before "বাংলাদেশ" (distance 4)
    if let (Some(pos_a), Some(pos_b)) = (
        corrections.iter().position(|w| w == "বাংলা"),
        corrections.iter().position(|w| w == "বাংলাদেশ"),
    ) {
        assert!(pos_a < pos_b);
    }
}

#[test]
fn test_corrections_empty_word() {
    let dict = Dictionary::load_embedded();
    let corrections = dict.get_corrections("", 2);
    assert!(corrections.is_empty());
}

// === BengaliEngine Integration Tests ===

#[test]
fn test_engine_has_dictionary() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("বাং");
    // Engine should have dictionary loaded and return suggestions
    assert!(!suggestions.is_empty());
}

#[test]
fn test_engine_get_suggestions_prefix() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("ক");
    // Should have suggestions (prefix matches or corrections)
    assert!(!suggestions.is_empty());
    // At least some suggestions should start with "ক" (the prefix matches)
    let has_prefix_match = suggestions.iter().any(|s| s.word.starts_with("ক"));
    assert!(has_prefix_match);
}

#[test]
fn test_engine_get_suggestions_correction() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("বাংল");
    // Should have "বাংলা" as a correction
    let has_correction = suggestions.iter().any(|s| s.word == "বাংলা");
    assert!(has_correction);
}

#[test]
fn test_engine_get_suggestions_empty() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("");
    assert!(suggestions.is_empty());
}

#[test]
fn test_engine_get_suggestions_word_source() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("বাং");
    // All suggestions should come from dictionary
    for suggestion in &suggestions {
        assert_eq!(suggestion.source, WordSource::Dictionary);
    }
}

#[test]
fn test_engine_get_suggestions_score() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("বাং");
    // Prefix matches should have score 1.0
    for suggestion in &suggestions {
        if suggestion.word.starts_with("বাং") {
            assert_eq!(suggestion.score, 1.0);
        }
    }
}

#[test]
fn test_engine_set_dictionary() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);
    let mut new_dict = Dictionary::new();
    new_dict.insert("নতুন");

    engine.set_dictionary(new_dict);

    let suggestions = engine.get_suggestions("নতু");
    let has_word = suggestions.iter().any(|s| s.word == "নতুন");
    assert!(has_word);
}

#[test]
fn test_engine_suggestions_deduplication() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    let suggestions = engine.get_suggestions("বাংলা");
    // Should not have duplicate words
    let mut words: Vec<_> = suggestions.iter().map(|s| &s.word).collect();
    words.sort();
    words.dedup();
    assert_eq!(words.len(), suggestions.len());
}

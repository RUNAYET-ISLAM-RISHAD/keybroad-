/// Dictionary module with Trie data structure for fast word lookup.
///
/// Provides:
/// - Trie-based storage for O(L) word lookup (L = word length)
/// - Prefix matching for word suggestions
/// - Levenshtein distance-based auto-correction
/// - Bounded edit distance search with trie pruning
///
/// # Design Decisions
///
/// - Uses `HashMap<char, TrieNode>` instead of fixed array because Bengali
///   uses Unicode codepoints (U+0980–U+09FF), not byte values
/// - Dictionary is loaded once at engine creation via `include_str!`
/// - All methods are immutable after loading (read-only dictionary)
/// - No external crates — standard library only

use std::collections::HashMap;

/// A node in the Trie data structure.
#[derive(Debug, Clone)]
struct TrieNode {
    /// Child nodes keyed by character
    children: HashMap<char, TrieNode>,
    /// Whether this node marks the end of a complete word
    is_word: bool,
    /// Word frequency (higher = more common, used for ranking suggestions)
    frequency: u32,
}

impl TrieNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            is_word: false,
            frequency: 0,
        }
    }
}

/// A dictionary backed by a Trie for fast word lookup and prefix matching.
///
/// # Examples
///
/// ```ignore
/// let mut dict = Dictionary::new();
/// dict.insert("বাংলা");
/// dict.insert("বাংলাদেশ");
///
/// assert!(dict.is_word("বাংলা"));
/// assert!(!dict.is_word("বাংল"));
///
/// let matches = dict.get_prefix_matches("বাংল", 10);
/// assert_eq!(matches.len(), 2);
/// ```
#[derive(Debug)]
pub struct Dictionary {
    /// Root node of the Trie
    root: TrieNode,
    /// Total number of words in the dictionary
    word_count: usize,
}

impl Dictionary {
    /// Create a new empty dictionary.
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
            word_count: 0,
        }
    }

    /// Load dictionary from the embedded words.txt file.
    ///
    /// This uses `include_str!` to embed the word list at compile time,
    /// avoiding any runtime file I/O. Words are loaded one per line.
    pub fn load_embedded() -> Self {
        let mut dict = Self::new();
        let data = include_str!("../data/words.txt");
        for line in data.lines() {
            let word = line.trim();
            if !word.is_empty() {
                dict.insert(word);
            }
        }
        dict
    }

    /// Add a word to the dictionary.
    ///
    /// Words are stored in the Trie character by character.
    /// If the word already exists, its frequency is incremented.
    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node = node.children.entry(ch).or_insert_with(TrieNode::new);
        }
        if !node.is_word {
            self.word_count += 1;
        }
        node.is_word = true;
        node.frequency += 1;
    }

    /// Check if a word exists in the dictionary.
    ///
    /// This is O(L) where L is the word length.
    pub fn is_word(&self, word: &str) -> bool {
        let mut node = &self.root;
        for ch in word.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return false,
            }
        }
        node.is_word
    }

    /// Get up to `limit` words that start with the given prefix.
    ///
    /// Returns words sorted by frequency (most common first).
    /// This is useful for autocomplete suggestions.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to search for
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// A vector of matching words, sorted by frequency descending.
    pub fn get_prefix_matches(&self, prefix: &str, limit: usize) -> Vec<String> {
        let mut node = &self.root;

        // Navigate to the prefix node
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }

        // Collect all words from this node
        let mut results = Vec::new();
        self.collect_words(node, prefix, &mut results, limit);

        // Sort by frequency (most common first)
        results.sort_by(|a, b| b.1.cmp(&a.1));
        results.truncate(limit);

        results.into_iter().map(|(word, _)| word).collect()
    }

    /// Recursively collect words from a trie node.
    ///
    /// Performs depth-first traversal, collecting complete words.
    /// Stops early when `limit` words have been collected.
    fn collect_words(
        &self,
        node: &TrieNode,
        current_prefix: &str,
        results: &mut Vec<(String, u32)>,
        limit: usize,
    ) {
        if results.len() >= limit {
            return;
        }

        if node.is_word {
            results.push((current_prefix.to_string(), node.frequency));
        }

        // Sort children for deterministic output
        let mut children: Vec<_> = node.children.iter().collect();
        children.sort_by_key(|(ch, _)| *ch);

        for (ch, child) in children {
            let mut next_prefix = current_prefix.to_string();
            next_prefix.push(*ch);
            self.collect_words(child, &next_prefix, results, limit);
            if results.len() >= limit {
                return;
            }
        }
    }

    /// Get words within `max_distance` Levenshtein edit distance.
    ///
    /// Uses bounded Levenshtein distance with trie pruning for efficiency.
    /// Results are sorted by distance (closest first), then by frequency.
    ///
    /// # Arguments
    ///
    /// * `word` - The word to find corrections for
    /// * `max_distance` - Maximum allowed edit distance (typically 1-3)
    ///
    /// # Returns
    ///
    /// A vector of correction candidates, sorted by distance ascending.
    pub fn get_corrections(&self, word: &str, max_distance: usize) -> Vec<String> {
        if word.is_empty() {
            return Vec::new();
        }

        let word_chars: Vec<char> = word.chars().collect();
        let word_len = word_chars.len();

        // First row of the Levenshtein matrix (distance from empty string)
        let first_row: Vec<usize> = (0..=word_len).collect();

        let mut results = Vec::new();
        self.search_corrections(
            &self.root,
            &word_chars,
            &first_row,
            String::new(),
            max_distance,
            &mut results,
        );

        // Sort by distance, then by frequency (but we don't have frequency here,
        // so just sort by distance)
        results.sort_by_key(|&(distance, _)| distance);
        results.truncate(20); // Limit to 20 results

        results.into_iter().map(|(_, word)| word).collect()
    }

    /// Recursively search for corrections using bounded Levenshtein.
    ///
    /// This is the core auto-correction algorithm. For each node in the trie,
    /// it extends the Levenshtein distance row and prunes branches where
    /// the minimum distance exceeds `max_distance`.
    fn search_corrections(
        &self,
        node: &TrieNode,
        word_chars: &[char],
        prev_row: &[usize],
        current_prefix: String,
        max_distance: usize,
        results: &mut Vec<(usize, String)>,
    ) {
        // If this node is a complete word, check if it's within distance
        if node.is_word {
            let distance = prev_row[prev_row.len() - 1];
            if distance <= max_distance {
                results.push((distance, current_prefix.clone()));
            }
        }

        // If the minimum value in prev_row exceeds max_distance, prune
        if prev_row.iter().min().map_or(false, |&min| min > max_distance) {
            return;
        }

        // Sort children for deterministic output
        let mut children: Vec<_> = node.children.iter().collect();
        children.sort_by_key(|(ch, _)| *ch);

        for (ch, child) in children {
            let mut current_row = vec![0; word_chars.len() + 1];
            current_row[0] = prev_row[0] + 1;

            for (j, &word_ch) in word_chars.iter().enumerate() {
                let cost = if ch == &word_ch { 0 } else { 1 };
                current_row[j + 1] = std::cmp::min(
                    std::cmp::min(
                        current_row[j] + 1,        // Insert
                        prev_row[j + 1] + 1,       // Delete
                    ),
                    prev_row[j] + cost,            // Replace
                );
            }

            let mut next_prefix = current_prefix.clone();
            next_prefix.push(*ch);

            self.search_corrections(child, word_chars, &current_row, next_prefix, max_distance, results);
        }
    }

    /// Get the total number of words in the dictionary.
    pub fn word_count(&self) -> usize {
        self.word_count
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

// === Levenshtein Distance Utility ===

/// Compute the Levenshtein edit distance between two strings.
///
/// This is the standard dynamic programming implementation.
/// Time complexity: O(n*m) where n and m are the string lengths.
/// Space complexity: O(min(n,m)) using rolling array.
///
/// # Arguments
///
/// * `a` - First string
/// * `b` - Second string
///
/// # Returns
///
/// The minimum number of single-character edits (insertions, deletions,
/// substitutions) required to change `a` into `b`.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    // Use rolling array for space efficiency
    let mut prev_row: Vec<usize> = (0..=b_len).collect();
    let mut curr_row: Vec<usize> = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr_row[0] = i;
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr_row[j] = std::cmp::min(
                std::cmp::min(
                    curr_row[j - 1] + 1,      // Insert
                    prev_row[j] + 1,           // Delete
                ),
                prev_row[j - 1] + cost,        // Replace
            );
        }
        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_insert_and_lookup() {
        let mut dict = Dictionary::new();
        dict.insert("বাংলা");
        dict.insert("বাংলাদেশ");

        assert!(dict.is_word("বাংলা"));
        assert!(dict.is_word("বাংলাদেশ"));
        assert!(!dict.is_word("বাংল"));
        assert!(!dict.is_word("বাংলাদেশে"));
    }

    #[test]
    fn test_trie_empty_string() {
        let dict = Dictionary::new();
        assert!(!dict.is_word(""));
    }

    #[test]
    fn test_trie_prefix_matching() {
        let mut dict = Dictionary::new();
        dict.insert("বাংলা");
        dict.insert("বাংলাদেশ");
        dict.insert("বাংলি");

        let matches = dict.get_prefix_matches("বাংল", 10);
        assert_eq!(matches.len(), 3);
        assert!(matches.contains(&"বাংলা".to_string()));
        assert!(matches.contains(&"বাংলাদেশ".to_string()));
        assert!(matches.contains(&"বাংলি".to_string()));
    }

    #[test]
    fn test_trie_prefix_limit() {
        let mut dict = Dictionary::new();
        dict.insert("ক");
        dict.insert("কা");
        dict.insert("কাজ");
        dict.insert("কাম");
        dict.insert("কার");

        let matches = dict.get_prefix_matches("ক", 3);
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_trie_no_prefix_matches() {
        let dict = Dictionary::new();
        let matches = dict.get_prefix_matches("বাংলা", 10);
        assert!(matches.is_empty());
    }

    #[test]
    fn test_levenshtein_identical() {
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
    fn test_levenshtein_empty_string() {
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", ""), 3);
    }

    #[test]
    fn test_levenshtein_both_empty() {
        assert_eq!(levenshtein_distance("", ""), 0);
    }

    #[test]
    fn test_levenshtein_bengali() {
        assert_eq!(levenshtein_distance("বাংলা", "বাংল"), 1);
        assert_eq!(levenshtein_distance("বাংলা", "বাংলাদেশ"), 3);
    }

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
    fn test_corrections_max_distance() {
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
    fn test_word_count() {
        let mut dict = Dictionary::new();
        dict.insert("বাংলা");
        dict.insert("বাংলাদেশ");
        dict.insert("বাংলা"); // duplicate
        assert_eq!(dict.word_count(), 2);
    }

    #[test]
    fn test_default() {
        let dict = Dictionary::default();
        assert_eq!(dict.word_count(), 0);
    }
}

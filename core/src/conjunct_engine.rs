/// Smart Conjunct Engine with dedicated যুক্ত (join) key
/// Handles unlimited conjunct composition via join mode

use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub enum JoinState {
    Normal,
    JoinPending, // After যুক্ত tapped, waiting for next consonant
    JoinActive,  // Inside multi-consonant cluster
}

#[derive(Debug, Deserialize)]
struct ConjunctEntry {
    components: Vec<String>,
    unicode: String,
    frequency: f32,
}

pub struct ConjunctEngine {
    dictionary: HashMap<String, ConjunctEntry>,
    state: JoinState,
    cluster: Vec<char>, // Current cluster components
}

impl ConjunctEngine {
    pub fn new() -> Self {
        let dict = Self::load_dictionary();
        Self {
            dictionary: dict,
            state: JoinState::Normal,
            cluster: Vec::new(),
        }
    }

    fn load_dictionary() -> HashMap<String, ConjunctEntry> {
        let json = include_str!("../data/conjuncts.json");
        serde_json::from_str(json).unwrap_or_default()
    }

    pub fn enter_join_mode(&mut self) {
        if !self.cluster.is_empty() {
            self.state = JoinState::JoinPending;
        }
    }

    pub fn start_join(&mut self, last_char: u32) {
        if let Some(ch) = char::from_u32(last_char) {
            self.cluster = vec![ch];
        }
        self.state = JoinState::JoinPending;
    }

    pub fn is_join_pending(&self) -> bool {
        self.state == JoinState::JoinPending || self.state == JoinState::JoinActive
    }

    pub fn reset(&mut self) {
        self.state = JoinState::Normal;
        self.cluster.clear();
    }

    pub fn push_consonant(&mut self, ch: char) -> Option<String> {
        match self.state {
            JoinState::Normal => {
                self.cluster = vec![ch];
                None
            }
            JoinState::JoinPending | JoinState::JoinActive => {
                self.cluster.push(ch);
                self.state = JoinState::JoinActive;
                // Try to form conjunct from cluster
                let conjunct = self.form_conjunct();
                Some(conjunct)
            }
        }
    }

    pub fn push_vowel(&mut self) {
        // Vowel exits join mode
        self.state = JoinState::Normal;
        self.cluster.clear();
    }

    fn form_conjunct(&self) -> String {
        // Check dictionary for exact match
        let _key: String = self.cluster.iter().collect();
        // Try to find in dictionary by components
        // For now, construct via halant joining and check dictionary for frequency
        // Unlimited: join with halant
        let mut result = String::new();
        for (i, &ch) in self.cluster.iter().enumerate() {
            if i > 0 {
                result.push('\u{09CD}'); // hasanta
            }
            result.push(ch);
        }
        // Check if this conjunct is in dictionary for suggestions, but still return constructed
        // The dictionary is for prediction, not for restricting formation
        result
    }

    pub fn get_suggestions_for_join(&self) -> Vec<String> {
        if self.cluster.is_empty() {
            return vec![];
        }
        let prefix: String = self.cluster.iter().collect();
        // Find all conjuncts that start with this prefix
        let mut suggestions = Vec::new();
        for (key, entry) in &self.dictionary {
            if key.starts_with(&prefix) || entry.unicode.starts_with(&prefix) {
                suggestions.push(entry.unicode.clone());
            }
            // Also check components prefix
            let comp_str: String = entry.components.join("");
            if comp_str.starts_with(&prefix) {
                suggestions.push(entry.unicode.clone());
            }
        }
        // Deduplicate and sort by frequency
        suggestions.sort_by(|a,b| {
            let fa = self.dictionary.get(a).map(|e| e.frequency).unwrap_or(0.0);
            let fb = self.dictionary.get(b).map(|e| e.frequency).unwrap_or(0.0);
            fb.partial_cmp(&fa).unwrap()
        });
        suggestions.dedup();
        suggestions.into_iter().take(5).collect()
    }

    pub fn get_state(&self) -> &JoinState {
        &self.state
    }
}

impl Default for ConjunctEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_mode() {
        let mut eng = ConjunctEngine::new();
        eng.push_consonant('ক');
        assert_eq!(eng.get_state(), &JoinState::Normal);
        eng.enter_join_mode();
        assert_eq!(eng.get_state(), &JoinState::JoinPending);
        let conj = eng.push_consonant('ষ').unwrap();
        assert_eq!(conj, "ক\u{09CD}ষ"); // ক্ষ
        assert_eq!(eng.get_state(), &JoinState::JoinActive);
    }

    #[test]
    fn test_multi_conjunct() {
        let mut eng = ConjunctEngine::new();
        eng.push_consonant('ক');
        eng.enter_join_mode();
        eng.push_consonant('ত');
        let conj = eng.push_consonant('র').unwrap();
        assert_eq!(conj, "ক\u{09CD}ত\u{09CD}র"); // ক্ত্র
    }

    #[test]
    fn test_vowel_exits_join() {
        let mut eng = ConjunctEngine::new();
        eng.push_consonant('ক');
        eng.enter_join_mode();
        eng.push_consonant('ষ');
        eng.push_vowel();
        assert_eq!(eng.get_state(), &JoinState::Normal);
    }

    #[test]
    fn test_suggestions_on_join() {
        let mut eng = ConjunctEngine::new();
        eng.push_consonant('ক');
        eng.enter_join_mode();
        let sug = eng.get_suggestions_for_join();
        assert!(sug.contains(&"ক্ষ".to_string()) || sug.iter().any(|s| s.contains('ক')));
    }
}

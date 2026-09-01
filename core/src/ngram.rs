use std::collections::HashMap;
use crate::types::{CandidateWord, WordSource};

/// N-gram prediction model using bigrams and trigrams.
/// Trains on an embedded Bengali corpus and predicts next words.
pub struct NgramModel {
    /// Maps a word to its possible next words with frequency counts
    bigrams: HashMap<String, Vec<(String, u32)>>,
    /// Maps (word1, word2) to possible next words with frequency counts
    trigrams: HashMap<(String, String), Vec<(String, u32)>>,
    /// Total number of sentences trained on
    sentence_count: usize,
}

impl NgramModel {
    /// Create an empty NgramModel (no training data).
    pub fn new() -> Self {
        Self {
            bigrams: HashMap::new(),
            trigrams: HashMap::new(),
            sentence_count: 0,
        }
    }

    /// Load the embedded corpus and build frequency tables.
    pub fn load_embedded() -> Self {
        let corpus = include_str!("../data/corpus.txt");
        let mut model = Self::new();
        for line in corpus.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let words: Vec<String> = line
                .split_whitespace()
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect();
            model.train_sentence(&words);
        }
        model
    }

    /// Train the model on a single sentence (sequence of words).
    pub fn train_sentence(&mut self, words: &[String]) {
        if words.len() < 2 {
            return;
        }
        self.sentence_count += 1;

        // Build bigrams: word[i] -> word[i+1]
        for i in 0..words.len() - 1 {
            let prev = &words[i];
            let next = &words[i + 1];
            self.bigrams
                .entry(prev.clone())
                .or_default()
                .push((next.clone(), 1));
        }

        // Build trigrams: (word[i], word[i+1]) -> word[i+2]
        for i in 0..words.len() - 2 {
            let key = (words[i].clone(), words[i + 1].clone());
            let next = &words[i + 2];
            self.trigrams
                .entry(key)
                .or_default()
                .push((next.clone(), 1));
        }

        // Merge duplicate counts
        self.merge_counts();
    }

    /// Merge duplicate entries and sort by frequency (descending).
    fn merge_counts(&mut self) {
        for counts in self.bigrams.values_mut() {
            // Sum counts for duplicate words
            let mut map: HashMap<String, u32> = HashMap::new();
            for (word, count) in counts.drain(..) {
                *map.entry(word).or_insert(0) += count;
            }
            let mut merged: Vec<(String, u32)> = map.into_iter().collect();
            merged.sort_by(|a, b| b.1.cmp(&a.1));
            *counts = merged;
        }
        for counts in self.trigrams.values_mut() {
            let mut map: HashMap<String, u32> = HashMap::new();
            for (word, count) in counts.drain(..) {
                *map.entry(word).or_insert(0) += count;
            }
            let mut merged: Vec<(String, u32)> = map.into_iter().collect();
            merged.sort_by(|a, b| b.1.cmp(&a.1));
            *counts = merged;
        }
    }

    /// Predict the next word(s) given a context of previous words.
    ///
    /// # Arguments
    /// * `context` - The previous words (most recent last)
    /// * `limit` - Maximum number of suggestions to return
    ///
    /// # Returns
    /// Vec of CandidateWord with scores normalized to [0.0, 1.0]
    pub fn predict_next(&self, context: &[String], limit: usize) -> Vec<CandidateWord> {
        if context.is_empty() || limit == 0 {
            return Vec::new();
        }

        // Try trigram first if we have at least 2 context words
        if context.len() >= 2 {
            let key = (context[context.len() - 2].clone(), context[context.len() - 1].clone());
            if let Some(candidates) = self.trigrams.get(&key) {
                if !candidates.is_empty() {
                    let max_count = candidates[0].1 as f32;
                    return candidates
                        .iter()
                        .take(limit)
                        .map(|(word, count)| CandidateWord {
                            word: word.clone(),
                            score: *count as f32 / max_count,
                            source: WordSource::AiPrediction,
                        })
                        .collect();
                }
            }
        }

        // Fallback to bigram
        if let Some(candidates) = self.bigrams.get(&context[context.len() - 1]) {
            if !candidates.is_empty() {
                let max_count = candidates[0].1 as f32;
                return candidates
                    .iter()
                    .take(limit)
                    .map(|(word, count)| CandidateWord {
                        word: word.clone(),
                        score: *count as f32 / max_count,
                        source: WordSource::AiPrediction,
                    })
                    .collect();
            }
        }

        Vec::new()
    }

    /// Get the number of sentences trained on.
    pub fn sentence_count(&self) -> usize {
        self.sentence_count
    }

    /// Check if the model has any training data.
    pub fn is_empty(&self) -> bool {
        self.sentence_count == 0
    }

    /// Get the number of unique bigram contexts.
    pub fn bigram_count(&self) -> usize {
        self.bigrams.len()
    }

    /// Get the number of unique trigram contexts.
    pub fn trigram_count(&self) -> usize {
        self.trigrams.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ngram_load_embedded() {
        let model = NgramModel::load_embedded();
        assert!(!model.is_empty());
        assert!(model.sentence_count() > 50);
        assert!(model.bigram_count() > 50);
        assert!(model.trigram_count() > 20);
    }

    #[test]
    fn test_ngram_empty_model() {
        let model = NgramModel::new();
        let preds = model.predict_next(&["test".to_string()], 5);
        assert!(preds.is_empty());
    }

    #[test]
    fn test_ngram_empty_context() {
        let model = NgramModel::load_embedded();
        let preds = model.predict_next(&[], 5);
        assert!(preds.is_empty());
    }

    #[test]
    fn test_ngram_bigram_prediction() {
        let model = NgramModel::load_embedded();
        let context = vec!["আমি".to_string()];
        let preds = model.predict_next(&context, 5);
        // Should find predictions for "আমি" (common word in corpus)
        assert!(!preds.is_empty());
        // All scores should be between 0.0 and 1.0
        for pred in &preds {
            assert!(pred.score > 0.0 && pred.score <= 1.0);
            assert_eq!(pred.source, WordSource::AiPrediction);
        }
    }

    #[test]
    fn test_ngram_trigram_prediction() {
        let model = NgramModel::load_embedded();
        let context = vec!["আমি".to_string(), "বাংলায়".to_string()];
        let preds = model.predict_next(&context, 5);
        // Should find predictions for "আমি বাংলায়" context
        if !preds.is_empty() {
            for pred in &preds {
                assert!(pred.score > 0.0 && pred.score <= 1.0);
                assert_eq!(pred.source, WordSource::AiPrediction);
            }
        }
    }

    #[test]
    fn test_ngram_fallback_to_bigram() {
        let mut model = NgramModel::new();
        // Train with bigram data only
        model.train_sentence(&["hello".to_string(), "world".to_string()]);
        // Query with trigram context - should fallback to bigram
        let context = vec!["unknown".to_string(), "hello".to_string()];
        let preds = model.predict_next(&context, 5);
        // Should find "world" as a prediction via bigram fallback
        assert_eq!(preds.len(), 1);
        assert_eq!(preds[0].word, "world");
    }

    #[test]
    fn test_ngram_unknown_context() {
        let model = NgramModel::load_embedded();
        let context = vec!["xyznonexistent".to_string()];
        let preds = model.predict_next(&context, 5);
        assert!(preds.is_empty());
    }

    #[test]
    fn test_ngram_limit() {
        let mut model = NgramModel::new();
        model.train_sentence(&["a".to_string(), "b".to_string()]);
        model.train_sentence(&["a".to_string(), "c".to_string()]);
        model.train_sentence(&["a".to_string(), "d".to_string()]);

        let context = vec!["a".to_string()];
        let preds = model.predict_next(&context, 2);
        assert!(preds.len() <= 2);
    }

    #[test]
    fn test_ngram_train_sentence() {
        let mut model = NgramModel::new();
        assert!(model.is_empty());

        model.train_sentence(&["নমস্কার".to_string(), "আপনি".to_string(), "কেমন".to_string(), "আছেন".to_string()]);
        assert!(!model.is_empty());
        assert_eq!(model.sentence_count(), 1);

        let context = vec!["নমস্কার".to_string()];
        let preds = model.predict_next(&context, 5);
        assert!(!preds.is_empty());
        assert_eq!(preds[0].word, "আপনি");
    }

    #[test]
    fn test_ngram_score_range() {
        let mut model = NgramModel::new();
        model.train_sentence(&["a".to_string(), "b".to_string(), "c".to_string()]);
        model.train_sentence(&["a".to_string(), "d".to_string()]);

        let context = vec!["a".to_string()];
        let preds = model.predict_next(&context, 5);
        for pred in &preds {
            assert!(pred.score >= 0.0 && pred.score <= 1.0);
        }
    }

    #[test]
    fn test_ngram_trigram_beats_bigram() {
        let mut model = NgramModel::new();
        model.train_sentence(&["x".to_string(), "y".to_string(), "z".to_string()]);
        model.train_sentence(&["w".to_string(), "y".to_string(), "q".to_string()]);

        // Trigram context (x, y) should return z
        let context = vec!["x".to_string(), "y".to_string()];
        let preds = model.predict_next(&context, 5);
        assert!(!preds.is_empty());
        assert_eq!(preds[0].word, "z");
    }

    #[test]
    fn test_ngram_merge_counts() {
        let mut model = NgramModel::new();
        model.train_sentence(&["a".to_string(), "b".to_string()]);
        model.train_sentence(&["a".to_string(), "b".to_string()]);
        model.train_sentence(&["a".to_string(), "b".to_string()]);
        model.train_sentence(&["a".to_string(), "c".to_string()]);

        let context = vec!["a".to_string()];
        let preds = model.predict_next(&context, 5);
        // "b" should appear with higher score than "c"
        assert!(preds.len() >= 2);
        let b_pred = preds.iter().find(|p| p.word == "b").unwrap();
        let c_pred = preds.iter().find(|p| p.word == "c").unwrap();
        assert!(b_pred.score > c_pred.score);
    }
}

/// Core Bengali keyboard engine.
///
/// The engine is designed as a pure functional component:
/// - Same input always produces same output (deterministic)
/// - No I/O (no network, no file access, no UI)
/// - No garbage collection pauses (uses arena-style allocation in production)
///
/// This implementation supports:
/// - Layout-based key mapping (loaded from JSON)
/// - Hasanta (্) handling for conjunct formation via backspace trick
/// - Basic shift handling
/// - Backspace and enter
/// - Composition buffer management
///
/// # Conjunct Formation (Backspace Trick)
///
/// Bengali conjuncts are formed using hasanta (virama):
/// 1. Consonant is committed immediately (e.g., ক)
/// 2. When hasanta is typed after a consonant, a Backspace(1) action is
///    emitted to remove the consonant, and hasanta_pending is set.
/// 3. When the next consonant arrives, the full conjunct sequence is
///    committed (e.g., ক্গ = ক + ্ + গ).
/// 4. If the next character is not a consonant, the hasanta state is
///    cleared and the hasanta character is committed.

use std::collections::{HashMap, HashSet};

use crate::conjunct::ConjunctTable;
use crate::dictionary::Dictionary;
use crate::layout::load_layout;
use crate::ngram::NgramModel;
use crate::types::*;

/// Maximum number of words to keep in history
const MAX_HISTORY: usize = 20;

/// Characters that trigger word boundary finalization
const WORD_BOUNDARY_CHARS: &[char] = &[' ', '\n', '.', ',', '!', '?', ';', ':'];

/// The main Bengali keyboard engine.
pub struct BengaliEngine {
    /// Current engine state
    state: EngineState,
    /// Loaded layouts (keyed by LayoutType)
    layouts: HashMap<LayoutType, Layout>,
    /// Conjunct table for consonant classification
    conjuncts: ConjunctTable,
    /// Dictionary for word suggestions and auto-correction
    dictionary: Option<Dictionary>,
    /// N-gram prediction model
    ngram: NgramModel,
    /// User-added words (runtime, mutable)
    user_words: HashSet<String>,
}

impl BengaliEngine {
    /// Create a new engine instance with the given layout.
    ///
    /// This loads the layout from the embedded JSON definition.
    pub fn new(layout: LayoutType) -> Self {
        let mut layouts = HashMap::new();

        // Load the requested layout
        if let Ok(loaded_layout) = load_layout(layout) {
            layouts.insert(layout, loaded_layout);
        }

        // Also pre-load English as fallback
        if layout != LayoutType::English {
            if let Ok(english_layout) = load_layout(LayoutType::English) {
                layouts.insert(LayoutType::English, english_layout);
            }
        }

        Self {
            state: EngineState {
                layout,
                ..Default::default()
            },
            layouts,
            conjuncts: ConjunctTable::new(),
            dictionary: Some(Dictionary::load_embedded()),
            ngram: NgramModel::load_embedded(),
            user_words: HashSet::new(),
        }
    }

    /// Create a new engine from a layout string identifier.
    pub fn from_layout_str(layout_str: &str) -> Option<Self> {
        LayoutType::from_str(layout_str).map(|layout| Self::new(layout))
    }

    /// Process a single key event and return actions for the platform.
    pub fn process_key(&mut self, event: KeyEvent) -> Result<Vec<OutputAction>, EngineError> {
        // Only process key-down events
        if !event.is_down {
            return Ok(vec![OutputAction::Nothing]);
        }

        // Handle special keys by key_code
        match event.key_code {
            // Backspace key
            67 => {
                return Ok(self.handle_backspace());
            }
            // Enter key (commit composition)
            66 => {
                return Ok(self.handle_enter());
            }
            // Shift keys (left and right)
            59 | 60 => {
                self.state.shift_state = match self.state.shift_state {
                    ShiftState::None => ShiftState::Shift,
                    ShiftState::Shift => ShiftState::None,
                    ShiftState::CapsLock => ShiftState::CapsLock,
                };
                return Ok(vec![OutputAction::Nothing]);
            }
            _ => {}
        }

        // Get the current layout
        let output_str = {
            let layout = self.layouts.get(&self.state.layout).ok_or_else(|| {
                EngineError::LayoutLoadError(format!(
                    "Layout '{}' not loaded",
                    self.state.layout.as_str()
                ))
            })?;

            // Determine if shift is active
            let shift_active = self.state.shift_state == ShiftState::Shift
                || self.state.shift_state == ShiftState::CapsLock;

            // Look up the character in the layout
            layout
                .lookup(event.unicode, shift_active)
                .ok_or_else(|| EngineError::InvalidKeyCode(event.unicode))?
                .to_string()
        };

        // Reset shift after single use (if not caps lock)
        if self.state.shift_state == ShiftState::Shift {
            self.state.shift_state = ShiftState::None;
        }

        // Process the output character through the composition engine
        let actions = self.process_character(&output_str)?;

        Ok(actions)
    }

    /// Process a character string from the layout lookup.
    ///
    /// This handles hasanta-based conjunct formation using the backspace trick.
    fn process_character(&mut self, output_str: &str) -> Result<Vec<OutputAction>, EngineError> {
        let mut actions = Vec::new();

        // Check if this is a word boundary character (single char like space or punctuation)
        if output_str.len() == 1 {
            if let Some(ch) = output_str.chars().next() {
                if WORD_BOUNDARY_CHARS.contains(&ch) {
                    // Finalize current word before processing boundary char
                    self.finalize_current_word();
                }
            }
        }

        for ch in output_str.chars() {
            let char_actions = self.process_single_char(ch)?;
            actions.extend(char_actions);

            // Track word boundary: append regular characters to current_word
            if !WORD_BOUNDARY_CHARS.contains(&ch) {
                self.state.current_word.push(ch);
            }
        }

        Ok(actions)
    }

    /// Process a single character through the composition engine.
    ///
    /// This is the core composition logic using the backspace trick:
    /// 1. If hasanta is pending and new char is consonant → commit conjunct sequence
    /// 2. If new char is hasanta after consonant → backspace + set hasanta_pending
    /// 3. Otherwise → commit character directly
    fn process_single_char(&mut self, ch: char) -> Result<Vec<OutputAction>, EngineError> {
        let mut actions = Vec::new();

        // Check if this is a hasanta character
        if ConjunctTable::is_hasanta(ch) {
            // Hasanta typed after a consonant → backspace trick
            if self.state.hasanta_base_consonant.is_some() {
                // Remove the previously committed consonant from composition buffer
                self.state.composition_buffer.pop();
                // Emit Backspace(1) to tell platform to remove the character
                actions.push(OutputAction::Backspace(1));
                self.state.hasanta_pending = true;
                // Don't output hasanta yet — wait for next consonant
                return Ok(actions);
            }
            // Hasanta without preceding consonant → output directly
            actions.push(OutputAction::CommitText(ch.to_string()));
            self.add_to_buffer(ch);
            return Ok(actions);
        }

        // Check if this character is a consonant
        let is_consonant = self.conjuncts.is_consonant(ch);

        // Case 1: Hasanta is pending and new char is a consonant → form conjunct
        if self.state.hasanta_pending && is_consonant {
            if let Some(base) = self.state.hasanta_base_consonant {
                // Check if there's a special conjunct form
                let seq = if let Some(conjunct_str) = self.conjuncts.lookup_hasanta_conjunct(base, ch) {
                    conjunct_str.to_string()
                } else {
                    // Default: output base + hasanta + consonant
                    format!("{}{}{}", base, ConjunctTable::hasanta(), ch)
                };

                actions.push(OutputAction::CommitText(seq.clone()));
                for c in seq.chars() {
                    self.add_to_buffer(c);
                }

                // Clear hasanta state
                self.state.hasanta_pending = false;
                self.state.hasanta_base_consonant = None;
                return Ok(actions);
            }
        }

        // Case 2: Consonant typed → commit immediately, remember for potential hasanta
        if is_consonant {
            actions.push(OutputAction::CommitText(ch.to_string()));
            self.add_to_buffer(ch);
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = Some(ch);
            return Ok(actions);
        }

        // Case 3: Non-consonant (vowel, digit, etc.) → commit directly
        // Clear hasanta state if not consumed
        if self.state.hasanta_pending {
            // Hasanta was pending but next char is not a consonant
            // Output the hasanta character, then the new character
            actions.push(OutputAction::CommitText(ConjunctTable::hasanta().to_string()));
            self.add_to_buffer(ConjunctTable::hasanta());
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
        }

        actions.push(OutputAction::CommitText(ch.to_string()));
        self.add_to_buffer(ch);

        Ok(actions)
    }

    /// Handle backspace key press.
    fn handle_backspace(&mut self) -> Vec<OutputAction> {
        // Clear hasanta state if pending
        if self.state.hasanta_pending {
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
            return vec![OutputAction::Nothing];
        }

        // Clear base consonant if set (consonant was typed but no hasanta followed)
        if self.state.hasanta_base_consonant.is_some() {
            self.state.hasanta_base_consonant = None;
        }

        // Remove from composition buffer
        if !self.state.composition_buffer.is_empty() {
            self.state.composition_buffer.pop();
            // Also remove last character from current_word if tracking
            self.state.current_word.pop();
            return vec![OutputAction::Backspace(1)];
        }

        vec![OutputAction::Nothing]
    }

    /// Handle enter key press.
    fn handle_enter(&mut self) -> Vec<OutputAction> {
        let mut actions = Vec::new();

        // Finalize current word before enter
        self.finalize_current_word();

        // If hasanta was pending, output it before committing
        if self.state.hasanta_pending {
            actions.push(OutputAction::CommitText(ConjunctTable::hasanta().to_string()));
            self.add_to_buffer(ConjunctTable::hasanta());
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
        }

        // Commit composition buffer if not empty
        if !self.state.composition_buffer.is_empty() {
            let text = self.composition_to_string();
            self.state.composition_buffer.clear();
            actions.push(OutputAction::Backspace(1));
            actions.push(OutputAction::CommitText(text));
        }

        if actions.is_empty() {
            actions.push(OutputAction::Nothing);
        }

        actions
    }

    /// Get a reference to the current engine state.
    pub fn get_state(&self) -> &EngineState {
        &self.state
    }

    /// Get a mutable reference to the engine state (for advanced operations).
    pub fn get_state_mut(&mut self) -> &mut EngineState {
        &mut self.state
    }

    /// Reset the engine state (for new input field).
    pub fn reset(&mut self) {
        self.state.composition_buffer.clear();
        self.state.candidates.clear();
        self.state.cursor_position = 0;
        self.state.shift_state = ShiftState::None;
        self.state.pending_char = None;
        self.state.hasanta_pending = false;
        self.state.hasanta_base_consonant = None;
        self.state.current_word.clear();
        self.state.history.clear();
        self.state.last_committed_word = None;
    }

    /// Switch to a different layout.
    pub fn set_layout(&mut self, layout: LayoutType) {
        // Load the layout if not already loaded
        if !self.layouts.contains_key(&layout) {
            if let Ok(loaded_layout) = load_layout(layout) {
                self.layouts.insert(layout, loaded_layout);
            }
        }

        self.state.layout = layout;
        self.reset();
    }

    /// Enable or disable incognito mode.
    pub fn set_incognito(&mut self, enabled: bool) {
        self.state.incognito_mode = enabled;
    }

    /// Check if incognito mode is active.
    pub fn is_incognito(&self) -> bool {
        self.state.incognito_mode
    }

    /// Get current candidates.
    pub fn get_candidates(&self) -> &[CandidateWord] {
        &self.state.candidates
    }

    /// Get a reference to a loaded layout.
    pub fn get_layout(&self, layout_type: LayoutType) -> Option<&Layout> {
        self.layouts.get(&layout_type)
    }

    /// Get a reference to the current active layout.
    pub fn get_active_layout(&self) -> Option<&Layout> {
        self.layouts.get(&self.state.layout)
    }

    /// Get a reference to the conjunct table.
    pub fn get_conjuncts(&self) -> &ConjunctTable {
        &self.conjuncts
    }

    /// Replace the dictionary with a new one.
    ///
    /// This allows loading a custom dictionary at runtime.
    pub fn set_dictionary(&mut self, dictionary: Dictionary) {
        self.dictionary = Some(dictionary);
    }

    /// Get word suggestions for the current input.
    ///
    /// Returns prefix matches and corrections combined, with corrections
    /// prioritized when the current word is not in the dictionary.
    /// Also checks user dictionary for exact matches.
    ///
    /// # Arguments
    ///
    /// * `current_word` - The partial word being typed
    ///
    /// # Returns
    ///
    /// A vector of `CandidateWord` suggestions, sorted by relevance.
    pub fn get_suggestions(&self, current_word: &str) -> Vec<CandidateWord> {
        if current_word.is_empty() {
            return Vec::new();
        }

        let mut suggestions = Vec::new();

        // 1. Check user dictionary first (highest priority)
        if self.user_words.contains(current_word) {
            suggestions.push(CandidateWord {
                word: current_word.to_string(),
                score: 1.0,
                source: WordSource::UserHistory,
            });
        }

        // 2. Get prefix matches from main dictionary (up to 5)
        if let Some(ref dict) = self.dictionary {
            let prefix_matches = dict.get_prefix_matches(current_word, 5);
            for word in prefix_matches {
                if !suggestions.iter().any(|s| s.word == word) {
                    suggestions.push(CandidateWord {
                        word,
                        score: 1.0,
                        source: WordSource::Dictionary,
                    });
                }
            }

            // 3. Get corrections if word is not in dictionary
            if !dict.is_word(current_word) && !self.user_words.contains(current_word) {
                let corrections = dict.get_corrections(current_word, 2);
                for word in corrections {
                    if !suggestions.iter().any(|s| s.word == word) {
                        suggestions.push(CandidateWord {
                            word,
                            score: 0.8,
                            source: WordSource::Dictionary,
                        });
                    }
                }
            }
        }

        suggestions
    }

    // === Private helpers ===

    /// Finalize the current word and update history.
    ///
    /// This is called when a word boundary is detected (space, punctuation, enter).
    /// If incognito mode is active, history and last_committed_word are not updated.
    fn finalize_current_word(&mut self) {
        if self.state.current_word.is_empty() {
            return;
        }

        let word = self.state.current_word.clone();

        // Update history and last_committed_word (unless incognito mode is active)
        if !self.state.incognito_mode {
            self.state.history.insert(0, word.clone());
            // Cap history at MAX_HISTORY
            if self.state.history.len() > MAX_HISTORY {
                self.state.history.truncate(MAX_HISTORY);
            }
            self.state.last_committed_word = Some(word);
        }

        // Clear current_word for next word
        self.state.current_word.clear();
    }

    /// Manually finalize the current word (useful for UI trigger like "send" button).
    pub fn finalize_word(&mut self) {
        self.finalize_current_word();
    }

    /// Add a word to the user dictionary.
    pub fn add_user_word(&mut self, word: &str) {
        self.user_words.insert(word.to_string());
    }

    /// Check if a word is in the user dictionary.
    pub fn is_user_word(&self, word: &str) -> bool {
        self.user_words.contains(word)
    }

    /// Get next-word suggestions based on n-gram context.
    ///
    /// Uses the last 1-2 words from history to predict what comes next.
    pub fn get_next_word_suggestions(&self, limit: usize) -> Vec<CandidateWord> {
        if self.state.incognito_mode {
            return Vec::new();
        }

        let mut context = Vec::new();

        // Build context from last_committed_word and history
        if let Some(ref last_word) = self.state.last_committed_word {
            context.push(last_word.clone());
            // If we have history, use the second-to-last word for trigram
            if !self.state.history.is_empty() {
                context.insert(0, self.state.history[0].clone());
            }
        }

        if context.is_empty() {
            return Vec::new();
        }

        self.ngram.predict_next(&context, limit)
    }

    // === Private helpers ===

    /// Add a character to the composition buffer.
    fn add_to_buffer(&mut self, ch: char) {
        let glyph = Glyph::simple(ch as u32);
        self.state.composition_buffer.push(glyph);
    }

    /// Convert composition buffer to a string.
    fn composition_to_string(&self) -> String {
        self.state
            .composition_buffer
            .iter()
            .filter_map(|g| char::from_u32(g.unicode))
            .collect()
    }
}

impl Default for BengaliEngine {
    fn default() -> Self {
        Self::new(LayoutType::Phonetic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = BengaliEngine::new(LayoutType::Phonetic);
        assert_eq!(engine.get_state().layout, LayoutType::Phonetic);
        assert!(engine.get_active_layout().is_some());
    }

    #[test]
    fn test_engine_from_layout_str() {
        let engine = BengaliEngine::from_layout_str("phonetic");
        assert!(engine.is_some());
        assert_eq!(engine.unwrap().get_state().layout, LayoutType::Phonetic);

        let invalid = BengaliEngine::from_layout_str("invalid");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_process_key_returns_action() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        let event = KeyEvent::down(29, 'a' as u32);
        let result = engine.process_key(event);
        assert!(result.is_ok());

        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_backspace_key() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();

        let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], OutputAction::Backspace(1));
    }

    #[test]
    fn test_backspace_empty_buffer() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], OutputAction::Nothing);
    }

    #[test]
    fn test_key_up_ignored() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        let event = KeyEvent::new(29, 'a' as u32, false, 0);
        let actions = engine.process_key(event).unwrap();
        assert_eq!(actions[0], OutputAction::Nothing);
    }

    #[test]
    fn test_shift_toggle() {
        let mut engine = BengaliEngine::new(LayoutType::English);

        let actions = engine.process_key(KeyEvent::down(59, 0)).unwrap();
        assert_eq!(actions[0], OutputAction::Nothing);
        assert_eq!(engine.get_state().shift_state, ShiftState::Shift);

        let actions = engine.process_key(KeyEvent::down(59, 0)).unwrap();
        assert_eq!(actions[0], OutputAction::Nothing);
        assert_eq!(engine.get_state().shift_state, ShiftState::None);
    }

    #[test]
    fn test_reset() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
        engine.process_key(KeyEvent::down(30, 'b' as u32)).unwrap();

        engine.reset();
        assert!(engine.get_state().composition_buffer.is_empty());
        assert_eq!(engine.get_state().cursor_position, 0);
    }

    #[test]
    fn test_set_layout() {
        let mut engine = BengaliEngine::new(LayoutType::Phonetic);
        assert_eq!(engine.get_state().layout, LayoutType::Phonetic);

        engine.set_layout(LayoutType::English);
        assert_eq!(engine.get_state().layout, LayoutType::English);
    }

    #[test]
    fn test_incognito_mode() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        assert!(!engine.is_incognito());

        engine.set_incognito(true);
        assert!(engine.is_incognito());
    }

    #[test]
    fn test_conjunct_table_loaded() {
        let engine = BengaliEngine::new(LayoutType::Phonetic);
        let conjuncts = engine.get_conjuncts();

        assert!(conjuncts.is_consonant('ক'));
        assert!(conjuncts.is_consonant('খ'));
        assert!(!conjuncts.is_consonant('া'));
    }
}

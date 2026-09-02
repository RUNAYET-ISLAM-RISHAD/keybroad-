use std::collections::{HashMap, HashSet};

use crate::conjunct::ConjunctTable;
use crate::conjunct_engine::ConjunctEngine;
use crate::dictionary::Dictionary;
use crate::layout::load_layout;
use crate::ngram::NgramModel;
use crate::phonetic::PhoneticEngine;
use crate::types::*;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

const MAX_HISTORY: usize = 20;
const WORD_BOUNDARY_CHARS: &[char] = &[' ', '\n', '.', ',', '!', '?', ';', ':'];

pub struct BengaliEngine {
    state: EngineState,
    layouts: HashMap<LayoutType, Layout>,
    conjuncts: ConjunctTable,
    dictionary: Option<Dictionary>,
    ngram: NgramModel,
    user_words: HashSet<String>,
    phonetic: PhoneticEngine,
    conjunct_engine: ConjunctEngine,
}

impl BengaliEngine {
    pub fn new(layout: LayoutType) -> Self {
        let mut layouts = HashMap::new();
        if let Ok(loaded_layout) = load_layout(layout) {
            layouts.insert(layout, loaded_layout);
        }
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
            phonetic: PhoneticEngine::new(),
            conjunct_engine: ConjunctEngine::new(),
        }
    }

    pub fn from_layout_str(layout_str: &str) -> Option<Self> {
        LayoutType::from_str(layout_str).map(|layout| Self::new(layout))
    }

    pub fn process_key(&mut self, event: KeyEvent) -> Result<Vec<OutputAction>, EngineError> {
        if !event.is_down {
            return Ok(vec![OutputAction::Nothing]);
        }
        match event.key_code {
            67 => {
                return Ok(self.handle_backspace());
            }
            66 => {
                return Ok(self.handle_enter());
            }
            59 | 60 => {
                self.state.shift_state = match self.state.shift_state {
                    ShiftState::None => ShiftState::Shift,
                    ShiftState::Shift => ShiftState::None,
                    ShiftState::CapsLock => ShiftState::CapsLock,
                };
                return Ok(vec![OutputAction::Nothing]);
            }
            1000 if self.state.layout != LayoutType::English => {
                // যুক্ত (join) key: capture last consonant
                if !self.state.composition_buffer.is_empty() {
                    let last_unicode = self.state.composition_buffer[self.state.composition_buffer.len() - 1].unicode;
                    self.conjunct_engine.start_join(last_unicode);
                } else {
                    self.conjunct_engine.enter_join_mode();
                }
                return Ok(vec![OutputAction::Nothing]);
            }
            1001 if self.state.layout != LayoutType::English => {
                // কার key - handled as popup, just return
                return Ok(vec![OutputAction::Nothing]);
            }
            _ => {}
        }

        // Phonetic layout: use transliteration engine
        if self.state.layout == LayoutType::Phonetic {
            let mut ch = char::from_u32(event.unicode).ok_or(EngineError::InvalidKeyCode(event.unicode))?;
            let shift_active = self.state.shift_state == ShiftState::Shift
                || self.state.shift_state == ShiftState::CapsLock;
            if shift_active && ch.is_ascii_lowercase() {
                // For phonetic, shift maps to aspirated/retroflex variant via uppercase
                // e.g., k + shift -> K -> খ, but we also handle digraph kh
                // We'll convert to uppercase to trigger uppercase mapping if exists
                // For now, keep as is and let phonetic handle shift via separate logic
                // If shift, treat as uppercase for transliteration
                ch = ch.to_ascii_uppercase();
            }
            if self.state.shift_state == ShiftState::Shift {
                self.state.shift_state = ShiftState::None;
            }
            // Join mode: exit on vowel input (aeiou) or non-alphabetic chars
            if self.conjunct_engine.is_join_pending() {
                let is_vowel = matches!(ch.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u');
                if is_vowel || !ch.is_ascii_alphabetic() {
                    self.conjunct_engine.reset();
                }
            }
            // Handle word boundary via phonetic engine
            let is_boundary = WORD_BOUNDARY_CHARS.contains(&ch);
            if is_boundary {
                self.finalize_current_word();
            }
            let (new_text, actions) = self.phonetic.push_char(ch);
            // Update composition buffer and current_word from new_text
            // Clear and rebuild composition buffer from new_text (NFC normalized)
            let normalized: String = new_text.nfc().collect();
            self.state.composition_buffer.clear();
            for c in normalized.chars() {
                self.add_to_buffer(c);
            }
            if !is_boundary && ch != ' ' {
                // Update current_word to last word fragment
                // current_word should track last word after space
                let last_word = normalized.split_whitespace().last().unwrap_or("").to_string();
                self.state.current_word = last_word;
            } else if is_boundary {
                // For space, current_word was finalized, start new
                self.state.current_word.clear();
            }
            // Return actions that represent diff (backspaces + commit)
            // But phonetic diff is in terms of roman buffer, we need to provide actions for UI
            // The actions from phonetic engine already contain Backspace + CommitText
            return Ok(actions);
        }

        // Non-phonetic: fixed layout lookup
        let output_str = {
            let layout = self.layouts.get(&self.state.layout).ok_or_else(|| {
                EngineError::LayoutLoadError(format!(
                    "Layout '{}' not loaded",
                    self.state.layout.as_str()
                ))
            })?;
            let shift_active = self.state.shift_state == ShiftState::Shift
                || self.state.shift_state == ShiftState::CapsLock;
            layout
                .lookup(event.unicode, shift_active)
                .ok_or_else(|| EngineError::InvalidKeyCode(event.unicode))?
                .to_string()
        };
        if self.state.shift_state == ShiftState::Shift {
            self.state.shift_state = ShiftState::None;
        }
        let actions = self.process_character(&output_str)?;
        Ok(actions)
    }

    fn process_character(&mut self, output_str: &str) -> Result<Vec<OutputAction>, EngineError> {
        let mut actions = Vec::new();
        if output_str.len() == 1 {
            if let Some(ch) = output_str.chars().next() {
                if WORD_BOUNDARY_CHARS.contains(&ch) {
                    self.finalize_current_word();
                }
            }
        }
        for ch in output_str.chars() {
            let char_actions = self.process_single_char(ch)?;
            actions.extend(char_actions);
            if !WORD_BOUNDARY_CHARS.contains(&ch) {
                self.state.current_word.push(ch);
            }
        }
        // Normalize actions' CommitText to NFC
        for action in &mut actions {
            if let OutputAction::CommitText(ref mut s) = action {
                let normalized: String = s.nfc().collect();
                *s = normalized;
            }
        }
        Ok(actions)
    }

    fn process_single_char(&mut self, ch: char) -> Result<Vec<OutputAction>, EngineError> {
        let mut actions = Vec::new();
        // Handle Join Mode for conjuncts (যুক্ত key)
        let is_bengali_layout = self.state.layout != LayoutType::English;
        if is_bengali_layout && self.conjunct_engine.is_join_pending() && self.conjuncts.is_consonant(ch) {
            // In join mode, next consonant forms conjunct with previous cluster
            // Remove previous cluster's last grapheme and form new conjunct
            let _prev_len = self.conjunct_engine.get_state().clone();
            // For now, use conjunct_engine to form conjunct
            if let Some(conjunct) = self.conjunct_engine.push_consonant(ch) {
                // Need to backspace the previous consonant(s) that are being joined
                // For simplicity, pop last grapheme(s) and push conjunct
                let text = self.composition_to_string();
                let graphemes: Vec<&str> = text.graphemes(true).collect();
                if !graphemes.is_empty() {
                    // Pop last grapheme (previous consonant) - but for multi-conjunct like ক + যুক্ত + ত + র -> need to handle
                    // For first join, pop one, for subsequent, the conjunct already includes previous, so we need to pop correctly
                    // Simplify: pop one grapheme for first join, for multi, the conjunct_engine's cluster length determines
                    let to_pop = 1; // For first join, pop one previous char
                    for _ in 0..to_pop {
                        // Find grapheme length to pop
                        let current_text = self.composition_to_string();
                        let cur_graphemes: Vec<&str> = current_text.graphemes(true).collect();
                        if let Some(last) = cur_graphemes.last() {
                            let len = last.chars().count() as u32;
                            for _ in 0..len {
                                self.state.composition_buffer.pop();
                            }
                            actions.push(OutputAction::Backspace(len));
                            self.state.current_word = self.state.current_word.graphemes(true).collect::<Vec<&str>>()[..self.state.current_word.graphemes(true).count()-1].concat();
                        }
                    }
                }
                let normalized: String = conjunct.nfc().collect();
                actions.push(OutputAction::CommitText(normalized.clone()));
                for c in normalized.chars() {
                    self.add_to_buffer(c);
                }
                self.state.current_word.push_str(&normalized);
                // Stay in join active for next consonant, unless vowel comes
                return Ok(actions);
            }
        } else if self.conjunct_engine.is_join_pending() && Self::is_vowel_sign(ch) {
            // Vowel exits join mode
            self.conjunct_engine.push_vowel();
        }

        // Smart kar handling: if ch is vowel sign and last char in buffer is also vowel sign, replace
        if Self::is_vowel_sign(ch) {
            if let Some(last) = self.state.composition_buffer.last() {
                if let Some(last_ch) = char::from_u32(last.unicode) {
                    if Self::is_vowel_sign(last_ch) {
                        // Replace previous vowel sign
                        self.state.composition_buffer.pop();
                        // Need to backspace one grapheme (but vowel sign is one codepoint)
                        actions.push(OutputAction::Backspace(1));
                        // Also need to handle current_word replacement?
                        self.state.current_word.pop();
                    }
                }
            }
            // Also exit join mode on vowel
            self.conjunct_engine.push_vowel();
        }

        if ConjunctTable::is_hasanta(ch) {
            if self.state.hasanta_base_consonant.is_some() {
                self.state.composition_buffer.pop();
                actions.push(OutputAction::Backspace(1));
                self.state.hasanta_pending = true;
                return Ok(actions);
            }
            actions.push(OutputAction::CommitText(ch.to_string()));
            self.add_to_buffer(ch);
            return Ok(actions);
        }

        let is_consonant = self.conjuncts.is_consonant(ch);
        if self.state.hasanta_pending && is_consonant {
            if let Some(base) = self.state.hasanta_base_consonant {
                let seq = if let Some(conjunct_str) = self.conjuncts.lookup_hasanta_conjunct(base, ch) {
                    conjunct_str.to_string()
                } else {
                    format!("{}{}{}", base, ConjunctTable::hasanta(), ch)
                };
                let normalized: String = seq.nfc().collect();
                actions.push(OutputAction::CommitText(normalized.clone()));
                for c in normalized.chars() {
                    self.add_to_buffer(c);
                }
                self.state.hasanta_pending = false;
                self.state.hasanta_base_consonant = None;
                return Ok(actions);
            }
        }

        if is_consonant {
            // Before pushing, if previous was consonant and we need to handle hasanta? Already via hasanta logic
            actions.push(OutputAction::CommitText(ch.to_string()));
            self.add_to_buffer(ch);
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = Some(ch);
            return Ok(actions);
        }

        if self.state.hasanta_pending {
            actions.push(OutputAction::CommitText(ConjunctTable::hasanta().to_string()));
            self.add_to_buffer(ConjunctTable::hasanta());
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
        }

        // Normalize vowel sign placement: ensure vowel sign after consonant
        // Already handled by smart kar above
        actions.push(OutputAction::CommitText(ch.to_string()));
        self.add_to_buffer(ch);
        Ok(actions)
    }

    fn is_vowel_sign(ch: char) -> bool {
        matches!(ch, 'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ' | '\u{09BC}' | '\u{09BE}'..='\u{09CC}')
    }

    fn handle_backspace(&mut self) -> Vec<OutputAction> {
        // Phonetic layout: delegate to phonetic engine
        if self.state.layout == LayoutType::Phonetic {
            let actions = self.phonetic.backspace();
            let new_text = self.phonetic.get_output().to_string();
            let normalized: String = new_text.nfc().collect();
            self.state.composition_buffer.clear();
            for c in normalized.chars() {
                self.add_to_buffer(c);
            }
            // Update current_word to last word
            let last_word = normalized.split_whitespace().last().unwrap_or("").to_string();
            self.state.current_word = last_word;
            // Also need to handle hasanta state clear
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
            return actions;
        }

        if self.state.hasanta_pending {
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
            return vec![OutputAction::Nothing];
        }
        if self.state.hasanta_base_consonant.is_some() {
            self.state.hasanta_base_consonant = None;
        }
        if !self.state.composition_buffer.is_empty() {
            // Grapheme-aware deletion
            let text = self.composition_to_string();
            let graphemes: Vec<&str> = text.graphemes(true).collect();
            if graphemes.is_empty() {
                return vec![OutputAction::Nothing];
            }
            let last_grapheme = graphemes.last().unwrap();
            let grapheme_len = last_grapheme.chars().count() as u32;
            // Remove grapheme from composition_buffer
            for _ in 0..grapheme_len {
                self.state.composition_buffer.pop();
            }
            // Remove from current_word as well (grapheme aware)
            let current = self.state.current_word.clone();
            let cur_graphemes: Vec<&str> = current.graphemes(true).collect();
            if !cur_graphemes.is_empty() {
                let new_current: String = cur_graphemes[..cur_graphemes.len()-1].concat();
                self.state.current_word = new_current;
            }
            // Need to handle that deleting a grapheme may affect hasanta state
            return vec![OutputAction::Backspace(grapheme_len)];
        }
        vec![OutputAction::Nothing]
    }

    fn handle_enter(&mut self) -> Vec<OutputAction> {
        let mut actions = Vec::new();
        self.finalize_current_word();
        if self.state.layout == LayoutType::Phonetic {
            // For phonetic, finalize phonetic engine?
            // Just commit buffer
            if !self.state.composition_buffer.is_empty() {
                let text = self.composition_to_string();
                // Clear phonetic buffer? Keep but reset?
                self.phonetic.reset();
                self.state.composition_buffer.clear();
                actions.push(OutputAction::Backspace(1));
                actions.push(OutputAction::CommitText(text));
            }
            if actions.is_empty() {
                actions.push(OutputAction::Nothing);
            }
            return actions;
        }
        if self.state.hasanta_pending {
            actions.push(OutputAction::CommitText(ConjunctTable::hasanta().to_string()));
            self.add_to_buffer(ConjunctTable::hasanta());
            self.state.hasanta_pending = false;
            self.state.hasanta_base_consonant = None;
        }
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

    pub fn get_text(&self) -> String {
        self.composition_to_string()
    }

    /// Direct Bengali character input for kar popup and suggestion completion.
    /// Bypasses layout lookup and phonetic transliteration, handling the
    /// character directly via process_single_char (smart kar, join mode, etc.).
    pub fn process_char(&mut self, ch: char) -> Vec<OutputAction> {
        // Word boundary handling
        if WORD_BOUNDARY_CHARS.contains(&ch) {
            self.finalize_current_word();
        }
        let actions = match self.process_single_char(ch) {
            Ok(a) => a,
            Err(_) => vec![OutputAction::Nothing],
        };
        if !WORD_BOUNDARY_CHARS.contains(&ch) {
            self.state.current_word.push(ch);
        }
        actions
    }

    /// Apply a full-word suggestion, replacing the current partial word.
    /// Engine remains single source of truth; UI must set text = get_text().
    pub fn apply_suggestion(&mut self, suggestion: &str) -> Vec<OutputAction> {
        let normalized: String = suggestion.nfc().collect();
        let current = self.state.current_word.clone();
        // Remove current_word graphemes from buffer tail (if present)
        if !current.is_empty() {
            let cur_graphemes: Vec<&str> = current.graphemes(true).collect();
            let buf_text = self.composition_to_string();
            if buf_text.ends_with(&current) {
                for g in cur_graphemes.iter().rev() {
                    let len = g.chars().count() as usize;
                    for _ in 0..len {
                        self.state.composition_buffer.pop();
                    }
                }
            } else {
                // Fallback: pop by grapheme count
                for _ in 0..cur_graphemes.len() {
                    // pop one grapheme (variable codepoints)
                    let txt = self.composition_to_string();
                    let gs: Vec<&str> = txt.graphemes(true).collect();
                    if let Some(last) = gs.last() {
                        let l = last.chars().count() as usize;
                        for _ in 0..l {
                            self.state.composition_buffer.pop();
                        }
                    }
                }
            }
            self.state.current_word.clear();
        }
        // Append suggestion chars directly (already composed Bengali)
        for ch in normalized.chars() {
            self.add_to_buffer(ch);
            self.state.current_word.push(ch);
        }
        // Finalize word and add trailing space
        self.finalize_current_word();
        self.add_to_buffer(' ');
        vec![OutputAction::CommitText(normalized + " ")]
    }

    pub fn is_join_mode(&self) -> bool {
        self.conjunct_engine.is_join_pending()
    }

    pub fn get_join_suggestions(&mut self) -> Vec<String> {
        self.conjunct_engine.get_suggestions_for_join()
    }

    pub fn current_word(&self) -> String {
        self.state.current_word.clone()
    }

    pub fn get_state(&self) -> &EngineState {
        &self.state
    }

    pub fn get_state_mut(&mut self) -> &mut EngineState {
        &mut self.state
    }

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
        self.phonetic.reset();
        self.conjunct_engine.reset();
    }

    pub fn set_layout(&mut self, layout: LayoutType) {
        if !self.layouts.contains_key(&layout) {
            if let Ok(loaded_layout) = load_layout(layout) {
                self.layouts.insert(layout, loaded_layout);
            }
        }
        self.state.layout = layout;
        self.reset();
    }

    pub fn set_incognito(&mut self, enabled: bool) {
        self.state.incognito_mode = enabled;
    }

    pub fn is_incognito(&self) -> bool {
        self.state.incognito_mode
    }

    pub fn get_candidates(&self) -> &[CandidateWord] {
        &self.state.candidates
    }

    pub fn get_layout(&self, layout_type: LayoutType) -> Option<&Layout> {
        self.layouts.get(&layout_type)
    }

    pub fn get_active_layout(&self) -> Option<&Layout> {
        self.layouts.get(&self.state.layout)
    }

    pub fn get_conjuncts(&self) -> &ConjunctTable {
        &self.conjuncts
    }

    pub fn set_dictionary(&mut self, dictionary: Dictionary) {
        self.dictionary = Some(dictionary);
    }

    pub fn get_suggestions(&self, current_word: &str) -> Vec<CandidateWord> {
        // Normalize input for suggestions
        let normalized: String = current_word.nfc().collect();
        let current_word = normalized.as_str();
        if current_word.is_empty() {
            return Vec::new();
        }
        let mut suggestions = Vec::new();
        if self.user_words.contains(current_word) {
            suggestions.push(CandidateWord {
                word: current_word.to_string(),
                score: 1.0,
                source: WordSource::UserHistory,
            });
        }
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

    fn finalize_current_word(&mut self) {
        if self.state.current_word.is_empty() {
            return;
        }
        let word = self.state.current_word.clone();
        let normalized: String = word.nfc().collect();
        if !self.state.incognito_mode {
            self.state.history.insert(0, normalized.clone());
            if self.state.history.len() > MAX_HISTORY {
                self.state.history.truncate(MAX_HISTORY);
            }
            self.state.last_committed_word = Some(normalized);
        }
        self.state.current_word.clear();
    }

    pub fn finalize_word(&mut self) {
        self.finalize_current_word();
    }

    pub fn add_user_word(&mut self, word: &str) {
        let normalized: String = word.nfc().collect();
        self.user_words.insert(normalized);
    }

    pub fn is_user_word(&self, word: &str) -> bool {
        let normalized: String = word.nfc().collect();
        self.user_words.contains(&normalized)
    }

    pub fn get_next_word_suggestions(&self, limit: usize) -> Vec<CandidateWord> {
        if self.state.incognito_mode {
            return Vec::new();
        }
        let mut context = Vec::new();
        if let Some(ref last_word) = self.state.last_committed_word {
            context.push(last_word.clone());
            if !self.state.history.is_empty() {
                context.insert(0, self.state.history[0].clone());
            }
        }
        if context.is_empty() {
            return Vec::new();
        }
        self.ngram.predict_next(&context, limit)
    }

    fn add_to_buffer(&mut self, ch: char) {
        let glyph = Glyph::simple(ch as u32);
        self.state.composition_buffer.push(glyph);
    }

    fn composition_to_string(&self) -> String {
        let s: String = self.state
            .composition_buffer
            .iter()
            .filter_map(|g| char::from_u32(g.unicode))
            .collect();
        s.nfc().collect()
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

    #[test]
    fn test_phonetic_transliteration() {
        let mut engine = BengaliEngine::new(LayoutType::Phonetic);
        // "ami" -> "আমি"
        for ch in "ami".chars() {
            engine.process_key(KeyEvent::from_char(ch)).unwrap();
        }
        let text = engine.composition_to_string();
        assert_eq!(text, "আমি");
    }

    #[test]
    fn test_grapheme_backspace() {
        let mut engine = BengaliEngine::new(LayoutType::English);
        // Simulate typing ক্ষ (which is 3 codepoints but 1 grapheme)
        // We'll directly push graphemes
        engine.process_key(KeyEvent::down(29, 'q' as u32)).unwrap(); // not relevant
        // Instead test via direct buffer
        engine.state.composition_buffer.clear();
        for ch in "ক্ষ".chars() {
            engine.add_to_buffer(ch);
        }
        engine.state.current_word = "ক্ষ".to_string();
        let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
        // Should delete whole grapheme "ক্ষ" -> 3 codepoints
        assert_eq!(actions[0], OutputAction::Backspace(3));
        assert!(engine.state.composition_buffer.is_empty());
    }
}

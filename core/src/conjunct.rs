/// Bengali conjunct character handling.
///
/// This module provides:
/// - Consonant classification for hasanta-based conjunct formation
/// - Consonant set for determining which characters can form conjuncts
///
/// Bengali conjunct formation rules:
/// 1. Hasanta (্): When a consonant is followed by hasanta (U+09CD) and
///    then another consonant, they form a conjunct sequence that the
///    rendering engine displays as a combined glyph.
///    (e.g., ক + ্ + ষ → ক্ষ)
///
/// In the phonetic layout:
/// - Each consonant key maps directly to its Bengali character (k→ক, g→গ)
/// - Shift variants provide aspirated consonants (shift+k→খ, shift+g→ঘ)
/// - The backslash key produces hasanta (্) for manual conjunct formation
/// - Special shortcuts: x→ক্ষ, z→জ্ঞ

use std::collections::HashMap;

/// Hasanta (virama) character - suppresses inherent vowel.
pub const HASANTA: char = '\u{09CD}';

/// The main conjunct table for Bengali phonetic typing.
pub struct ConjunctTable {
    /// Set of Bengali consonants (Unicode range U+0995..U+09B9 plus others).
    /// These are characters that can participate in conjunct formation
    /// with hasanta (virama).
    consonants: HashMap<char, bool>,

    /// Maps (consonant1, consonant2) → combined output for hasanta conjuncts.
    /// Example: ('ক', 'ষ') → "ক্ষ" (as a sequence of 3 codepoints)
    /// Note: Most conjuncts are just consonant + hasanta + consonant,
    /// but some have special rendered forms.
    hasanta_conjuncts: HashMap<(char, char), String>,
}

impl ConjunctTable {
    /// Create a new ConjunctTable with all standard Bengali consonant
    /// definitions and hasanta conjunct mappings.
    pub fn new() -> Self {
        // Bengali consonants (U+0995 to U+09B9, plus others)
        let mut consonants = HashMap::new();
        // Main consonant range: ক খ গ ঘ ঙ চ ছ জ ঝ ঞ ট ঠ ড ঢ ণ
        //                       ত থ দ ধ ন প ফ ব ভ ম য র ল শ ষ স হ
        for ch in '\u{0995}'..='\u{09B9}' {
            consonants.insert(ch, true);
        }
        // Also include ড় (U+09DC), ঢ় (U+09DD), য় (U+09DF)
        consonants.insert('\u{09DC}', true); // ড়
        consonants.insert('\u{09DD}', true); // ঢ়
        consonants.insert('\u{09DF}', true); // য়
        // And the special conjunct consonants that have their own codepoints
        consonants.insert('\u{099E}', true); // ঞ
        consonants.insert('\u{0999}', true); // ঙ

        // Hasanta conjuncts (consonant1 + hasanta + consonant2 → special form)
        // Most conjuncts are just the sequence rendered by the font,
        // but some have dedicated Unicode codepoints or special rendering
        let mut hasanta_conjuncts = HashMap::new();
        // ক্ষ (ক+্+ষ) — one of the most common conjuncts
        hasanta_conjuncts.insert(
            ('\u{0995}', '\u{09B7}'),
            "\u{0995}\u{09CD}\u{09B7}".to_string(),
        );
        // জ্ঞ (জ+্+ঞ) — common conjunct
        hasanta_conjuncts.insert(
            ('\u{099C}', '\u{099E}'),
            "\u{099C}\u{09CD}\u{099E}".to_string(),
        );
        // ত্ত (ট+্+ট)
        hasanta_conjuncts.insert(
            ('\u{099F}', '\u{099F}'),
            "\u{099F}\u{09CD}\u{099F}".to_string(),
        );
        // ষ্ণ (ষ+্+ণ)
        hasanta_conjuncts.insert(
            ('\u{09B7}', '\u{09A3}'),
            "\u{09B7}\u{09CD}\u{09A3}".to_string(),
        );
        // স্ত (স+্+ত)
        hasanta_conjuncts.insert(
            ('\u{09B8}', '\u{099F}'),
            "\u{09B8}\u{09CD}\u{099F}".to_string(),
        );
        // স্থ (স+্+থ)
        hasanta_conjuncts.insert(
            ('\u{09B8}', '\u{09A5}'),
            "\u{09B8}\u{09CD}\u{09A5}".to_string(),
        );
        // ন্ত (ন+্+ত)
        hasanta_conjuncts.insert(
            ('\u{09A8}', '\u{099F}'),
            "\u{09A8}\u{09CD}\u{099F}".to_string(),
        );
        // ন্দ (ন+্+দ)
        hasanta_conjuncts.insert(
            ('\u{09A8}', '\u{09A6}'),
            "\u{09A8}\u{09CD}\u{09A6}".to_string(),
        );
        // ষ্ট (ষ+্+ট)
        hasanta_conjuncts.insert(
            ('\u{09B7}', '\u{099F}'),
            "\u{09B7}\u{09CD}\u{099F}".to_string(),
        );

        Self {
            consonants,
            hasanta_conjuncts,
        }
    }

    /// Check if a character is a Bengali consonant.
    ///
    /// Consonants are characters that can participate in conjunct formation
    /// with hasanta (virama).
    pub fn is_consonant(&self, ch: char) -> bool {
        self.consonants.contains_key(&ch)
    }

    /// Look up a hasanta conjunct (consonant + hasanta + consonant).
    ///
    /// Returns the string representation of the conjunct sequence,
    /// or None if no special conjunct exists (use default rendering).
    pub fn lookup_hasanta_conjunct(&self, first: char, second: char) -> Option<&str> {
        self.hasanta_conjuncts
            .get(&(first, second))
            .map(|s| s.as_str())
    }

    /// Get the hasanta character.
    pub fn hasanta() -> char {
        HASANTA
    }

    /// Check if a character is the hasanta (virama).
    pub fn is_hasanta(ch: char) -> bool {
        ch == HASANTA
    }

    /// Get the number of consonants in the table.
    pub fn consonant_count(&self) -> usize {
        self.consonants.len()
    }
}

impl Default for ConjunctTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conjunct_table_creation() {
        let table = ConjunctTable::new();
        assert!(table.consonant_count() > 0);
    }

    #[test]
    fn test_is_consonant() {
        let table = ConjunctTable::new();

        // Bengali consonants
        assert!(table.is_consonant('ক')); // U+0995
        assert!(table.is_consonant('খ')); // U+0996
        assert!(table.is_consonant('গ')); // U+0997
        assert!(table.is_consonant('হ')); // U+09B9

        // Non-consonants
        assert!(!table.is_consonant('া')); // Vowel sign
        assert!(!table.is_consonant('ি')); // Vowel sign
        assert!(!table.is_consonant('ে')); // Vowel sign
        assert!(!table.is_consonant('a')); // English letter
        assert!(!table.is_consonant('0')); // Digit
    }

    #[test]
    fn test_is_hasanta() {
        assert!(ConjunctTable::is_hasanta('\u{09CD}')); // Hasanta
        assert!(!ConjunctTable::is_hasanta('ক'));
        assert!(!ConjunctTable::is_hasanta('া'));
    }

    #[test]
    fn test_hasanta_constant() {
        assert_eq!(ConjunctTable::hasanta(), '\u{09CD}');
    }

    #[test]
    fn test_hasanta_conjunct_kaksha() {
        let table = ConjunctTable::new();

        // ক্ষ (ক+্+ষ)
        let conjunct = table.lookup_hasanta_conjunct('ক', 'ষ');
        assert!(conjunct.is_some());
        let c = conjunct.unwrap();
        assert!(c.contains('\u{0995}')); // ক
        assert!(c.contains('\u{09CD}')); // ্
        assert!(c.contains('\u{09B7}')); // ষ
    }

    #[test]
    fn test_hasanta_conjunct_jnya() {
        let table = ConjunctTable::new();

        // জ্ঞ (জ+্+ঞ)
        let conjunct = table.lookup_hasanta_conjunct('জ', 'ঞ');
        assert!(conjunct.is_some());
        let c = conjunct.unwrap();
        assert!(c.contains('\u{099C}')); // জ
        assert!(c.contains('\u{09CD}')); // ্
        assert!(c.contains('\u{099E}')); // ঞ
    }

    #[test]
    fn test_hasanta_conjunct_no_match() {
        let table = ConjunctTable::new();

        // ক+্+গ has no special conjunct
        let conjunct = table.lookup_hasanta_conjunct('ক', 'গ');
        assert!(conjunct.is_none());
    }
}

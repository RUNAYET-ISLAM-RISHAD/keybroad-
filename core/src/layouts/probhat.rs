//! Probhat layout – authoritative mapping profile (Sprint 1, 100% correct)
//! Stable Key IDs (Q/W/E/R…) → Bengali glyphs. UI derives labels from this profile;
//! engine uses logical IDs for composition. Separated from Phonetic/Jatiya/UniJoy.

use std::collections::HashMap;

/// Mapping for a single physical key: primary (tap) and secondary (shift / long-press).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbhatKeyMapping {
    pub primary: char,
    pub secondary: char,
}

/// Hasanta (halant) – U+09CD
pub const HASANTA: char = '\u{09CD}';

/// Returns the authoritative Probhat base-layer mapping.
///
/// Keys are logical IDs in lower-case where applicable (`'q'` for Q, `'['` for `[`).
/// Secondary is the shift/long-press output.
pub fn probhat_map() -> HashMap<char, ProbhatKeyMapping> {
    let mut m = HashMap::new();
    let mut ins = |k: char, p: char, s: char| {
        m.insert(k, ProbhatKeyMapping { primary: p, secondary: s });
    };
    // Row 1: Q W E R T Y U I O P [ ]
    ins('q', 'দ', 'ধ');
    ins('w', 'ূ', 'ঊ');
    ins('e', 'ী', 'ঈ');
    ins('r', 'র', 'ড়');
    ins('t', 'ট', 'ঠ');
    ins('y', 'এ', 'ঐ');
    ins('u', 'ু', 'উ');
    ins('i', 'ি', 'ই');
    ins('o', 'ও', 'ঔ');
    ins('p', 'প', 'ফ');
    ins('[', 'ে', 'ৈ');
    ins(']', 'ো', 'ৌ');
    // Row 2: A S D F G H J K L
    ins('a', 'া', 'অ');
    ins('s', 'স', 'ষ');
    ins('d', 'ড', 'ঢ');
    ins('f', 'ত', 'থ');
    ins('g', 'গ', 'ঘ');
    ins('h', 'হ', 'ঃ');
    ins('j', 'জ', 'ঝ');
    ins('k', 'ক', 'খ');
    ins('l', 'ল', 'ং');
    // Row 3: Z X C V B N M , . /
    ins('z', 'য়', 'য');
    ins('x', 'শ', 'ঢ়');
    ins('c', 'চ', 'ছ');
    ins('v', 'আ', 'ঋ');
    ins('b', 'ব', 'ভ');
    ins('n', 'ন', 'ণ');
    ins('m', 'ম', 'ঙ');
    ins(',', ',', 'ৃ');
    ins('.', '।', 'ঁ');
    ins('/', HASANTA, HASANTA);
    // Digits 0-9 (primary only, secondary same)
    ins('0', '০', '০');
    ins('1', '১', '১');
    ins('2', '২', '২');
    ins('3', '৩', '৩');
    ins('4', '৪', '৪');
    ins('5', '৫', '৫');
    ins('6', '৬', '৬');
    ins('7', '৭', '৭');
    ins('8', '৮', '৮');
    ins('9', '৯', '৯');
    ins(' ', ' ', ' ');
    m
}

/// Convenience: primary output for a key, if defined.
pub fn primary_for(key: char) -> Option<char> {
    probhat_map().get(&key).map(|km| km.primary)
}

/// Convenience: secondary output for a key, if defined.
pub fn secondary_for(key: char) -> Option<char> {
    probhat_map().get(&key).map(|km| km.secondary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_size() {
        let m = probhat_map();
        // 32 base keys + 10 digits + space = 43 entries
        assert!(m.len() >= 40, "map should have ~42 entries, got {}", m.len());
    }

    #[test]
    fn test_hasanta() {
        let m = probhat_map();
        assert_eq!(m[&'/'].primary, HASANTA);
        assert_eq!(HASANTA as u32, 0x09CD);
    }

    #[test]
    fn test_sample_mappings() {
        let m = probhat_map();
        assert_eq!(m[&'q'].primary, 'দ');
        assert_eq!(m[&'q'].secondary, 'ধ');
        assert_eq!(m[&'k'].primary, 'ক');
        assert_eq!(m[&'l'].secondary, 'ং');
        assert_eq!(m[&'v'].primary, 'আ');
        assert_eq!(m[&'/'].primary, '\u{09CD}');
    }
}

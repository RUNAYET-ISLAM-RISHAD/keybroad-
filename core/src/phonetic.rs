/// Phonetic transliteration engine (Avro-style Roman → Bengali).
///
/// Implements rule-based transliteration where Roman QWERTY input is
/// converted to Bengali Unicode. Handles digraphs, vowel signs,
/// conjuncts, and contextual rules.

/// Check if a Bengali char is a consonant (for kar logic).
fn is_bengali_consonant(ch: char) -> bool {
    matches!(ch,
        'ক' | 'খ' | 'গ' | 'ঘ' | 'ঙ' |
        'চ' | 'ছ' | 'জ' | 'ঝ' | 'ঞ' |
        'ট' | 'ঠ' | 'ড' | 'ঢ' | 'ণ' |
        'ত' | 'থ' | 'দ' | 'ধ' | 'ন' |
        'প' | 'ফ' | 'ব' | 'ভ' | 'ম' |
        'য' | 'র' | 'ল' |
        'শ' | 'ষ' | 'স' | 'হ' |
        '\u{09DC}' | '\u{09DD}' | '\u{09DF}' |
        'ৎ'
    )
}

fn is_bengali_vowel_sign(ch: char) -> bool {
    matches!(ch,
        'া' | 'ি' | 'ী' | 'ু' | 'ূ' | 'ৃ' | 'ে' | 'ৈ' | 'ো' | 'ৌ' | '\u{09BE}'..='\u{09CC}'
    )
}

const HASANTA: char = '\u{09CD}';

/// Main transliteration function: Roman string → Bengali string.
///
/// Processes the entire input buffer each time to handle incremental
/// changes (e.g., 'k' → 'ক', 'kh' → 'খ' replaces previous).
pub fn transliterate(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let s = input;
    let mut out = String::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    while i < chars.len() {
        // Try longest match first (3 chars, then 2, then 1)
        let mut matched = false;

        // Check for 3-char patterns like "ksh", "ngg" etc (longest)
        if i + 2 < chars.len() {
            let tri: String = chars[i..i+3].iter().collect();
            if let Some(beng) = match_trigraph(&tri) {
                // For trigraph, need to handle vowel context?
                // "ksh" is consonant conjunct, output directly
                out.push_str(beng);
                i += 3;
                matched = true;
                continue;
            }
        }

        if i + 1 < chars.len() {
            let di: String = chars[i..i+2].iter().collect();
            // Check digraphs first
            if let Some(beng) = match_digraph(&di) {
                // For vowel digraphs, need kar vs independent logic
                if is_vowel_digraph(&di) {
                    let kar = vowel_kar(&di);
                    let indep = vowel_independent(&di);
                    if needs_kar(&out) {
                        out.push_str(kar);
                    } else {
                        out.push_str(indep);
                    }
                } else {
                    // Consonant digraph like kh, gh, etc.
                    // Check if previous was consonant without vowel and this is 'r' handling?
                    // For now, handle 'r' special after consonant
                    if di == "kr" || di == "gr" || di == "pr" || di == "br" || di == "tr" || di == "sr" || di == "hr" {
                        // This case shouldn't happen as "kr" is not in digraph map, but handle ra-phala
                        // Instead, we handle 'r' as single char after consonant
                        // Fall through to single handling
                    } else {
                        // Handle r after consonant: if digraph contains 'r' as second char and previous was consonant
                        // For now just push
                        // Special for "ng": map to ং, but need to decide if after vowel?
                        out.push_str(beng);
                    }
                }
                i += 2;
                matched = true;
                continue;
            }
        }

        // Single char handling
        if !matched {
            let ch = chars[i];
            let single = ch.to_string();
            // Check if single is vowel
            if is_vowel_single(&single) {
                let kar = vowel_kar(&single);
                let indep = vowel_independent(&single);
                if needs_kar(&out) {
                    // Smart kar: if last char is already a vowel sign, replace it
                    if ends_with_vowel_sign(&out) {
                        // Replace previous vowel sign
                        out.pop();
                        // Handle that ো is single char, but we popped correctly
                        // For ৌ etc, also single
                    }
                    out.push_str(kar);
                } else {
                    out.push_str(indep);
                }
            } else if single == "r" {
                // Special handling for 'r': ra-phala after consonant, reph before consonant, or independent র
                if needs_kar(&out) || ends_with_consonant(&out) {
                    // After consonant: ra-phala (্র)
                    // Check if next char is vowel? For "ri" case, we need to handle "r" + "i" as "ৃ"?
                    // But here we handle single 'r' as ্র
                    // If out ends with consonant, add hasanta + র
                    // Need to avoid double hasanta if already has hasanta
                    if out.ends_with(HASANTA) {
                        out.push('র');
                    } else {
                        out.push(HASANTA);
                        out.push('র');
                    }
                } else {
                    // Independent র
                    out.push('র');
                }
            } else if let Some(beng) = match_single(&single) {
                // Consonant
                // Before pushing consonant, if previous char was consonant without vowel, we may need hasanta?
                // In Avro, consonant cluster without vowel automatically gets hasanta? For "k" + "r" -> "ক্র", we already handled r as ra-phala
                // For "s" + "h" -> "শ" is digraph, not two consonants
                // For "n" + "g" -> "ং" is digraph, handled above
                // For other consonant clusters, we should insert hasanta between? Like "k" + "k" -> "ক্ক"? That's ক + ্ + ক
                // Should we insert hasanta automatically when two consonants in a row without vowel?
                // Avro does: consecutive consonants without vowel get hasanta. e.g., "kk" -> "ক্ক"
                // Let's handle: if out ends with consonant and new char is consonant, insert hasanta
                if ends_with_consonant(&out) && is_bengali_consonant_str(beng) {
                    // Check if we are in middle of word and no vowel intervened
                    // Insert hasanta before new consonant
                    out.push(HASANTA);
                }
                out.push_str(beng);
            } else {
                // Digit or punctuation or unknown: keep as is or map digits
                if let Some(mapped) = map_digit_or_punct(&single) {
                    out.push_str(mapped);
                } else {
                    out.push(ch);
                }
            }
            i += 1;
        }
    }

    // Special handling for "ksh" already handled as trigraph, but also handle "kkh" etc via digraph

    // Post-process: handle "ng" context, normalize, ensure vowel signs correct
    // For "bangla" -> we mapped "ng" -> "ং", so "ban" + "g" + "la" -> need to ensure "a" after "ং" is handled?
    // "ং" is not consonant, so vowel after it should be independent? Actually "bangla" -> ব + া + ং + ল + া, after "ং", next "l" is consonant, then "a" is kar.
    // Our needs_kar checks if ends with consonant, so after "ং" (which is not consonant), 'l' will be independent consonant, correct.

    // Handle special conjunct "kk" etc already via hasanta insertion.

    // Normalize Unicode (NFC) for vowel signs like ো = ে + া? But we directly output ো, so okay.
    // For now, return as is, but apply NFC normalization

    // Use unicode-normalization if available, else return
    out
}

fn ends_with_consonant(s: &str) -> bool {
    s.chars().last().map_or(false, |c| is_bengali_consonant(c))
}

fn ends_with_vowel_sign(s: &str) -> bool {
    s.chars().last().map_or(false, |c| is_bengali_vowel_sign(c))
}

fn needs_kar(s: &str) -> bool {
    // Need kar if string ends with a consonant (or hasanta+consonant cluster)
    // Check last non-hasanta char is consonant
    if s.is_empty() {
        return false;
    }
    // Find last Bengali char that is not hasanta
    for ch in s.chars().rev() {
        if ch == HASANTA {
            continue;
        }
        return is_bengali_consonant(ch);
    }
    false
}

fn is_bengali_consonant_str(s: &str) -> bool {
    s.chars().next().map_or(false, |c| is_bengali_consonant(c))
}

fn is_vowel_single(s: &str) -> bool {
    matches!(s, "a" | "i" | "u" | "e" | "o")
}

fn is_vowel_digraph(s: &str) -> bool {
    matches!(s, "aa" | "ii" | "uu" | "ei" | "oi" | "ou" | "au" | "ee")
}

fn vowel_kar(s: &str) -> &'static str {
    match s {
        "a" | "aa" => "া",
        "i" | "I" => "ি",
        "ii" => "ী",
        "u" => "ু",
        "uu" => "ূ",
        "ri" => "ৃ",
        "e" => "ে",
        "oi" => "ৈ",
        "o" => "ো",
        "ou" => "ৌ",
        "ae" => "ৈ",
        _ => "",
    }
}

fn vowel_independent(s: &str) -> &'static str {
    match s {
        "a" | "aa" => "আ",
        "i" => "ই",
        "ii" | "I" => "ঈ",
        "u" => "উ",
        "uu" | "U" => "ঊ",
        "ri" | "rri" => "ঋ",
        "e" => "এ",
        "oi" => "ঐ",
        "o" => "ও",
        "ou" => "ঔ",
        _ => "",
    }
}

fn match_trigraph(s: &str) -> Option<&'static str> {
    match s {
        "ksh" => Some("ক্ষ"), // ক + ্ + ষ
        "kkh" => Some("ক্ষ"), // variant?
        "ngg" => Some("ঙ্গ"), // ঙ + ্ + গ
        "chh" => Some("ছ"),
        _ => None,
    }
}

fn match_digraph(s: &str) -> Option<&'static str> {
    match s {
        "kh" => Some("খ"),
        "gh" => Some("ঘ"),
        "ch" => Some("চ"),
        "jh" => Some("ঝ"),
        "th" => Some("থ"),
        "dh" => Some("ধ"),
        "ph" => Some("ফ"),
        "bh" => Some("ভ"),
        "sh" => Some("শ"),
        "ng" => Some("ং"), // anusvara
        "Ng" => Some("ঙ"),
        "aa" => Some("আ"), // but handled as vowel
        "ii" => Some("ঈ"),
        "uu" => Some("ঊ"),
        "ee" => Some("ঈ"), // variant
        "oi" => Some("ঐ"),
        "ou" => Some("ঔ"),
        "Th" => Some("ঠ"),
        "Dh" => Some("ঢ"),
        "Sh" => Some("ষ"),
        "Rh" => Some("ঢ়"),
        _ => None,
    }
}

fn match_single(s: &str) -> Option<&'static str> {
    match s {
        "k" => Some("ক"),
        "K" => Some("খ"),
        "g" => Some("গ"),
        "G" => Some("ঘ"),
        "c" => Some("চ"),
        "C" => Some("ছ"),
        "j" => Some("জ"),
        "J" => Some("ঝ"),
        "t" => Some("ত"),
        "T" => Some("ট"),
        "d" => Some("দ"),
        "D" => Some("ড"),
        "n" => Some("ন"),
        "N" => Some("ণ"),
        "p" => Some("প"),
        "P" => Some("ফ"),
        "b" => Some("ব"),
        "B" => Some("ভ"),
        "m" => Some("ম"),
        "y" => Some("য"),
        "Y" => Some("য়"),
        "r" => Some("র"), // handled specially
        "R" => Some("ড়"),
        "l" => Some("ল"),
        "s" => Some("স"),
        "S" => Some("ষ"),
        "h" => Some("হ"),
        "f" => Some("ফ"), // f as ph
        "F" => Some("ফ"),
        "z" => Some("য"), // not used
        "q" => Some("ক"), // fallback
        "w" => Some("ও"), //?
        "x" => Some("ক্স"), //?
        "v" => Some("ভ"), //?
        _ => None,
    }
}

fn map_digit_or_punct(s: &str) -> Option<&'static str> {
    match s {
        "0" => Some("০"),
        "1" => Some("১"),
        "2" => Some("২"),
        "3" => Some("৩"),
        "4" => Some("৪"),
        "5" => Some("৫"),
        "6" => Some("৬"),
        "7" => Some("৭"),
        "8" => Some("৮"),
        "9" => Some("৯"),
        " " => Some(" "),
        "." => Some("।"), // Bengali danda? But keep .
        "," => Some(","),
        "?" => Some("?"),
        "!" => Some("!"),
        _ => None,
    }
}

// Incremental transliteration helper for engine: maintains roman buffer and bengali output

pub struct PhoneticEngine {
    roman_buffer: String,
    bengali_output: String,
}

impl PhoneticEngine {
    pub fn new() -> Self {
        Self {
            roman_buffer: String::new(),
            bengali_output: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.roman_buffer.clear();
        self.bengali_output.clear();
    }

    pub fn push_char(&mut self, ch: char) -> (String, Vec<crate::types::OutputAction>) {
        // For space and punctuation, finalize word
        if ch == ' ' || ch == '\n' || ch == '.' || ch == ',' {
            self.roman_buffer.push(ch);
            let new_bengali = transliterate(&self.roman_buffer);
            let diff = Self::diff_bengali(&self.bengali_output, &new_bengali);
            self.bengali_output = new_bengali;
            // For word boundary, we should also clear roman buffer? But keep for history
            // Actually for next word, roman_buffer should continue, but transliteration handles spaces
            // Keep buffer
            return (self.bengali_output.clone(), diff);
        }

        self.roman_buffer.push(ch);
        let new_bengali = transliterate(&self.roman_buffer);
        let diff = Self::diff_bengali(&self.bengali_output, &new_bengali);
        self.bengali_output = new_bengali.clone();
        (new_bengali, diff)
    }

    pub fn backspace(&mut self) -> Vec<crate::types::OutputAction> {
        if self.roman_buffer.is_empty() {
            return vec![crate::types::OutputAction::Nothing];
        }
        // Remove last roman char (grapheme aware for roman is just one)
        self.roman_buffer.pop();
        let new_bengali = transliterate(&self.roman_buffer);
        let diff = Self::diff_bengali(&self.bengali_output, &new_bengali);
        self.bengali_output = new_bengali;
        diff
    }

    pub fn get_output(&self) -> &str {
        &self.bengali_output
    }

    pub fn get_roman(&self) -> &str {
        &self.roman_buffer
    }

    fn diff_bengali(old: &str, new: &str) -> Vec<crate::types::OutputAction> {
        // Compute longest common prefix
        let old_chars: Vec<char> = old.chars().collect();
        let new_chars: Vec<char> = new.chars().collect();

        let mut common = 0;
        while common < old_chars.len() && common < new_chars.len() && old_chars[common] == new_chars[common] {
            common += 1;
        }

        let mut actions = Vec::new();
        let to_delete = old_chars.len() - common;
        if to_delete > 0 {
            // Use grapheme-aware deletion: count graphemes to delete
            // For simplicity, count chars, but we should count graphemes
            // Old string's suffix from common to end is to be deleted
            // For transliteration, deleting one roman may delete multiple bengali chars (e.g., kh -> খ is 1, but k -> ক is 1, so 'h' changes ক to খ = 1 delete + 1 insert)
            actions.push(crate::types::OutputAction::Backspace(to_delete as u32));
        }
        if common < new_chars.len() {
            let new_suffix: String = new_chars[common..].iter().collect();
            actions.push(crate::types::OutputAction::CommitText(new_suffix));
        }
        if actions.is_empty() {
            actions.push(crate::types::OutputAction::Nothing);
        }
        actions
    }
}

impl Default for PhoneticEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transliterate_ami() {
        assert_eq!(transliterate("ami"), "আমি");
    }

    #[test]
    fn test_transliterate_bangla() {
        // bangla -> বাংলা (ব + া + ং + ল + া)
        let out = transliterate("bangla");
        assert_eq!(out, "বাংলা");
    }

    #[test]
    fn test_transliterate_khela() {
        assert_eq!(transliterate("khela"), "খেলা");
    }

    #[test]
    fn test_transliterate_kh() {
        assert_eq!(transliterate("kh"), "খ");
    }

    #[test]
    fn test_transliterate_gh() {
        assert_eq!(transliterate("gh"), "ঘ");
    }

    #[test]
    fn test_transliterate_chh() {
        // chh -> ছ
        assert_eq!(transliterate("chh"), "ছ");
        assert_eq!(transliterate("ch"), "চ");
    }

    #[test]
    fn test_transliterate_sh() {
        assert_eq!(transliterate("sh"), "শ");
    }

    #[test]
    fn test_transliterate_ng() {
        assert_eq!(transliterate("ng"), "ং");
    }

    #[test]
    fn test_transliterate_aa() {
        assert_eq!(transliterate("aa"), "আ");
        assert_eq!(transliterate("a"), "আ");
    }

    #[test]
    fn test_transliterate_ii() {
        assert_eq!(transliterate("ii"), "ঈ");
    }

    #[test]
    fn test_transliterate_uu() {
        assert_eq!(transliterate("uu"), "ঊ");
    }

    #[test]
    fn test_transliterate_oi() {
        assert_eq!(transliterate("oi"), "ঐ");
    }

    #[test]
    fn test_transliterate_ou() {
        assert_eq!(transliterate("ou"), "ঔ");
    }

    #[test]
    fn test_transliterate_kri() {
        // k + r + i -> ক্রি
        assert_eq!(transliterate("kri"), "ক্রি");
    }

    #[test]
    fn test_transliterate_shri() {
        // sh + r + i -> শ্রি
        assert_eq!(transliterate("shri"), "শ্রি");
    }

    #[test]
    fn test_transliterate_ksh() {
        assert_eq!(transliterate("ksh"), "ক্ষ");
    }

    #[test]
    fn test_phonetic_engine_incremental() {
        let mut eng = PhoneticEngine::new();
        let (out, _) = eng.push_char('a');
        assert_eq!(out, "আ");
        let (out, _) = eng.push_char('m');
        assert_eq!(out, "আম");
        let (out, _) = eng.push_char('i');
        assert_eq!(out, "আমি");
    }

    #[test]
    fn test_phonetic_backspace() {
        let mut eng = PhoneticEngine::new();
        eng.push_char('a');
        eng.push_char('m');
        eng.push_char('i');
        assert_eq!(eng.get_output(), "আমি");
        eng.backspace();
        // after deleting 'i', should be "আম"
        assert_eq!(eng.get_output(), "আম");
    }

    #[test]
    fn test_kh_incremental() {
        let mut eng = PhoneticEngine::new();
        let (out, _) = eng.push_char('k');
        assert_eq!(out, "ক");
        let (out, _) = eng.push_char('h');
        assert_eq!(out, "খ");
    }
}

use keybroad_core::{BengaliEngine, KeyEvent, LayoutType};
use keybroad_core::layouts::probhat::{probhat_map, HASANTA};
use unicode_normalization::UnicodeNormalization;

fn probhat_engine() -> BengaliEngine {
    BengaliEngine::new(LayoutType::Probhat)
}

fn type_key(engine: &mut BengaliEngine, key: char) -> String {
    let _ = engine.process_key(KeyEvent::down(key as u32, key as u32)).unwrap();
    engine.get_text()
}

fn type_key_shift(engine: &mut BengaliEngine, key: char) -> String {
    // Toggle shift, type key, shift auto-resets
    let _ = engine.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    let _ = engine.process_key(KeyEvent::down(key as u32, key as u32)).unwrap();
    engine.get_text()
}

// 1. Key Mapping Tests — every primary output
#[test]
fn test_probhat_all_primary_mappings() {
    let map = probhat_map();
    for (key, mapping) in map.iter() {
        if key.is_ascii_digit() || *key == ' ' {
            continue;
        }
        let mut engine = probhat_engine();
        let _ = engine.process_key(KeyEvent::down(*key as u32, *key as u32)).unwrap();
        let text: String = engine.get_text().nfc().collect();
        let expected: String = mapping.primary.to_string().nfc().collect();
        assert!(
            text.contains(&expected) || text.nfc().collect::<String>().contains(&expected),
            "key '{}' should produce primary '{}' (NFC '{}'), got text '{}' (NFC '{}')",
            key,
            mapping.primary,
            expected,
            engine.get_text(),
            text
        );
    }
}

#[test]
fn test_probhat_primary_spot_checks() {
    let cases = [
        ('q', 'দ'),
        ('w', 'ূ'),
        ('e', 'ী'),
        ('r', 'র'),
        ('t', 'ট'),
        ('y', 'এ'),
        ('u', 'ু'),
        ('i', 'ি'),
        ('o', 'ও'),
        ('p', 'প'),
        ('[', 'ে'),
        (']', 'ো'),
        ('a', 'া'),
        ('s', 'স'),
        ('d', 'ড'),
        ('f', 'ত'),
        ('g', 'গ'),
        ('h', 'হ'),
        ('j', 'জ'),
        ('k', 'ক'),
        ('l', 'ল'),
        ('z', 'য়'),
        ('x', 'শ'),
        ('c', 'চ'),
        ('v', 'আ'),
        ('b', 'ব'),
        ('n', 'ন'),
        ('m', 'ম'),
        (',', ','),
        ('.', '।'),
        ('/', HASANTA),
    ];
    for (key, expected) in cases {
        let mut engine = probhat_engine();
        let _ = engine.process_key(KeyEvent::down(key as u32, key as u32)).unwrap();
        let text: String = engine.get_text().nfc().collect();
        let expected_nfc: String = expected.to_string().nfc().collect();
        assert_eq!(
            text, expected_nfc,
            "key '{}' primary mismatch: got '{}' vs expected '{}'",
            key, text, expected_nfc
        );
    }
}

// 2. Shift / secondary outputs via shift toggle
#[test]
fn test_probhat_shift_secondary() {
    let cases = [
        ('q', 'ধ'),
        ('w', 'ঊ'),
        ('e', 'ঈ'),
        ('r', 'ড়'),
        ('t', 'ঠ'),
        ('y', 'ঐ'),
        ('u', 'উ'),
        ('i', 'ই'),
        ('o', 'ঔ'),
        ('p', 'ফ'),
        ('[', 'ৈ'),
        (']', 'ৌ'),
        ('a', 'অ'),
        ('s', 'ষ'),
        ('d', 'ঢ'),
        ('f', 'থ'),
        ('g', 'ঘ'),
        ('h', 'ঃ'),
        ('j', 'ঝ'),
        ('k', 'খ'),
        ('l', 'ং'),
        ('z', 'য'),
        ('x', 'ঢ়'),
        ('c', 'ছ'),
        ('v', 'ঋ'),
        ('b', 'ভ'),
        ('n', 'ণ'),
        ('m', 'ঙ'),
        (',', 'ৃ'),
        ('.', 'ঁ'),
    ];
    for (key, expected) in cases {
        let mut engine = probhat_engine();
        let text: String = type_key_shift(&mut engine, key).nfc().collect();
        let expected_nfc: String = expected.to_string().nfc().collect();
        assert_eq!(
            text, expected_nfc,
            "shift+key '{}' should be '{}' (NFC), got '{}'",
            key, expected_nfc, text
        );
    }
}

// 3. Vowel sign composition
#[test]
fn test_probhat_vowel_sign_ka_aa() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    assert_eq!(engine.get_text(), "ক");
    type_key(&mut engine, 'a'); // া
    assert_eq!(engine.get_text(), "কা");
}

#[test]
fn test_probhat_vowel_sign_ka_i() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    type_key(&mut engine, 'i'); // ি
    assert_eq!(engine.get_text(), "কি");
}

#[test]
fn test_probhat_vowel_sign_ka_e() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    type_key(&mut engine, '['); // ে
    assert_eq!(engine.get_text(), "কে");
}

#[test]
fn test_probhat_vowel_sign_ka_o() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    type_key(&mut engine, ']'); // ো
    // Depending on NFC, ো may be composed; check contains
    let text = engine.get_text();
    assert!(text.contains('ক'), "ka+o should contain ক, got {}", text);
    // ো is U+09CB, may normalize; just ensure not plain ক
    assert_ne!(text, "ক");
}

// 4. Halant / conjunct
#[test]
fn test_probhat_halant_conjunct_ka_ca() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    type_key(&mut engine, '/'); // ্
    type_key(&mut engine, 'c'); // চ
    assert_eq!(engine.get_text(), "ক্চ");
}

#[test]
fn test_probhat_halant_conjunct_ka_khiya() {
    let mut engine = probhat_engine();
    // ক + ্ + ষ = ক্ষ ; ষ is shift+s
    type_key(&mut engine, 'k'); // ক
    type_key(&mut engine, '/'); // ্
    type_key_shift(&mut engine, 's'); // ষ
    assert_eq!(engine.get_text(), "ক্ষ");
}

#[test]
fn test_probhat_halant_conjunct_ta_ra() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'f'); // ত
    type_key(&mut engine, '/'); // ্
    type_key(&mut engine, 'r'); // র
    assert_eq!(engine.get_text(), "ত্র");
}

// 5. Common words via Probhat keys
#[test]
fn test_probhat_word_katha() {
    // কথা: ক (k) + থ (shift+f) + া (a)
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    type_key_shift(&mut engine, 'f'); // থ
    type_key(&mut engine, 'a'); // া
    assert_eq!(engine.get_text(), "কথা");
}

#[test]
fn test_probhat_word_desh() {
    // দেশ: দ (q) + ে ([) + শ (x)
    let mut engine = probhat_engine();
    type_key(&mut engine, 'q'); // দ
    type_key(&mut engine, '['); // ে
    type_key(&mut engine, 'x'); // শ
    assert_eq!(engine.get_text(), "দেশ");
}

#[test]
fn test_probhat_word_bangla_via_keys() {
    // বাংলা: ব (b) + া (a) + ং (shift+l) + ল (l) + া (a)
    // Actually ং is on shift+l, so need shift
    let mut engine = probhat_engine();
    type_key(&mut engine, 'b'); // ব
    type_key(&mut engine, 'a'); // া -> বা
    assert_eq!(engine.get_text(), "বা");
    type_key_shift(&mut engine, 'l'); // ং -> বাং
    assert_eq!(engine.get_text(), "বাং");
    type_key(&mut engine, 'l'); // ল -> বাংল? Need to check composition
    let text_after_la = engine.get_text();
    assert!(text_after_la.contains('ল'), "বাংল should contain ল");
    type_key(&mut engine, 'a'); // া -> বাংলা
    assert_eq!(engine.get_text(), "বাংলা");
}

#[test]
fn test_probhat_word_manush() {
    // মানুষ: ম (m) + া (a) + ন (n) + ু (u) + ষ (shift+s)
    let mut engine = probhat_engine();
    type_key(&mut engine, 'm'); // ম
    type_key(&mut engine, 'a'); // া -> মা
    type_key(&mut engine, 'n'); // ন -> মান
    type_key(&mut engine, 'u'); // ু -> মানু
    type_key_shift(&mut engine, 's'); // ষ -> মানুষ
    assert_eq!(engine.get_text(), "মানুষ");
}

#[test]
fn test_probhat_word_bangladesh() {
    // বাংলাদেশ: ব (b) + া (a) + ং (shift+l) + ল (l) + া (a) + দ (q) + ে ([) + শ (x)
    let mut engine = probhat_engine();
    type_key(&mut engine, 'b');
    type_key(&mut engine, 'a');
    type_key_shift(&mut engine, 'l');
    type_key(&mut engine, 'l');
    type_key(&mut engine, 'a');
    assert_eq!(engine.get_text(), "বাংলা");
    type_key(&mut engine, 'q'); // দ
    type_key(&mut engine, '['); // ে
    type_key(&mut engine, 'x'); // শ
    assert_eq!(engine.get_text(), "বাংলাদেশ");
}

// 6. Backspace deletes full grapheme
#[test]
fn test_probhat_backspace_grapheme() {
    let mut engine = probhat_engine();
    // Build ক্ষ : ক + ্ + ষ
    type_key(&mut engine, 'k');
    type_key(&mut engine, '/');
    type_key_shift(&mut engine, 's');
    assert_eq!(engine.get_text(), "ক্ষ");
    // Backspace one grapheme cluster (3 codepoints)
    let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    // Should backspace the whole cluster (3 chars)
    use keybroad_core::types::OutputAction;
    assert_eq!(actions[0], OutputAction::Backspace(3));
    assert_eq!(engine.get_text(), "");
}

#[test]
fn test_probhat_backspace_simple() {
    let mut engine = probhat_engine();
    type_key(&mut engine, 'k'); // ক
    assert_eq!(engine.get_text(), "ক");
    let _ = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert_eq!(engine.get_text(), "");
}

#[test]
fn test_probhat_layout_type_exists() {
    let engine = BengaliEngine::new(LayoutType::Probhat);
    assert_eq!(engine.get_state().layout, LayoutType::Probhat);
    assert!(engine.get_active_layout().is_some());
}

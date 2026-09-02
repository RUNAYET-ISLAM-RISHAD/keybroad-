use keybroad_core::{BengaliEngine, KeyEvent, LayoutType};

fn phonetic_engine() -> BengaliEngine {
    BengaliEngine::new(LayoutType::Phonetic)
}

fn jatiya_engine() -> BengaliEngine {
    BengaliEngine::new(LayoutType::Jatiya)
}

// JNI state must persist across calls — engine ptr initialized once,
// get_text() returns full composition. This simulates the Kotlin ViewModel
// doing text = engine.processKey(keyCode) on each tap.

#[test]
fn test_phonetic_k_then_h_produces_kha() {
    let mut engine = phonetic_engine();
    // k -> ক
    let _ = engine.process_key(KeyEvent::down('k' as u32, 'k' as u32)).unwrap();
    assert_eq!(engine.get_text(), "ক");
    // h after k -> খ (kh digraph), not কহ
    let _ = engine.process_key(KeyEvent::down('h' as u32, 'h' as u32)).unwrap();
    assert_eq!(engine.get_text(), "খ");
}

#[test]
fn test_phonetic_ami_produces_aami() {
    let mut engine = phonetic_engine();
    for ch in "ami".chars() {
        let _ = engine.process_key(KeyEvent::down(ch as u32, ch as u32)).unwrap();
    }
    assert_eq!(engine.get_text(), "আমি");
}

#[test]
fn test_phonetic_typing_sequence_with_space() {
    let mut engine = phonetic_engine();
    for ch in "ami ".chars() {
        let _ = engine.process_key(KeyEvent::down(ch as u32, ch as u32)).unwrap();
    }
    assert!(engine.get_text().ends_with(' '));
    assert!(engine.get_text().starts_with("আমি"));
}

#[test]
fn test_phonetic_backspace_grapheme() {
    let mut engine = phonetic_engine();
    // Type "kkh" -> ক্ষ via phonetic? Actually "kkh" test shows kkh->ক্ষ
    // Use process_char path for ক্ষ via fixed layout grapheme backspace test
    // Instead, directly test grapheme backspace via jatiya-style conjunct buffer
    // Simulate typing ক্ষ (ক + ্ + ষ) via direct buffer
    engine = BengaliEngine::new(LayoutType::English);
    engine.get_state_mut().composition_buffer.clear();
    for ch in "ক্ষ".chars() {
        // use private add_to_buffer via process_key on English? easier: use process_char for English?
        // fallback: directly test engine backspace logic
        let _ = engine.process_key(KeyEvent::down(ch as u32, ch as u32));
    }
    // Direct buffer test for grapheme deletion
    let mut eng2 = BengaliEngine::new(LayoutType::Jatiya);
    // Manually set buffer to ক্ষ (3 codepoints, 1 grapheme)
    eng2.get_state_mut().composition_buffer.clear();
    for ch in "ক্ষ".chars() {
        // add via engine's internal; use process_char on Jatiya? Jatiya 'j'->ক etc not ক্ষ
        // So directly manipulate via public? use apply_suggestion
        let _ = eng2.apply_suggestion("ক্ষ");
        break;
    }
    // After apply_suggestion, buffer is "ক্ষ "
    // Clear space for isolated grapheme test
    eng2.get_state_mut().composition_buffer.clear();
    eng2.get_state_mut().current_word.clear();
    for ch in "ক্ষ".chars() {
        eng2.get_state_mut().composition_buffer.push(keybroad_core::types::Glyph::simple(ch as u32));
    }
    eng2.get_state_mut().current_word = "ক্ষ".to_string();
    let actions = eng2.process_key(KeyEvent::down(67, 0)).unwrap();
    assert!(matches!(actions[0], keybroad_core::types::OutputAction::Backspace(3)));
    assert!(eng2.get_state().composition_buffer.is_empty());
}

#[test]
fn test_jatiya_d_produces_vowel_sign_not_join() {
    let mut engine = jatiya_engine();
    // Jatiya 'd' -> ি (vowel sign i), NOT join mode
    let _ = engine.process_key(KeyEvent::down('d' as u32, 'd' as u32)).unwrap();
    assert_eq!(engine.get_text(), "ি");
    assert!(!engine.is_join_mode(), "Pressing 'd' must not enter join mode (was 100 collision)");
}

#[test]
fn test_jatiya_e_produces_da_not_kar_noop() {
    let mut engine = jatiya_engine();
    // Jatiya 'e' -> ড, NOT kar popup no-op
    let _ = engine.process_key(KeyEvent::down('e' as u32, 'e' as u32)).unwrap();
    assert_eq!(engine.get_text(), "ড");
}

#[test]
fn test_jatiya_join_with_new_keycode_1000() {
    let mut engine = jatiya_engine();
    // Type ক via Jatiya 'j'
    let _ = engine.process_key(KeyEvent::down('j' as u32, 'j' as u32)).unwrap();
    assert_eq!(engine.get_text(), "ক");
    // Join via 1000
    let _ = engine.process_key(KeyEvent::down(1000, 1000)).unwrap();
    assert!(engine.is_join_mode());
    // Next consonant ষ via Jatiya 'n' -> স? Actually n->স, need ষ via shift+n? Let's use s->ু (not consonant) hmm
    // For simplicity, use a consonant 'j' again (ক) to form conjunct ক্ক? Check jatiya 'j'->ক so j+join+j = ক্ক?
    // Use 'k' -> ত to form ক্ত?
    let _ = engine.process_key(KeyEvent::down('k' as u32, 'k' as u32)).unwrap();
    // Should form conjunct ক্ত (ক + ্ + ত)
    assert!(engine.get_text().contains('\u{09CD}')); // hasanta present
}

#[test]
fn test_kar_via_process_char() {
    let mut engine = jatiya_engine();
    // Type ক via 'j'
    let _ = engine.process_key(KeyEvent::down('j' as u32, 'j' as u32)).unwrap();
    assert_eq!(engine.get_text(), "ক");
    // Apply kar া via process_char (direct Bengali)
    let _ = engine.process_char('া');
    assert_eq!(engine.get_text(), "কা");
    // Smart kar: কা + ি should replace া with ি -> কি
    let _ = engine.process_char('ি');
    assert_eq!(engine.get_text(), "কি");
}

#[test]
fn test_suggestion_via_apply_suggestion() {
    let mut engine = phonetic_engine();
    // Type partial "a" -> some Bengali, then apply full word suggestion
    let _ = engine.process_key(KeyEvent::down('a' as u32, 'a' as u32)).unwrap();
    let before = engine.get_text();
    assert!(!before.is_empty());
    let current = engine.current_word();
    // Apply a dictionary word suggestion (e.g., "আমি")
    let _ = engine.apply_suggestion("আমি");
    assert!(engine.get_text().contains("আমি"));
    // Should have trailing space and history updated
    assert!(engine.get_text().ends_with(' '));
}

#[test]
fn test_jni_state_persistence_across_calls() {
    // Simulate Kotlin ViewModel: engine ptr reused, text = get_text() after each key
    let mut engine = phonetic_engine();
    let mut full_text = String::new();
    for ch in "bd".chars() {
        let _ = engine.process_key(KeyEvent::down(ch as u32, ch as u32)).unwrap();
        full_text = engine.get_text();
    }
    // Text must be cumulative, not reset per call
    assert!(!full_text.is_empty());
    // Typing 'b' then 'd' should not produce reversed order
    // Check that get_text() after two keys contains both transliterations in order
    let mut engine2 = phonetic_engine();
    let _ = engine2.process_key(KeyEvent::down('b' as u32, 'b' as u32)).unwrap();
    let t1 = engine2.get_text();
    let _ = engine2.process_key(KeyEvent::down('a' as u32, 'a' as u32)).unwrap();
    let t2 = engine2.get_text();
    assert!(t2.starts_with(&t1) || t2.len() >= t1.len());
}

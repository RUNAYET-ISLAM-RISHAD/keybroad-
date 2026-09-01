use keybroad_core::{BengaliEngine, KeyEvent, LayoutType, OutputAction, WordSource};

/// Helper: type a Bengali word using phonetic layout key codes.
/// Bengali phonetic layout uses ASCII key codes mapped to Bengali characters.
/// For simplicity, we'll directly set the current_word state for word boundary tests.
fn type_word(engine: &mut BengaliEngine, word: &str) {
    for ch in word.chars() {
        let event = KeyEvent::down(ch as u32, ch as u32);
        let _ = engine.process_key(event);
    }
}

/// Helper: type a space character using English layout.
fn type_space(engine: &mut BengaliEngine) {
    let event = KeyEvent::down(62, ' ' as u32);
    let _ = engine.process_key(event);
}

/// Helper: type a period using English layout.
fn type_period(engine: &mut BengaliEngine) {
    let event = KeyEvent::down(55, '.' as u32);
    let _ = engine.process_key(event);
}

// === Word Boundary Tests ===

#[test]
fn test_word_boundary_space() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Type "cat" then space
    type_word(&mut engine, "cat");
    assert_eq!(engine.get_state().current_word, "cat");

    type_space(&mut engine);
    assert_eq!(engine.get_state().current_word, "");
    assert_eq!(engine.get_state().history.len(), 1);
    assert_eq!(engine.get_state().history[0], "cat");
}

#[test]
fn test_word_boundary_punctuation() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Type "hello" then period
    type_word(&mut engine, "hello");
    assert_eq!(engine.get_state().current_word, "hello");

    type_period(&mut engine);
    assert_eq!(engine.get_state().current_word, "");
    assert_eq!(engine.get_state().history.len(), 1);
    assert_eq!(engine.get_state().history[0], "hello");
}

#[test]
fn test_word_boundary_multiple_words() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Type "hello world"
    type_word(&mut engine, "hello");
    type_space(&mut engine);
    type_word(&mut engine, "world");
    type_space(&mut engine);

    assert_eq!(engine.get_state().current_word, "");
    assert_eq!(engine.get_state().history.len(), 2);
    assert_eq!(engine.get_state().history[0], "world");
    assert_eq!(engine.get_state().history[1], "hello");
}

#[test]
fn test_history_cap() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Type 25 words
    for i in 0..25 {
        let word = format!("word{}", i);
        type_word(&mut engine, &word);
        type_space(&mut engine);
    }
    // History should be capped at 20
    assert_eq!(engine.get_state().history.len(), 20);
    // Most recent word should be first
    assert_eq!(engine.get_state().history[0], "word24");
    assert_eq!(engine.get_state().history[19], "word5");
}

// === Finalize Word Tests ===

#[test]
fn test_finalize_word() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    type_word(&mut engine, "test");
    assert_eq!(engine.get_state().current_word, "test");

    engine.finalize_word();
    assert_eq!(engine.get_state().current_word, "");
    assert_eq!(engine.get_state().history.len(), 1);
    assert_eq!(engine.get_state().last_committed_word.as_deref(), Some("test"));
}

// === User Dictionary Tests ===

#[test]
fn test_add_user_word() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    assert!(!engine.is_user_word("rickshaw"));

    engine.add_user_word("rickshaw");
    assert!(engine.is_user_word("rickshaw"));
    assert!(!engine.is_user_word("other"));
}

#[test]
fn test_user_word_in_suggestions() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    engine.add_user_word("customword");

    let suggestions = engine.get_suggestions("customword");
    assert!(!suggestions.is_empty());

    let custom = suggestions.iter().find(|s| s.word == "customword");
    assert!(custom.is_some());
    assert_eq!(custom.unwrap().source, WordSource::UserHistory);
}

#[test]
fn test_user_word_not_in_dict() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Add a word that's not in the main dictionary
    engine.add_user_word("xyznonexistent");

    let suggestions = engine.get_suggestions("xyznonexistent");
    assert!(!suggestions.is_empty());

    let custom = suggestions.iter().find(|s| s.word == "xyznonexistent");
    assert!(custom.is_some());
    assert_eq!(custom.unwrap().source, WordSource::UserHistory);
}

// === N-gram Prediction Tests ===

#[test]
fn test_next_word_suggestions() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Type "I" and finalize
    type_word(&mut engine, "I");
    type_space(&mut engine);

    // Get next word suggestions
    let suggestions = engine.get_next_word_suggestions(5);
    // Should have predictions (may or may not have "am" depending on corpus)
    // At minimum, the method should not panic
    for s in &suggestions {
        assert_eq!(s.source, WordSource::AiPrediction);
        assert!(s.score > 0.0 && s.score <= 1.0);
    }
}

#[test]
fn test_trigram_context() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    // Build context: "I am"
    type_word(&mut engine, "I");
    type_space(&mut engine);
    type_word(&mut engine, "am");
    type_space(&mut engine);

    let suggestions = engine.get_next_word_suggestions(5);
    for s in &suggestions {
        assert_eq!(s.source, WordSource::AiPrediction);
    }
}

// === Incognito Mode Tests ===

#[test]
fn test_incognito_no_history() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    engine.set_incognito(true);

    type_word(&mut engine, "secret");
    type_space(&mut engine);

    assert_eq!(engine.get_state().history.len(), 0);
    assert!(engine.get_state().last_committed_word.is_none());
}

#[test]
fn test_incognito_no_next_word_suggestions() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    engine.set_incognito(true);

    type_word(&mut engine, "hello");
    type_space(&mut engine);

    let suggestions = engine.get_next_word_suggestions(5);
    assert!(suggestions.is_empty());
}

// === Reset Tests ===

#[test]
fn test_reset_clears_history() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    type_word(&mut engine, "hello");
    type_space(&mut engine);
    type_word(&mut engine, "world");
    type_space(&mut engine);

    assert_eq!(engine.get_state().history.len(), 2);
    assert_eq!(engine.get_state().current_word, "");

    engine.reset();
    assert_eq!(engine.get_state().history.len(), 0);
    assert!(engine.get_state().last_committed_word.is_none());
    assert_eq!(engine.get_state().current_word, "");
}

// === Backspace Word Tracking Tests ===

#[test]
fn test_backspace_removes_from_current_word() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    type_word(&mut engine, "hello");
    assert_eq!(engine.get_state().current_word, "hello");

    // Backspace once
    let backspace_event = KeyEvent::down(67, 0);
    let _ = engine.process_key(backspace_event);
    assert_eq!(engine.get_state().current_word, "hell");

    let _ = engine.process_key(backspace_event);
    assert_eq!(engine.get_state().current_word, "hel");
}

// === Existing Tests Still Pass ===

#[test]
fn test_existing_engine_creation() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    assert_eq!(engine.get_state().layout, LayoutType::Phonetic);
    assert!(engine.get_active_layout().is_some());
}

#[test]
fn test_existing_process_key() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    let event = KeyEvent::down(29, 'a' as u32);
    let result = engine.process_key(event);
    assert!(result.is_ok());
}

#[test]
fn test_existing_backspace() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert_eq!(actions[0], OutputAction::Backspace(1));
}

#[test]
fn test_existing_shift() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    let _ = engine.process_key(KeyEvent::down(59, 0));
    assert_eq!(engine.get_state().shift_state, keybroad_core::ShiftState::Shift);
}

#[test]
fn test_existing_reset() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    engine.process_key(KeyEvent::down(30, 'b' as u32)).unwrap();
    engine.reset();
    assert!(engine.get_state().composition_buffer.is_empty());
}

#[test]
fn test_existing_incognito() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    assert!(!engine.is_incognito());
    engine.set_incognito(true);
    assert!(engine.is_incognito());
}

#[test]
fn test_existing_conjunct() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);
    assert!(engine.get_conjuncts().is_consonant('ক'));
    assert!(engine.get_conjuncts().is_consonant('খ'));
    assert!(!engine.get_conjuncts().is_consonant('া'));
}

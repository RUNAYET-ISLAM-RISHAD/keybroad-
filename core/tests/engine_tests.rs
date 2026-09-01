/// Integration tests for the Bengali keyboard engine.
///
/// These tests verify the engine works correctly as a whole,
/// testing multiple keystrokes and state transitions.

use keybroad_core::{BengaliEngine, ConjunctTable, KeyEvent, LayoutType, OutputAction, ShiftState};

// === English layout tests ===

#[test]
fn test_typing_multiple_characters() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Type "hello"
    let chars = vec![
        KeyEvent::down(36, 'h' as u32),
        KeyEvent::down(33, 'e' as u32),
        KeyEvent::down(38, 'l' as u32),
        KeyEvent::down(38, 'l' as u32),
        KeyEvent::down(39, 'o' as u32),
    ];

    let mut all_actions = Vec::new();
    for event in chars {
        let actions = engine.process_key(event).unwrap();
        all_actions.extend(actions);
    }

    // Should have 5 CommitText actions
    let commit_actions: Vec<_> = all_actions
        .iter()
        .filter(|a| matches!(a, OutputAction::CommitText(_)))
        .collect();
    assert_eq!(commit_actions.len(), 5);
}

#[test]
fn test_type_and_backspace() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Type "ab"
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    engine.process_key(KeyEvent::down(30, 'b' as u32)).unwrap();

    // Backspace
    let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert_eq!(actions[0], OutputAction::Backspace(1));

    // Buffer should have 1 character left
    assert_eq!(engine.get_state().composition_buffer.len(), 1);
}

#[test]
fn test_shift_then_char() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Press shift
    engine.process_key(KeyEvent::down(59, 0)).unwrap();
    assert_eq!(engine.get_state().shift_state, ShiftState::Shift);

    // Type 'a' — should produce shifted output "A"
    let actions = engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("A".to_string()));

    // Shift should auto-reset after single use
    assert_eq!(engine.get_state().shift_state, ShiftState::None);
}

#[test]
fn test_incognito_mode_active() {
    let mut engine = BengaliEngine::new(LayoutType::English);
    engine.set_incognito(true);

    // Typing should still work
    let actions = engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("a".to_string()));

    // But incognito flag should be set
    assert!(engine.is_incognito());
}

#[test]
fn test_engine_state_immutability() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Get initial state
    let initial_layout = engine.get_state().layout;
    assert_eq!(initial_layout, LayoutType::Phonetic);

    // Process a key
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();

    // Layout should not change
    assert_eq!(engine.get_state().layout, LayoutType::Phonetic);
}

#[test]
fn test_enter_commits_composition() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Type a character
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();

    // Press enter to commit
    let actions = engine.process_key(KeyEvent::down(66, 0)).unwrap();

    // Should have Backspace(1) + CommitText("a")
    assert!(actions.contains(&OutputAction::Backspace(1)));
    assert!(actions.contains(&OutputAction::CommitText("a".to_string())));
}

#[test]
fn test_reset_clears_state() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Build up some state
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    engine.process_key(KeyEvent::down(59, 0)).unwrap(); // shift

    assert!(!engine.get_state().composition_buffer.is_empty());

    // Reset
    engine.reset();

    assert!(engine.get_state().composition_buffer.is_empty());
    assert!(engine.get_state().candidates.is_empty());
    assert_eq!(engine.get_state().cursor_position, 0);
    assert_eq!(engine.get_state().shift_state, ShiftState::None);
}

#[test]
fn test_layout_switch_resets_state() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type something
    engine.process_key(KeyEvent::down(29, 'a' as u32)).unwrap();
    assert!(!engine.get_state().composition_buffer.is_empty());

    // Switch layout — should reset
    engine.set_layout(LayoutType::English);
    assert_eq!(engine.get_state().layout, LayoutType::English);
    assert!(engine.get_state().composition_buffer.is_empty());
}

#[test]
fn test_rapid_key_presses() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Simulate rapid typing of 100 characters
    for i in 0..100 {
        let key_code = 29 + (i % 26); // Cycle through a-z
        let unicode = b'a' + (i % 26) as u8;
        let actions = engine.process_key(KeyEvent::down(key_code, unicode as u32)).unwrap();

        // Every keystroke should produce exactly one action
        assert_eq!(actions.len(), 1);
    }

    // Buffer should have 100 characters
    assert_eq!(engine.get_state().composition_buffer.len(), 100);
}

// === Phonetic layout tests ===

#[test]
fn test_phonetic_k_to_ka() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' — should produce ক (0x0995)
    let actions = engine.process_key(KeyEvent::from_char('k')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::CommitText("\u{0995}".to_string()));
}

#[test]
fn test_phonetic_a_to_aa_kar() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'a' — should produce া (0x09be, aa-kar)
    let actions = engine.process_key(KeyEvent::from_char('a')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::CommitText("\u{09be}".to_string()));
}

#[test]
fn test_phonetic_shift_k_to_kha() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Press shift, then 'k' — should produce খ (0x0996)
    engine.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    let actions = engine.process_key(KeyEvent::from_char('k')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::CommitText("\u{0996}".to_string()));
}

#[test]
fn test_phonetic_shift_g_to_gha() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Press shift, then 'g' — should produce ঘ (0x0998)
    engine.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    let actions = engine.process_key(KeyEvent::from_char('g')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::CommitText("\u{0998}".to_string()));
}

#[test]
fn test_phonetic_typing_word() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type "kama" (ক + া + ম + া) — should produce "কামা"
    let actions_k = engine.process_key(KeyEvent::from_char('k')).unwrap();
    let actions_a = engine.process_key(KeyEvent::from_char('a')).unwrap();
    let actions_m = engine.process_key(KeyEvent::from_char('m')).unwrap();
    let actions_a2 = engine.process_key(KeyEvent::from_char('a')).unwrap();

    assert_eq!(actions_k[0], OutputAction::CommitText("\u{0995}".to_string())); // ক
    assert_eq!(actions_a[0], OutputAction::CommitText("\u{09be}".to_string())); // া
    assert_eq!(actions_m[0], OutputAction::CommitText("\u{09ae}".to_string())); // ম
    assert_eq!(actions_a2[0], OutputAction::CommitText("\u{09be}".to_string())); // া

    // Composition buffer should have 4 glyphs
    assert_eq!(engine.get_state().composition_buffer.len(), 4);
}

#[test]
fn test_phonetic_backspace_removes_last() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type "ka" (ক + া)
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    engine.process_key(KeyEvent::from_char('a')).unwrap();
    assert_eq!(engine.get_state().composition_buffer.len(), 2);

    // Backspace — should remove া
    let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert_eq!(actions[0], OutputAction::Backspace(1));
    assert_eq!(engine.get_state().composition_buffer.len(), 1);

    // Verify remaining glyph is ক
    let remaining = &engine.get_state().composition_buffer[0];
    assert_eq!(remaining.unicode, 0x0995); // ক
}

#[test]
fn test_phonetic_enter_commits() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type "ka" (ক + া)
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    engine.process_key(KeyEvent::from_char('a')).unwrap();

    // Press enter to commit
    let actions = engine.process_key(KeyEvent::down(66, 0)).unwrap();

    // Should have Backspace(1) + CommitText("কা")
    assert!(actions.contains(&OutputAction::Backspace(1)));
    assert!(actions.contains(&OutputAction::CommitText("\u{0995}\u{09be}".to_string())));
}

#[test]
fn test_phonetic_layout_loaded() {
    let engine = BengaliEngine::new(LayoutType::Phonetic);

    // Verify layout is loaded
    let layout = engine.get_active_layout().unwrap();
    assert_eq!(layout.id, LayoutType::Phonetic);
    assert!(layout.key_count() > 0);

    // Verify specific mappings exist
    assert!(layout.has_key('k' as u32));
    assert!(layout.has_key('a' as u32));
    assert!(layout.has_key('z' as u32));
}

#[test]
fn test_english_shift_produces_uppercase() {
    let mut engine = BengaliEngine::new(LayoutType::English);

    // Type 'a' — should produce "a"
    let actions = engine.process_key(KeyEvent::from_char('a')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("a".to_string()));

    // Press shift + 'b' — should produce "B"
    engine.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    let actions = engine.process_key(KeyEvent::from_char('b')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("B".to_string()));
}

// === Conjunct handling tests ===

#[test]
fn test_conjunct_table_consonants() {
    let table = ConjunctTable::new();

    // All Bengali consonants should be recognized
    assert!(table.is_consonant('ক'));
    assert!(table.is_consonant('খ'));
    assert!(table.is_consonant('গ'));
    assert!(table.is_consonant('ঘ'));
    assert!(table.is_consonant('ঙ'));
    assert!(table.is_consonant('চ'));
    assert!(table.is_consonant('ছ'));
    assert!(table.is_consonant('জ'));
    assert!(table.is_consonant('ঝ'));
    assert!(table.is_consonant('ঞ'));
    assert!(table.is_consonant('ট'));
    assert!(table.is_consonant('ঠ'));
    assert!(table.is_consonant('ড'));
    assert!(table.is_consonant('ঢ'));
    assert!(table.is_consonant('ণ'));
    assert!(table.is_consonant('ত'));
    assert!(table.is_consonant('থ'));
    assert!(table.is_consonant('দ'));
    assert!(table.is_consonant('ধ'));
    assert!(table.is_consonant('ন'));
    assert!(table.is_consonant('প'));
    assert!(table.is_consonant('ফ'));
    assert!(table.is_consonant('ব'));
    assert!(table.is_consonant('ভ'));
    assert!(table.is_consonant('ম'));
    assert!(table.is_consonant('য'));
    assert!(table.is_consonant('র'));
    assert!(table.is_consonant('ল'));
    assert!(table.is_consonant('শ'));
    assert!(table.is_consonant('ষ'));
    assert!(table.is_consonant('স'));
    assert!(table.is_consonant('হ'));

    // Non-consonants should NOT be recognized
    assert!(!table.is_consonant('া'));
    assert!(!table.is_consonant('ি'));
    assert!(!table.is_consonant('ী'));
    assert!(!table.is_consonant('ে'));
    assert!(!table.is_consonant('ৈ'));
    assert!(!table.is_consonant('ো'));
    assert!(!table.is_consonant('ৌ'));
    assert!(!table.is_consonant('k'));
    assert!(!table.is_consonant('a'));
    assert!(!table.is_consonant('0'));
}

#[test]
fn test_hasanta_after_consonant_backspaces() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' → outputs ক
    let actions = engine.process_key(KeyEvent::from_char('k')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("\u{0995}".to_string())); // ক
    assert_eq!(engine.get_state().hasanta_base_consonant, Some('\u{0995}'));

    // Type '\' (hasanta) → should emit Backspace(1) to remove ক
    let actions = engine.process_key(KeyEvent::from_char('\\')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::Backspace(1));
    assert!(engine.get_state().hasanta_pending);
}

#[test]
fn test_conjunct_formation_kg() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' → ক
    engine.process_key(KeyEvent::from_char('k')).unwrap();

    // Type '\' (hasanta) → Backspace(1)
    engine.process_key(KeyEvent::from_char('\\')).unwrap();

    // Type 'g' → should output ক্গ (ক+্+গ)
    let actions = engine.process_key(KeyEvent::from_char('g')).unwrap();
    assert_eq!(actions.len(), 1);
    let expected = "\u{0995}\u{09CD}\u{0997}"; // ক্গ
    assert_eq!(actions[0], OutputAction::CommitText(expected.to_string()));

    // hasanta_pending should be cleared
    assert!(!engine.get_state().hasanta_pending);
    assert!(engine.get_state().hasanta_base_consonant.is_none());
}

#[test]
fn test_conjunct_formation_ksh() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' → ক
    engine.process_key(KeyEvent::from_char('k')).unwrap();

    // Type '\' (hasanta)
    engine.process_key(KeyEvent::from_char('\\')).unwrap();

    // Type shift+'s' → ষ (should form ক্ষ)
    engine.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    let actions = engine.process_key(KeyEvent::from_char('s')).unwrap();
    assert_eq!(actions.len(), 1);
    let text = match &actions[0] {
        OutputAction::CommitText(t) => t.clone(),
        _ => panic!("Expected CommitText"),
    };
    assert!(text.contains('\u{0995}')); // ক
    assert!(text.contains('\u{09CD}')); // ্
    assert!(text.contains('\u{09B7}')); // ষ
}

#[test]
fn test_special_conjunct_x_ka_ksha() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'x' → should produce ক্ষ via backspace trick
    // Process: 'ক' → CommitText, '্' → Backspace(1), 'ষ' → CommitText("ক্ষ")
    let actions = engine.process_key(KeyEvent::from_char('x')).unwrap();
    // Should have 3 actions: CommitText(ক), Backspace(1), CommitText(ক্ষ)
    assert!(actions.len() >= 2);
    // The final action should contain the conjunct
    let last_action = actions.last().unwrap();
    match last_action {
        OutputAction::CommitText(t) => {
            assert!(t.contains('\u{0995}')); // ক
            assert!(t.contains('\u{09CD}')); // ্
            assert!(t.contains('\u{09B7}')); // ষ
        }
        _ => panic!("Expected CommitText with conjunct"),
    }
}

#[test]
fn test_special_conjunct_z_jnya() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'z' → should produce জ্ঞ via backspace trick
    let actions = engine.process_key(KeyEvent::from_char('z')).unwrap();
    assert!(actions.len() >= 2);
    let last_action = actions.last().unwrap();
    match last_action {
        OutputAction::CommitText(t) => {
            assert!(t.contains('\u{099C}')); // জ
            assert!(t.contains('\u{09CD}')); // ্
        }
        _ => panic!("Expected CommitText with conjunct"),
    }
}

#[test]
fn test_backspace_during_hasanta_pending() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' → ক
    engine.process_key(KeyEvent::from_char('k')).unwrap();

    // Type '\' (hasanta) → Backspace(1), sets hasanta_pending
    engine.process_key(KeyEvent::from_char('\\')).unwrap();
    assert!(engine.get_state().hasanta_pending);

    // Backspace → should clear hasanta_pending
    let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert_eq!(actions[0], OutputAction::Nothing);
    assert!(!engine.get_state().hasanta_pending);

    // Type 'g' → should output গ (not a conjunct, since hasanta was cleared)
    let actions = engine.process_key(KeyEvent::from_char('g')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("\u{0997}".to_string())); // গ
}

#[test]
fn test_backspace_after_conjunct() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' + '\' + 'g' → ক্গ (3 codepoints in buffer)
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    engine.process_key(KeyEvent::from_char('\\')).unwrap();
    engine.process_key(KeyEvent::from_char('g')).unwrap();

    let buffer_before = engine.get_state().composition_buffer.len();
    assert_eq!(buffer_before, 3); // ক, ্, গ

    // Backspace → should remove 1 codepoint from the conjunct
    let actions = engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert_eq!(actions[0], OutputAction::Backspace(1));

    // Buffer should have 2 characters left (one codepoint removed)
    assert_eq!(engine.get_state().composition_buffer.len(), 2);
}

#[test]
fn test_hasanta_without_consonant_outputs_directly() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type '\' (hasanta) without preceding consonant → outputs ্ directly
    let actions = engine.process_key(KeyEvent::from_char('\\')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::CommitText("\u{09CD}".to_string())); // ্
}

#[test]
fn test_conjunct_with_vowel_after() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' + '\' + 'g' → ক্গ
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    engine.process_key(KeyEvent::from_char('\\')).unwrap();
    engine.process_key(KeyEvent::from_char('g')).unwrap();

    // Type 'a' (aa-kar) → should add া after the conjunct
    let actions = engine.process_key(KeyEvent::from_char('a')).unwrap();
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], OutputAction::CommitText("\u{09BE}".to_string())); // া
}

#[test]
fn test_multiple_conjuncts_in_sequence() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type "k + \ + g" → ক্গ
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    engine.process_key(KeyEvent::from_char('\\')).unwrap();
    engine.process_key(KeyEvent::from_char('g')).unwrap();

    // Type "t + \ + t" → ত্ত
    engine.process_key(KeyEvent::from_char('t')).unwrap();
    engine.process_key(KeyEvent::from_char('\\')).unwrap();
    engine.process_key(KeyEvent::from_char('t')).unwrap();

    // Both conjuncts should be in the buffer
    let buffer = &engine.get_state().composition_buffer;
    assert!(buffer.len() >= 6); // At least 6 codepoints for two conjuncts
}

#[test]
fn test_conjunct_enter_commits() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' + '\' + 'g' → ক্গ
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    engine.process_key(KeyEvent::from_char('\\')).unwrap();
    engine.process_key(KeyEvent::from_char('g')).unwrap();

    // Press enter to commit
    let actions = engine.process_key(KeyEvent::down(66, 0)).unwrap();

    // Should have Backspace(1) + CommitText with the conjunct
    assert!(actions.contains(&OutputAction::Backspace(1)));
    let commit_action = actions.iter().find(|a| matches!(a, OutputAction::CommitText(_)));
    assert!(commit_action.is_some());
}

#[test]
fn test_conjunct_shift_clears_after_use() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Press shift
    engine.process_key(KeyEvent::down(59, 0)).unwrap();
    assert_eq!(engine.get_state().shift_state, ShiftState::Shift);

    // Type 'k' with shift → outputs খ (shift+k in phonetic layout)
    let actions = engine.process_key(KeyEvent::from_char('k')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("\u{0996}".to_string())); // খ

    // Shift should auto-reset
    assert_eq!(engine.get_state().shift_state, ShiftState::None);

    // Type 'k' again → should output ক (no shift)
    let actions = engine.process_key(KeyEvent::from_char('k')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("\u{0995}".to_string())); // ক
}

#[test]
fn test_backspace_after_consonant_clears_base() {
    let mut engine = BengaliEngine::new(LayoutType::Phonetic);

    // Type 'k' → ক, sets hasanta_base_consonant
    engine.process_key(KeyEvent::from_char('k')).unwrap();
    assert_eq!(engine.get_state().hasanta_base_consonant, Some('\u{0995}'));

    // Backspace → should clear hasanta_base_consonant
    engine.process_key(KeyEvent::down(67, 0)).unwrap();
    assert!(engine.get_state().hasanta_base_consonant.is_none());

    // Type '\' → should output ্ directly (no backspace trick)
    let actions = engine.process_key(KeyEvent::from_char('\\')).unwrap();
    assert_eq!(actions[0], OutputAction::CommitText("\u{09CD}".to_string())); // ্
}

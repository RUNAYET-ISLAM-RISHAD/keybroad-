use keybroad_core::{BengaliEngine, KeyEvent, LayoutType};
use unicode_normalization::UnicodeNormalization;

fn probhat() -> BengaliEngine {
    BengaliEngine::new(LayoutType::Probhat)
}

/// Helper to send keycode as Android UI does: logical ID (e.g., 'k' 107)
fn send(engine: &mut BengaliEngine, key: char) {
    let _ = engine.process_key(KeyEvent::down(key as u32, key as u32)).unwrap();
}

#[test]
fn test_repro_probhat_katha_lowercase() {
    // Probhat: কথা = ক (k) + থ (shift+f) + া (a)  — 3 keystrokes
    let mut e = probhat();
    send(&mut e, 'k'); // ক
    println!("after k: '{}' buf={:?}", e.get_text(), e.get_text().chars().collect::<Vec<_>>());
    let _ = e.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    send(&mut e, 'f'); // থ (shift+f)
    println!("after F(th): '{}'", e.get_text());
    send(&mut e, 'a'); // া -> কথা
    println!("after final a: '{}'", e.get_text());
    let got: String = e.get_text().nfc().collect();
    let expected: String = "কথা".nfc().collect();
    println!("expected='{}' got='{}' expected_debug={:?} got_debug={:?}", expected, got, expected.chars().collect::<Vec<_>>(), got.chars().collect::<Vec<_>>());
    assert_eq!(got, expected, "typing k,shift+f,a should produce কথা");
}

#[test]
fn test_repro_probhat_katha_uppercase_task_spec() {
    // Task spec lists K(107='k'), A(65='A'), F(70='F'), A(65) but Probhat uses
    // lower logical IDs ('k','a','f') with shift flag. Upper 'A' (65) is not a
    // valid Probhat key — engine should return InvalidKeyCode. Verify UI sends
    // lower codes, not upper. Here we just log the behaviour.
    let mut e = probhat();
    let seq = [107u32, 97, 102, 97]; // correct lower: k,a,f,a -> but a after k already forms কা, so this is k,a,f,a = কাথা (wrong order)
    for &code in &seq {
        let ch = char::from_u32(code).unwrap();
        println!("sending keycode {} ('{}')", code, ch);
        let res = e.process_key(KeyEvent::down(code, code));
        println!("  -> res={:?} text='{}'", res, e.get_text());
    }
    println!("final text correct-seq(k,a,f,a): '{}'", e.get_text());
    // Correct কথা is k, shift+f, a
    let mut e2 = probhat();
    send(&mut e2, 'k');
    let _ = e2.process_key(KeyEvent::down(59, 0)).unwrap();
    send(&mut e2, 'f');
    send(&mut e2, 'a');
    println!("correct কথা via k,shift+f,a: '{}'", e2.get_text());
    assert_eq!(e2.get_text().nfc().collect::<String>(), "কথা".nfc().collect::<String>());
}

#[test]
fn test_repro_probhat_bangla() {
    // বাংলা via probhat keys: b->ব, a->া, shift+l->ং, l->ল, a->া
    let mut e = probhat();
    send(&mut e, 'b'); // ব
    send(&mut e, 'a'); // বা
    let _ = e.process_key(KeyEvent::down(59, 0)).unwrap(); // shift
    send(&mut e, 'l'); // ং -> বাং
    send(&mut e, 'l'); // ল -> বাংল
    send(&mut e, 'a'); // া -> বাংলা
    let got: String = e.get_text().nfc().collect();
    let expected: String = "বাংলা".nfc().collect();
    println!("বাংলা got='{}' expected='{}'", got, expected);
    assert_eq!(got, expected);
}

#[test]
fn test_repro_conjunct_k_c() {
    // ক (k) + ্ (/) + চ (c) -> ক্চ
    let mut e = probhat();
    send(&mut e, 'k');
    send(&mut e, '/');
    send(&mut e, 'c');
    let got: String = e.get_text().nfc().collect();
    let expected: String = "ক্চ".nfc().collect();
    println!("conjunct k/c got='{}' expected='{}'", got, expected);
    assert_eq!(got, expected);
}

#[test]
fn test_repro_engine_state_preserved() {
    let mut e = probhat();
    send(&mut e, 'k');
    let t1 = e.get_text();
    send(&mut e, 'a');
    let t2 = e.get_text();
    println!("t1='{}' t2='{}' t2 starts_with t1[0]? {}", t1, t2, t2.starts_with(&t1.chars().next().unwrap().to_string()));
    assert!(t2.len() > t1.len(), "state must accumulate, not reset");
    assert!(t2.starts_with("ক"), "second char should append, not reverse");
}

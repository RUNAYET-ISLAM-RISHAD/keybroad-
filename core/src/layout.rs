/// Layout loading and management.
///
/// This module handles loading keyboard layouts from JSON files
/// and providing key mapping lookups.

use std::collections::HashMap;

use crate::types::{Layout, LayoutJson, LayoutType};

/// Embedded layout JSON files (compiled into the binary).
/// This ensures layouts are always available without filesystem access.
fn get_layout_json(layout_type: LayoutType) -> &'static str {
    match layout_type {
        LayoutType::Phonetic => include_str!("../layouts/phonetic.json"),
        LayoutType::English => include_str!("../layouts/english.json"),
        LayoutType::Jatiya => include_str!("../layouts/jatiya.json"),
        LayoutType::Probhat => include_str!("../layouts/probhat.json"),
        LayoutType::Unijoy => include_str!("../layouts/unijoy.json"),
    }
}

/// Load a keyboard layout from its JSON definition.
///
/// This function parses the embedded JSON and constructs a `Layout` struct
/// with efficient HashMap-based key lookups.
///
/// # Arguments
/// * `layout_type` - The layout to load
///
/// # Returns
/// `Ok(Layout)` if loaded successfully, `Err(EngineError)` otherwise.
pub fn load_layout(layout_type: LayoutType) -> Result<Layout, crate::types::EngineError> {
    let json_str = get_layout_json(layout_type);

    let layout_json: LayoutJson = serde_json::from_str(json_str).map_err(|e| {
        crate::types::EngineError::LayoutLoadError(format!(
            "Failed to parse layout '{}': {}",
            layout_type.as_str(),
            e
        ))
    })?;

    // Convert string-based key map to unicode-based key map
    let mut key_map = HashMap::new();

    for (key_str, mapping) in &layout_json.keys {
        // Each key string is a single character (e.g., "a", "b", "0")
        if let Some(ch) = key_str.chars().next() {
            let unicode = ch as u32;
            key_map.insert(unicode, mapping.clone());
        }
    }

    Ok(Layout {
        id: layout_type,
        name: layout_json.name,
        description: layout_json.description,
        key_map,
        special_keys: layout_json.special_keys,
    })
}

/// Load all available layouts.
///
/// Returns a HashMap of LayoutType -> Layout for all supported layouts.
pub fn load_all_layouts() -> HashMap<LayoutType, Layout> {
    let mut layouts = HashMap::new();

    let all_layouts = [
        LayoutType::Phonetic,
        LayoutType::English,
        LayoutType::Jatiya,
        LayoutType::Probhat,
        LayoutType::Unijoy,
    ];

    for layout_type in all_layouts {
        if let Ok(layout) = load_layout(layout_type) {
            layouts.insert(layout_type, layout);
        }
    }

    layouts
}

/// Parse a single character from a JSON key string to a unicode codepoint.
/// Returns None if the string is not a single character.
pub fn parse_key_string(s: &str) -> Option<u32> {
    let mut chars = s.chars();
    let ch = chars.next()?;
    if chars.next().is_some() {
        return None; // Not a single character
    }
    Some(ch as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_phonetic_layout() {
        let layout = load_layout(LayoutType::Phonetic).unwrap();
        assert_eq!(layout.id, LayoutType::Phonetic);
        assert_eq!(layout.name, "Phonetic");
        assert!(layout.key_count() > 0);
    }

    #[test]
    fn test_load_english_layout() {
        let layout = load_layout(LayoutType::English).unwrap();
        assert_eq!(layout.id, LayoutType::English);
        assert_eq!(layout.name, "English");
        assert!(layout.key_count() > 0);
    }

    #[test]
    fn test_load_jatiya_layout() {
        let layout = load_layout(LayoutType::Jatiya).unwrap();
        assert_eq!(layout.id, LayoutType::Jatiya);
        assert_eq!(layout.name, "Jatiya");
        assert!(layout.key_count() > 0);
    }

    #[test]
    fn test_load_probhat_layout() {
        let layout = load_layout(LayoutType::Probhat).unwrap();
        assert_eq!(layout.id, LayoutType::Probhat);
        assert_eq!(layout.name, "Probhat");
        assert!(layout.key_count() > 0);
    }

    #[test]
    fn test_load_unijoy_layout() {
        let layout = load_layout(LayoutType::Unijoy).unwrap();
        assert_eq!(layout.id, LayoutType::Unijoy);
        assert_eq!(layout.name, "Unijoy");
        assert!(layout.key_count() > 0);
    }

    #[test]
    fn test_phonetic_has_key_mappings() {
        let layout = load_layout(LayoutType::Phonetic).unwrap();

        // Phonetic is now Roman QWERTY (Avro-style) - 'k' maps to "k", not ক
        assert!(layout.has_key('k' as u32));
        let mapping = layout.key_map.get(&('k' as u32)).unwrap();
        assert_eq!(mapping.output, "k");

        // 'a' should map to "a" (Roman), transliteration happens in engine
        assert!(layout.has_key('a' as u32));
        let mapping = layout.key_map.get(&('a' as u32)).unwrap();
        assert_eq!(mapping.output, "a");
    }

    #[test]
    fn test_english_has_key_mappings() {
        let layout = load_layout(LayoutType::English).unwrap();

        // 'a' should map to 'a'
        assert!(layout.has_key('a' as u32));
        let mapping = layout.key_map.get(&('a' as u32)).unwrap();
        assert_eq!(mapping.output, "a");

        // Shift 'a' should map to 'A'
        assert_eq!(mapping.shift_output, "A");
    }

    #[test]
    fn test_jatiya_has_key_mappings() {
        let layout = load_layout(LayoutType::Jatiya).unwrap();

        // Jatiya: j -> ক, i -> হ, g -> hasanta
        assert!(layout.has_key('j' as u32));
        let mapping = layout.key_map.get(&('j' as u32)).unwrap();
        assert_eq!(mapping.output, "ক");

        assert!(layout.has_key('i' as u32));
        let mapping = layout.key_map.get(&('i' as u32)).unwrap();
        assert_eq!(mapping.output, "হ");

        // hasanta is on 'g' for Jatiya
        assert!(layout.has_key('g' as u32));
        let mapping = layout.key_map.get(&('g' as u32)).unwrap();
        assert_eq!(mapping.output, "\u{09CD}");
    }

    #[test]
    fn test_probhat_has_key_mappings() {
        let layout = load_layout(LayoutType::Probhat).unwrap();

        // Probhat: k -> ক, i -> ি
        assert!(layout.has_key('k' as u32));
        let mapping = layout.key_map.get(&('k' as u32)).unwrap();
        assert_eq!(mapping.output, "ক");

        assert!(layout.has_key('i' as u32));
        let mapping = layout.key_map.get(&('i' as u32)).unwrap();
        assert_eq!(mapping.output, "ি");

        // hasanta is on '/' for Probhat
        assert!(layout.has_key('/' as u32));
        let mapping = layout.key_map.get(&('/' as u32)).unwrap();
        assert_eq!(mapping.output, "\u{09CD}");
    }

    #[test]
    fn test_unijoy_has_key_mappings() {
        let layout = load_layout(LayoutType::Unijoy).unwrap();

        // UniJoy: j -> ক, i -> হ, g -> hasanta
        assert!(layout.has_key('j' as u32));
        let mapping = layout.key_map.get(&('j' as u32)).unwrap();
        assert_eq!(mapping.output, "ক");

        assert!(layout.has_key('i' as u32));
        let mapping = layout.key_map.get(&('i' as u32)).unwrap();
        assert_eq!(mapping.output, "হ");

        assert!(layout.has_key('g' as u32));
        let mapping = layout.key_map.get(&('g' as u32)).unwrap();
        assert_eq!(mapping.output, "\u{09CD}");
    }

    #[test]
    fn test_parse_key_string() {
        assert_eq!(parse_key_string("a"), Some('a' as u32));
        assert_eq!(parse_key_string("z"), Some('z' as u32));
        assert_eq!(parse_key_string("0"), Some('0' as u32));
        assert_eq!(parse_key_string("ab"), None); // Multi-char
        assert_eq!(parse_key_string(""), None); // Empty
    }

    #[test]
    fn test_load_all_layouts() {
        let layouts = load_all_layouts();
        assert!(layouts.contains_key(&LayoutType::Phonetic));
        assert!(layouts.contains_key(&LayoutType::English));
        assert!(layouts.contains_key(&LayoutType::Jatiya));
        assert!(layouts.contains_key(&LayoutType::Probhat));
        assert!(layouts.contains_key(&LayoutType::Unijoy));
    }

    #[test]
    fn test_all_layouts_have_consistent_structure() {
        let layout_types = [
            LayoutType::Phonetic,
            LayoutType::English,
            LayoutType::Jatiya,
            LayoutType::Probhat,
            LayoutType::Unijoy,
        ];

        for layout_type in &layout_types {
            let layout = load_layout(*layout_type).unwrap();
            assert!(layout.key_count() > 0, "Layout {:?} has no keys", layout_type);
            assert!(!layout.name.is_empty(), "Layout {:?} has no name", layout_type);
        }
    }

    #[test]
    fn test_bengali_layouts_have_hasanta() {
        // Phonetic is Roman transliteration, no hasanta key - only fixed layouts
        let layout_types = [
            LayoutType::Jatiya,
            LayoutType::Probhat,
            LayoutType::Unijoy,
        ];

        for layout_type in &layout_types {
            let layout = load_layout(*layout_type).unwrap();
            let hasanta_key = match layout_type {
                LayoutType::Jatiya => 'g',
                LayoutType::Probhat => '/',
                LayoutType::Unijoy => 'g',
                _ => '\\',
            };
            assert!(layout.has_key(hasanta_key as u32), "Layout {:?} missing hasanta key '{}'", layout_type, hasanta_key);
            let mapping = layout.key_map.get(&(hasanta_key as u32)).unwrap();
            assert_eq!(mapping.output, "\u{09CD}", "Layout {:?} hasanta mapping incorrect", layout_type);
        }
    }

    #[test]
    fn test_bengali_layouts_have_digits() {
        let layout_types = [
            LayoutType::Phonetic,
            LayoutType::Jatiya,
            LayoutType::Probhat,
            LayoutType::Unijoy,
        ];

        for layout_type in &layout_types {
            let layout = load_layout(*layout_type).unwrap();
            // Check that digit keys exist
            for digit in '0'..='9' {
                assert!(layout.has_key(digit as u32), "Layout {:?} missing digit {}", layout_type, digit);
            }
        }
    }
}

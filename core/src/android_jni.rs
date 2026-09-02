use jni::objects::{JClass, JString, JObject};
use jni::sys::{jlong, jstring, jobjectArray};
use jni::JNIEnv;
use std::collections::HashMap;

use crate::engine::BengaliEngine;
use crate::types::{KeyEvent, OutputAction, LayoutType, KeyMapping};
use crate::types::CandidateWord;

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeInit(
    env: JNIEnv,
    _class: JClass,
) -> jlong {
    let engine = Box::new(BengaliEngine::new(LayoutType::Phonetic));
    Box::into_raw(engine) as jlong
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeProcessKey(
    env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    key_code: i32,
    is_shift: bool,
    _is_caps: bool,
) -> jstring {
    if ptr == 0 {
        let output = env.new_string("").unwrap();
        return output.into_raw();
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    
    // Handle shift/caps for this key press
    let was_shift = engine.get_state().shift_state;
    if is_shift {
        engine.get_state_mut().shift_state = crate::types::ShiftState::Shift;
    } else if _is_caps {
        engine.get_state_mut().shift_state = crate::types::ShiftState::CapsLock;
    }
    let key_event = KeyEvent::down(key_code as u32, key_code as u32);
    let _ = engine.process_key(key_event);
    // Reset shift if it was temporary
    if is_shift && was_shift == crate::types::ShiftState::None {
        engine.get_state_mut().shift_state = crate::types::ShiftState::None;
    }
    
    // Return full composition text (NFC normalized) - fixes typing bug where delta was returned
    let result = engine.get_text();
    
    let output = env.new_string(&result).unwrap();
    output.into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeProcessChar(
    env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    unicode: i32,
) -> jstring {
    if ptr == 0 {
        let output = env.new_string("").unwrap();
        return output.into_raw();
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    if let Some(ch) = char::from_u32(unicode as u32) {
        let _ = engine.process_char(ch);
    }
    let result = engine.get_text();
    let output = env.new_string(&result).unwrap();
    output.into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeApplySuggestion(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    suggestion: JString,
) -> jstring {
    if ptr == 0 {
        let output = env.new_string("").unwrap();
        return output.into_raw();
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    let sug: String = env.get_string(&suggestion).unwrap().into();
    let _ = engine.apply_suggestion(&sug);
    let result = engine.get_text();
    let output = env.new_string(&result).unwrap();
    output.into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeGetSuggestions(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jobjectArray {
    if ptr == 0 {
        let empty_array = env.new_object_array(0, "java/lang/String", JObject::null()).unwrap();
        return empty_array.into_raw();
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    // Use current_word (partial prefix) for suggestions, not full text
    let current = engine.current_word();
    let suggestions: Vec<CandidateWord> = engine.get_suggestions(&current);
    let mut string_array = env.new_object_array(
        suggestions.len() as i32,
        "java/lang/String",
        JObject::null(),
    ).unwrap();
    for (i, s) in suggestions.iter().enumerate() {
        let jstr = env.new_string(&s.word).unwrap();
        env.set_object_array_element(&string_array, i as i32, JObject::from(jstr)).unwrap();
    }
    string_array.into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeIsJoinMode(
    env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> bool {
    if ptr == 0 {
        return false;
    }
    let engine = unsafe { &*(ptr as *mut BengaliEngine) };
    engine.is_join_mode()
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeGetJoinSuggestions(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jobjectArray {
    if ptr == 0 {
        let empty_array = env.new_object_array(0, "java/lang/String", JObject::null()).unwrap();
        return empty_array.into_raw();
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    let suggestions = engine.get_join_suggestions();
    let mut string_array = env.new_object_array(
        suggestions.len() as i32,
        "java/lang/String",
        JObject::null(),
    ).unwrap();
    for (i, s) in suggestions.iter().enumerate() {
        let jstr = env.new_string(s).unwrap();
        env.set_object_array_element(&string_array, i as i32, JObject::from(jstr)).unwrap();
    }
    string_array.into_raw()
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeSwitchLayout(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
    layout_name: JString,
) {
    if ptr == 0 {
        return;
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    let name: String = env.get_string(&layout_name).unwrap().into();
    let layout_type = match name.to_lowercase().as_str() {
        "phonetic" => LayoutType::Phonetic,
        "jatiya" => LayoutType::Jatiya,
        "probhat" => LayoutType::Probhat,
        "unijoy" => LayoutType::Unijoy,
        "english" => LayoutType::English,
        _ => LayoutType::Phonetic,
    };
    engine.set_layout(layout_type);
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) {
    if ptr != 0 {
        unsafe {
            drop(Box::from_raw(ptr as *mut BengaliEngine));
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_com_keybroad_bridge_KeyboardEngine_nativeGetLayout(
    mut env: JNIEnv,
    _class: JClass,
    ptr: jlong,
) -> jstring {
    if ptr == 0 {
        let output = env.new_string("[]").unwrap();
        return output.into_raw();
    }
    let engine = unsafe { &mut *(ptr as *mut BengaliEngine) };
    let layout = engine.get_layout(LayoutType::Phonetic);

    let mut json = String::from("[");
    let mut first = true;

    // Standard QWERTY row order for UI display
    let qwerty_order = "qwertyuiopasdfghjklzxcvbnm0123456789".chars();

    if let Some(layout) = layout {
        for key_char in qwerty_order {
            let unicode = key_char as u32;
            if let Some(mapping) = layout.key_map.get(&unicode) {
                if !first {
                    json.push(',');
                }
                first = false;
                // Escape special characters in output
                let output_escaped = mapping.output
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                let shift_escaped = mapping.shift_output
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                let display_escaped = mapping.display
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                json.push_str(&format!(
                    r#"{{"key":"{}","output":"{}","shift":"{}","display":"{}"}}"#,
                    key_char, output_escaped, shift_escaped, display_escaped
                ));
            }
        }
    }

    json.push(']');

    let output = env.new_string(&json).unwrap();
    output.into_raw()
}

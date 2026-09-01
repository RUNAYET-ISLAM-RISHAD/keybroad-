/// Bengali keyboard engine type definitions.
///
/// All types are `repr(C)` where needed for FFI compatibility.
/// The engine follows a pure functional design: same input always produces same output.

use std::collections::HashMap;
use serde::Deserialize;

// === Layout Types ===

/// A single key mapping in a layout.
#[derive(Debug, Clone, Deserialize)]
pub struct KeyMapping {
    /// The output character(s) when this key is pressed normally
    pub output: String,
    /// The output character(s) when this key is pressed with shift
    pub shift_output: String,
    /// Display label for the key (for UI rendering)
    #[serde(default)]
    pub display: String,
}

/// Special key definition.
#[derive(Debug, Clone, Deserialize)]
pub struct SpecialKeyDef {
    /// Platform key code for this special key
    pub key_code: u32,
    /// Action identifier ("backspace", "enter", "shift", "space")
    pub action: String,
}

/// A loaded keyboard layout with all key mappings.
#[derive(Debug, Clone)]
pub struct Layout {
    /// Layout identifier
    pub id: LayoutType,
    /// Human-readable name
    pub name: String,
    /// Description of the layout
    pub description: String,
    /// Mapping from character codepoint to key mapping
    pub key_map: HashMap<u32, KeyMapping>,
    /// Special key definitions
    pub special_keys: HashMap<String, SpecialKeyDef>,
}

impl Layout {
    /// Look up the output for a given character codepoint.
    /// Returns None if the character is not mapped in this layout.
    pub fn lookup(&self, unicode: u32, shift_active: bool) -> Option<&str> {
        self.key_map.get(&unicode).map(|mapping| {
            if shift_active {
                mapping.shift_output.as_str()
            } else {
                mapping.output.as_str()
            }
        })
    }

    /// Check if a character is mapped in this layout.
    pub fn has_key(&self, unicode: u32) -> bool {
        self.key_map.contains_key(&unicode)
    }

    /// Get the number of key mappings in this layout.
    pub fn key_count(&self) -> usize {
        self.key_map.len()
    }
}

/// Serde-compatible representation of the JSON layout file.
#[derive(Debug, Deserialize)]
pub struct LayoutJson {
    pub layout_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: u32,
    pub keys: HashMap<String, KeyMapping>,
    #[serde(default)]
    pub special_keys: HashMap<String, SpecialKeyDef>,
}

// === Keyboard Layout Enum ===

/// Supported keyboard layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum LayoutType {
    Phonetic = 0,
    Jatiya = 1,
    Probhat = 2,
    Unijoy = 3,
    English = 4,
}

impl LayoutType {
    /// Parse layout from a string identifier.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "phonetic" => Some(LayoutType::Phonetic),
            "jatiya" => Some(LayoutType::Jatiya),
            "probhat" => Some(LayoutType::Probhat),
            "unijoy" => Some(LayoutType::Unijoy),
            "english" => Some(LayoutType::English),
            _ => None,
        }
    }

    /// Get the string identifier for this layout.
    pub fn as_str(&self) -> &'static str {
        match self {
            LayoutType::Phonetic => "phonetic",
            LayoutType::Jatiya => "jatiya",
            LayoutType::Probhat => "probhat",
            LayoutType::Unijoy => "unijoy",
            LayoutType::English => "english",
        }
    }

    /// Get the JSON filename for this layout.
    pub fn filename(&self) -> &'static str {
        match self {
            LayoutType::Phonetic => "phonetic.json",
            LayoutType::Jatiya => "jatiya.json",
            LayoutType::Probhat => "probhat.json",
            LayoutType::Unijoy => "unijoy.json",
            LayoutType::English => "english.json",
        }
    }
}

// === Input Event Types ===

/// Represents a single key event from the platform.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// Platform key code (e.g., Android KeyEvent.KEYCODE_A = 29)
    pub key_code: u32,
    /// Unicode codepoint of the character
    pub unicode: u32,
    /// Whether the key is pressed down (true) or released (false)
    pub is_down: bool,
    /// Timestamp in milliseconds (for timing analysis)
    pub timestamp_ms: u64,
}

impl KeyEvent {
    /// Create a new key event.
    pub fn new(key_code: u32, unicode: u32, is_down: bool, timestamp_ms: u64) -> Self {
        Self {
            key_code,
            unicode,
            is_down,
            timestamp_ms,
        }
    }

    /// Create a key-down event with zero timestamp (for testing).
    pub fn down(key_code: u32, unicode: u32) -> Self {
        Self::new(key_code, unicode, true, 0)
    }

    /// Create a key-down event from a character (for testing convenience).
    pub fn from_char(ch: char) -> Self {
        Self::new(ch as u32, ch as u32, true, 0)
    }
}

// === Output Types ===

/// A glyph in the composition buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// Unicode codepoint
    pub unicode: u32,
    /// Whether this is a combining character (vowel sign, etc.)
    pub is_combining: bool,
    /// Whether this is part of a conjunct character
    pub is_conjunct: bool,
    /// Index into the conjunct lookup table (0 if not a conjunct)
    pub conjunct_id: u16,
}

impl Glyph {
    /// Create a simple glyph (non-combining, non-conjunct).
    pub fn simple(unicode: u32) -> Self {
        Self {
            unicode,
            is_combining: false,
            is_conjunct: false,
            conjunct_id: 0,
        }
    }

    /// Create a combining glyph (e.g., vowel sign).
    pub fn combining(unicode: u32) -> Self {
        Self {
            unicode,
            is_combining: true,
            is_conjunct: false,
            conjunct_id: 0,
        }
    }
}

/// A candidate word for suggestion bar.
#[derive(Debug, Clone, PartialEq)]
pub struct CandidateWord {
    /// The suggested word (UTF-8 bytes, fixed max length)
    pub word: String,
    /// Confidence score (0.0 - 1.0)
    pub score: f32,
    /// Source of this candidate
    pub source: WordSource,
}

/// Where a candidate word came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordSource {
    /// From the built-in dictionary
    Dictionary,
    /// From user typing history
    UserHistory,
    /// From AI prediction engine
    AiPrediction,
}

/// Actions the platform should execute after processing a key.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputAction {
    /// Append text to the input field
    CommitText(String),
    /// Update the composition buffer (for inline editing / underline preview)
    UpdateComposition(Vec<Glyph>),
    /// Delete N characters before the cursor
    Backspace(u32),
    /// Move the cursor by N positions (positive = forward, negative = backward)
    MoveCursor(i32),
    /// Update the candidate suggestion bar
    UpdateCandidates(Vec<CandidateWord>),
    /// No action needed
    Nothing,
}

// === State Types ===

/// Shift key state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShiftState {
    /// No shift active
    None,
    /// Next character will be shifted (single press)
    Shift,
    /// Caps lock is active (double press or toggle)
    CapsLock,
}

/// Keyboard state maintained between keystrokes.
#[derive(Debug, Clone)]
pub struct EngineState {
    /// Current active layout
    pub layout: LayoutType,
    /// Cursor position in the input field
    pub cursor_position: u32,
    /// Current composition buffer (characters being composed)
    pub composition_buffer: Vec<Glyph>,
    /// Current shift state
    pub shift_state: ShiftState,
    /// Whether caps lock is active
    pub caps_lock: bool,
    /// Whether incognito mode is active (no learning, no prediction)
    pub incognito_mode: bool,
    /// Current candidate suggestions
    pub candidates: Vec<CandidateWord>,
    /// Pending character for digraph detection.
    /// When a character is typed, it's held here until the next key is pressed.
    /// If the next key forms a digraph with this character, the digraph is output.
    /// If not, this character is flushed to the output.
    pub pending_char: Option<char>,
    /// Whether we're waiting for a consonant after hasanta.
    /// When hasanta is typed after a consonant, this flag is set.
    /// The next consonant will complete the conjunct sequence.
    pub hasanta_pending: bool,
    /// The consonant before the pending hasanta (for conjunct formation).
    pub hasanta_base_consonant: Option<char>,
    /// The word currently being typed (accumulates characters until word boundary)
    pub current_word: String,
    /// History of previously completed words (capped at 20)
    pub history: Vec<String>,
    /// The most recently committed word (for n-gram context lookup)
    pub last_committed_word: Option<String>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            layout: LayoutType::Phonetic,
            cursor_position: 0,
            composition_buffer: Vec::with_capacity(64),
            shift_state: ShiftState::None,
            caps_lock: false,
            incognito_mode: false,
            candidates: Vec::new(),
            pending_char: None,
            hasanta_pending: false,
            hasanta_base_consonant: None,
            current_word: String::new(),
            history: Vec::new(),
            last_committed_word: None,
        }
    }
}

// === Error Types ===

/// Errors that can occur during engine operations.
#[derive(Debug, Clone, PartialEq)]
pub enum EngineError {
    /// Unknown or unsupported layout
    UnknownLayout(String),
    /// Invalid key code for current layout
    InvalidKeyCode(u32),
    /// Layout file not found or invalid
    LayoutLoadError(String),
    /// Engine is in an inconsistent state
    InternalError(String),
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::UnknownLayout(s) => write!(f, "Unknown layout: {}", s),
            EngineError::InvalidKeyCode(code) => write!(f, "Invalid key code: {}", code),
            EngineError::LayoutLoadError(msg) => write!(f, "Layout load error: {}", msg),
            EngineError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for EngineError {}

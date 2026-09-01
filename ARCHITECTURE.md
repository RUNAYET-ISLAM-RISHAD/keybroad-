# ARCHITECTURE.md — Modern Bengali Keyboard

> Technical Design Document v1.0
> Date: 2026-08-31
> Status: Approved for Implementation

---

## Table of Contents

1. [System Architecture](#1-system-architecture)
2. [Core Engine Design](#2-core-engine-design)
3. [Data Layer Design](#3-data-layer-design)
4. [Presentation Layer Design](#4-presentation-layer-design)
5. [On-Device AI Strategy](#5-on-device-ai-strategy)
6. [Security & Privacy Design](#6-security--privacy-design)
7. [Backend & Cloud Sync Design](#7-backend--cloud-sync-design)
8. [Testing Strategy](#8-testing-strategy)
9. [Performance Budgets](#9-performance-budgets)
10. [Build & CI/CD](#10-build--cicd)
11. [Risk Register](#11-risk-register)

---

## 1. System Architecture

### 1.1 High-Level Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         ANDROID                                  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                 PRESENTATION LAYER                        │   │
│  │  Jetpack Compose (Keyboard UI)                           │   │
│  │  KeyboardView (Custom Canvas rendering for critical path)│   │
│  │  StateViewModel ← Kotlin Flow ← Core Engine             │   │
│  └──────────────────────┬───────────────────────────────────┘   │
│                         │ JNI Bridge                             │
│  ┌──────────────────────▼───────────────────────────────────┐   │
│  │                    CORE LAYER                             │   │
│  │  libkeybroad_core.so (Rust, compiled to ARM64)           │   │
│  │  ┌─────────────┐ ┌──────────────┐ ┌──────────────────┐  │   │
│  │  │ TypeEngine  │ │ LayoutMapper │ │ PredictionEngine │  │   │
│  │  └─────────────┘ └──────────────┘ └──────────────────┘  │   │
│  └──────────────────────┬───────────────────────────────────┘   │
│                         │ FFI                                    │
│  ┌──────────────────────▼───────────────────────────────────┐   │
│  │                    DATA LAYER                             │   │
│  │  SQLCipher (encrypted SQLite)                            │   │
│  │  CRDT Sync Engine (custom Rust crate)                    │   │
│  │  EncryptedSharedPreferences (keychain data)              │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  SERVICE LAYER                            │   │
│  │  Companion App Process (Settings, Theme Store)           │   │
│  │  ← NO INTERNET in keyboard process →                     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AI LAYER                               │   │
│  │  TFLite Runtime (on-device inference)                    │   │
│  │  Federated Learning Client (weights only)                │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                         iOS                                      │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                 PRESENTATION LAYER                        │   │
│  │  SwiftUI (Keyboard Extension UI)                         │   │
│  │  CustomCALayer (critical rendering path)                 │   │
│  │  KeyboardViewModel ← Combine ← Core Engine              │   │
│  └──────────────────────┬───────────────────────────────────┘   │
│                         │ FFI Bridge                             │
│  ┌──────────────────────▼───────────────────────────────────┐   │
│  │                    CORE LAYER                             │   │
│  │  libkeybroad_core.a (Rust, static library)               │   │
│  │  (Same Rust codebase as Android)                         │   │
│  └──────────────────────┬───────────────────────────────────┘   │
│                         │ FFI                                    │
│  ┌──────────────────────▼───────────────────────────────────┐   │
│  │                    DATA LAYER                             │   │
│  │  SQLCipher via GRDB.swift                                │   │
│  │  Keychain + Secure Enclave                               │   │
│  │  CRDT Sync Engine (same Rust crate)                      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    AI LAYER                               │   │
│  │  CoreML Runtime (on-device inference)                    │   │
│  │  Federated Learning Client                               │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    CLOUD / BACKEND                               │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Node.js + Fastify Server                                │   │
│  │  WebSocket (CRDT sync propagation)                       │   │
│  │  REST API (auth, billing, theme store)                   │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  PostgreSQL (user accounts, encrypted blobs)             │   │
│  │  Redis (session cache, rate limiting)                    │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  Federated Learning Aggregator                           │   │
│  │  (receives model weights, aggregates, redistributes)     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Communication Flow

```
User taps key
    │
    ▼
[Platform InputEvent] ──→ JNI/FFI ──→ [Rust Core Engine]
                                            │
                                            ├──→ process_key() → EngineState
                                            │
                                            ├──→ Trie lookup → candidate words
                                            │
                                            └──→ OutputAction[] ──→ JNI/FFI ──→ [Platform]
                                                                      │
                                                                      ▼
                                                            [Compose/SwiftUI renders]
                                                                      │
                                                                      ▼
                                                            User sees character (<8ms)
```

### 1.3 Layer Isolation Rules

| Rule | Enforcement |
|---|---|
| Core Layer has ZERO imports from Presentation, Data, or Service | Rust crate dependencies enforced at compile time |
| Presentation Layer cannot call Core Layer directly | Must go through JNI/FFI bridge |
| Data Layer exposes only persistence APIs | No business logic in data layer |
| Service Layer runs in separate process (Android) | OS-enforced process isolation |
| Keyboard process has no INTERNET permission | AndroidManifest.xml enforcement |

---

## 2. Core Engine Design

### 2.1 Technology Decision: **Rust**

**Chosen:** Rust (compiled to native libraries for Android and iOS)

**Alternatives Considered:**
- Kotlin/Native — rejected: KMM ecosystem immature for real-time systems; no memory control
- C++ — rejected: memory safety concerns; Rust provides same performance with safety guarantees
- Separate Kotlin + Swift — rejected: code duplication, divergent behavior risk

**Justification:**

| Criterion | Rust | Kotlin Native | C++ |
|---|---|---|---|
| Performance | Excellent (LLVM backend) | Good (but boxing overhead) | Excellent |
| Memory Safety | Yes (ownership system) | Yes (GC) | No (manual) |
| Zero-Cost Abstractions | Yes | Partial | Yes |
| Garbage Collector | None | Yes (kotlin.native) | None |
| Cross-Platform | Android + iOS + WASM | Android + iOS | Android + iOS |
| FFI Quality | Excellent (cbindgen) | Kotlin/Native interop | Native |
| WebAssembly Target | Yes (wasm32) | Limited | Limited |
| Learning Curve | Steep | Moderate | Moderate |

**Risk:** Rust has a steep learning curve. Mitigated by: the engine is a self-contained module; once built, it rarely changes; and the performance benefits are definitive for a real-time keyboard system.

### 2.2 Public API

```rust
// core/src/lib.rs

// === Engine State ===
#[repr(C)]
pub struct EngineState {
    pub layout: LayoutType,
    pub cursor_position: u32,
    pub composition_buffer: CompositionBuffer,
    pub shift_state: bool,
    pub caps_lock: bool,
    pub incognito_mode: bool,
    pub candidates: Vec<CandidateWord>,
}

#[repr(C)]
pub enum LayoutType {
    Phonetic,
    Jatiya,
    Probhat,
    Unijoy,
    English,
}

#[repr(C)]
pub struct CompositionBuffer {
    glyphs: Vec<Glyph>,        // Pre-allocated, arena-backed
    len: u32,
}

#[repr(C)]
pub struct Glyph {
    pub unicode: u32,
    pub is_combining: bool,
    pub is_conjunct: bool,
    pub conjunct_id: u16,       // Index into conjunct lookup table
}

#[repr(C)]
pub struct CandidateWord {
    pub word: [u8; 128],        // Fixed-size, no allocation
    pub score: f32,
    pub source: WordSource,
}

#[repr(C)]
pub enum WordSource {
    Dictionary,
    UserHistory,
    AiPrediction,
}

// === Core Engine API ===
#[repr(C)]
pub struct BengaliEngine {
    trie: TrieNode,
    layouts: LayoutStore,
    state: EngineState,
    arena: ArenaAllocator,
}

impl BengaliEngine {
    /// Create a new engine instance. Loads layouts from JSON, initializes trie.
    pub fn new(layout_dir: &str) -> Result<Self, EngineError>;

    /// Process a single key event. Pure function: same input → same output.
    /// Returns actions the platform should execute.
    pub fn process_key(
        &mut self,
        event: KeyEvent,
    ) -> Result<Vec<OutputAction>, EngineError>;

    /// Process a gesture (swipe typing). Returns candidate words.
    pub fn process_gesture(
        &mut self,
        points: &[Point],
    ) -> Result<Vec<CandidateWord>, EngineError>;

    /// Select a candidate word from suggestions.
    pub fn select_candidate(&mut self, index: u32) -> Result<OutputAction, EngineError>;

    /// Get current engine state (for serialization/sync).
    pub fn get_state(&self) -> &EngineState;

    /// Reset engine state (for new input field).
    pub fn reset(&mut self);

    /// Enable/disable incognito mode.
    pub fn set_incognito(&mut self, enabled: bool);

    /// Add a word to user dictionary.
    pub fn add_to_dictionary(&mut self, word: &str) -> Result<(), EngineError>;

    /// Get prediction candidates for current composition.
    pub fn get_candidates(&self) -> &[CandidateWord];
}

#[repr(C)]
pub struct KeyEvent {
    pub key_code: u32,
    pub unicode: u32,
    pub is_down: bool,
    pub timestamp_ms: u64,
}

#[repr(C)]
pub enum OutputAction {
    /// Append text to the input field
    CommitText(String),
    /// Update the composition buffer (underlines, preview)
    UpdateComposition(CompositionBuffer),
    /// Delete N characters before cursor
    Backspace(u32),
    /// Move cursor
    MoveCursor(i32),
    /// Show/hide candidate bar
    UpdateCandidates(Vec<CandidateWord>),
    /// No action needed
    Nothing,
}
```

### 2.3 Internal Design

#### 2.3.1 Composition Buffer (Arena Allocator)

```
Arena Layout (pre-allocated at engine init):
┌──────────────────────────────────────────┐
│  Glyph Pool (4096 glyphs = ~48KB)        │
│  ┌───┬───┬───┬───┬───┬───┬───┬───┐     │
│  │ G │ G │ G │ G │ G │ G │ G │...│     │
│  └───┴───┴───┴───┴───┴───┴───┴───┘     │
│                                          │
│  String Pool (64KB)                      │
│  ┌──────────────────────────────────┐   │
│  │ Pre-allocated string storage     │   │
│  └──────────────────────────────────┘   │
│                                          │
│  Candidate Buffer (20 slots × 128B)     │
│  ┌────────┬────────┬────────┬─────┐    │
│  │ Cand 0 │ Cand 1 │ Cand 2 │ ... │    │
│  └────────┴────────┴────────┴─────┘    │
└──────────────────────────────────────────┘

Total: ~130KB pre-allocated at engine creation.
Per-keystroke allocation: 0 bytes (all from arena).
```

#### 2.3.2 Trie Structure (Bengali Conjunct Optimized)

```
Trie Node Layout:
┌─────────────────────────────────────────────┐
│ TrieNode {                                  │
│   children: [Option<Box<TrieNode>>; 256],   │
│   is_word: bool,                            │
│   word_id: u32,                             │
│   frequency: u32,                           │
│   is_conjunct_boundary: bool,               │
│ }                                           │
└─────────────────────────────────────────────┘

Bengali Conjunct Handling:
- Conjuncts (ক্ষ, জ্ঞ, চ্ছ) are pre-computed lookup tables
- On input: key sequence → check conjunct table → if match, emit conjunct glyph
- No backtracking needed — single-pass processing

Example flow for "ক্ষ":
  ক → engine checks if next input could form conjunct
  ি → engine combines: ক + ি → কি (direct mapping)
  
Example flow for "ক্ষ":
  ক → engine enters "possible conjunct" state
  ং → engine sees: ক + ং = ক্ং (conjunct found via lookup)
  
  Conjunct Table (pre-computed):
  ┌─────────┬──────────┬──────────┐
  │ Base    │ Modifier │ Result   │
  ├─────────┼──────────┼──────────┤
  │ ক       │ ং        │ ক্ং      │
  │ ক       │ ষ        │ ক্ষ      │
  │ জ       │ ঞ        │ জ্ঞ      │
  │ ...     │ ...      │ ...      │
  └─────────┴──────────┴──────────┘
```

#### 2.3.3 State Machine

```
States:
  IDLE → composing → SELECTING_CANDIDATE → committed
                    ↑                        │
                    └────────────────────────┘

Input handling:
  Key Down:
    if IDLE:
      start composition, enter COMPOSING
    if COMPOSING:
      extend composition buffer, update candidates
    if SELECTING_CANDIDATE:
      move selection highlight

  Key Up:
    if COMPOSING and no more pending keys:
      commit text, return to IDLE

  Candidate Selected:
    commit selected word, return to IDLE

  Backspace:
    if COMPOSING:
      remove last glyph from buffer
      if buffer empty: return to IDLE
```

### 2.4 Layout System

Layouts are defined as JSON files loaded at runtime:

```json
{
  "layout_id": "phonetic",
  "name": "Phonetic",
  "version": 1,
  "keys": [
    {
      "code": 97,
      "display": "a",
      "output": "া",
      "shift_output": "া",
      "long_press": ["আ", "া"],
      "row": 0,
      "col": 0
    },
    {
      "code": 107,
      "display": "k",
      "output": "ক",
      "shift_output": "খ",
      "long_press": ["গ", "ঘ", "ঙ"],
      "row": 0,
      "col": 4
    }
  ],
  "conjunct_rules": {
    "inherent_vowel_replacement": true,
    "virama_char": "্"
  }
}
```

**5 layout files:**
- `phonetic.json` — English-to-Bangla transliteration
- `jatiya.json` — National layout
- `probhat.json` — Probhat layout
- `unijoy.json` — Unijoy layout
- `english.json` — Standard QWERTY

---

## 3. Data Layer Design

### 3.1 Technology Decision: **SQLCipher** (both platforms)

**Chosen:** SQLCipher (encrypted SQLite) for local storage

**Alternatives Considered:**
- Realm — rejected: larger binary size, less ecosystem, no FTS5 for dictionary search
- Core Data (iOS only) — rejected: platform-specific, no Android equivalent
- DataStore (Android only) — rejected: platform-specific

**Justification:**
- AES-256-CBC encryption built-in
- Full SQLite compatibility (FTS5 for dictionary full-text search)
- Single database format works across platforms
- Small binary footprint (~500KB)
- Battle-tested in production apps

### 3.2 Schema Design

```sql
-- ==========================================
-- DICTIONARY TABLE
-- ==========================================
CREATE TABLE dictionary (
    id INTEGER PRIMARY KEY,
    word TEXT NOT NULL,
    frequency INTEGER DEFAULT 0,
    language TEXT DEFAULT 'bn',
    is_user_added INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX idx_dictionary_word ON dictionary(word);
CREATE INDEX idx_dictionary_frequency ON dictionary(frequency DESC);

-- Full-text search for dictionary lookup
CREATE VIRTUAL TABLE dictionary_fts USING fts5(
    word,
    content='dictionary',
    content_rowid='id'
);

-- ==========================================
-- USER DICTIONARY (custom words)
-- ==========================================
CREATE TABLE user_dictionary (
    id INTEGER PRIMARY KEY,
    word TEXT NOT NULL UNIQUE,
    usage_count INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL
);

-- ==========================================
-- TYPING HISTORY (for N-gram predictions)
-- ==========================================
CREATE TABLE typing_history (
    id INTEGER PRIMARY KEY,
    bigram TEXT NOT NULL,           -- "word1 word2"
    frequency INTEGER DEFAULT 1,
    last_used INTEGER NOT NULL,
    -- CRDT fields
    crdt_id TEXT NOT NULL UNIQUE,   -- UUID
    crdt_timestamp INTEGER NOT NULL, -- Lamport timestamp
    crdt_deleted INTEGER DEFAULT 0
);

CREATE INDEX idx_typing_bigram ON typing_history(bigram);

-- ==========================================
-- THEMES
-- ==========================================
CREATE TABLE themes (
    id TEXT PRIMARY KEY,            -- UUID
    name TEXT NOT NULL,
    is_premium INTEGER DEFAULT 0,
    creator_id TEXT,
    data_json TEXT NOT NULL,        -- Full theme config
    preview_image BLOB,
    version INTEGER DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- ==========================================
-- CLIPBOARD HISTORY
-- ==========================================
CREATE TABLE clipboard_history (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    content_type TEXT DEFAULT 'text', -- text, image_uri
    pinned INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    expires_at INTEGER               -- Auto-delete after N days
);

-- ==========================================
-- USER SETTINGS
-- ==========================================
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    value_type TEXT DEFAULT 'string' -- string, int, bool, json
);

-- Default settings
INSERT INTO settings VALUES ('active_layout', 'phonetic', 'string');
INSERT INTO settings VALUES ('theme_id', 'default_dark', 'string');
INSERT INTO settings VALUES ('sound_enabled', 'true', 'bool');
INSERT INTO settings VALUES ('vibration_enabled', 'true', 'bool');
INSERT INTO settings VALUES ('auto_correct', 'true', 'bool');
INSERT INTO settings VALUES ('next_word_suggestion', 'true', 'bool');
INSERT INTO settings VALUES ('clipboard_history_enabled', 'true', 'bool');
INSERT INTO settings VALUES ('incognito_mode', 'false', 'bool');

-- ==========================================
-- SYNC STATE (for CRDT tracking)
-- ==========================================
CREATE TABLE sync_metadata (
    table_name TEXT NOT NULL,
    last_synced_timestamp INTEGER DEFAULT 0,
    device_id TEXT NOT NULL,
    sync_status TEXT DEFAULT 'pending', -- pending, synced, conflict
    PRIMARY KEY (table_name, device_id)
);

-- ==========================================
-- FEDERATED LEARNING
-- ==========================================
CREATE TABLE fl_model_state (
    id INTEGER PRIMARY KEY,
    model_version INTEGER NOT NULL,
    local_updates_count INTEGER DEFAULT 0,
    last_aggregated_at INTEGER,
    pending_weights BLOB            -- Encrypted model weight deltas
);
```

### 3.3 CRDT Sync Implementation

**Approach:** Custom CRDT implementation in Rust using RGA (Replicated Growable Array) for text sequences and LWW-Register (Last-Writer-Wins) for scalar values.

**Library:** Custom implementation (no suitable Rust CRDT library exists that meets our requirements).

```rust
// crdt/src/lib.rs

/// Lamport timestamp for causal ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LamportTimestamp {
    pub counter: u64,
    pub node_id: u32,  // Device-specific ID
}

/// CRDT-enabled dictionary entry
#[derive(Debug, Clone)]
pub struct CrdtEntry<T> {
    pub id: uuid::Uuid,
    pub value: T,
    pub timestamp: LamportTimestamp,
    pub deleted: bool,
}

/// RGA (Replicated Growable Array) for ordered sequences
pub struct Rga<T> {
    entries: Vec<CrdtEntry<T>>,
    node_id: u32,
    clock: LamportTimestamp,
}

impl<T: Clone + PartialEq> Rga<T> {
    pub fn insert(&mut self, index: usize, value: T) -> CrdtEntry<T>;
    pub fn delete(&mut self, id: uuid::Uuid);
    pub fn merge(&mut self, other: Rga<T>);  // Automatic conflict resolution
    pub fn to_list(&self) -> Vec<&T>;        // Final merged view
}

/// LWW-Register for single-value fields (settings, theme selection)
pub struct LwwRegister<T> {
    value: T,
    timestamp: LamportTimestamp,
}

impl<T: Clone> LwwRegister<T> {
    pub fn set(&mut self, value: T, timestamp: LamportTimestamp);
    pub fn merge(&mut self, other: LwwRegister<T>);  // Higher timestamp wins
    pub fn get(&self) -> &T;
}
```

**Sync Protocol:**

```
Device A (offline)     Cloud Server      Device B (offline)
      │                     │                     │
      │── add "রিকশা" ────→│                     │
      │   (local CRDT op)   │                     │
      │                     │── store blob ──────→│
      │                     │                     │── add "রিকশা" ──→│
      │                     │                     │   (local CRDT op) │
      │                     │←── store blob ─────│
      │                     │                     │
      │←── pull latest ─────│                     │
      │   merge (auto)      │                     │
      │   result: 1 entry   │                     │
      │                     │←── pull latest ─────│
      │                     │   merge (auto)      │
      │                     │   result: 1 entry   │
      │                     │                     │
      │  ✓ No duplicates    │  ✓ No data loss     │  ✓ No conflicts
```

**Data that syncs:**
- User dictionary words
- Typing history (bigrams for N-gram prediction)
- User settings
- Theme preferences

**Data that NEVER syncs:**
- Actual typed text content
- Passwords/OTP input
- Clipboard content (unless user explicitly enables)

### 3.4 Encryption at Rest

```
SQLCipher Configuration:
  cipher_page_size: 4096
  kdf_iter: 256000
  cipher_hmac_algorithm: HMAC_SHA512
  cipher_kdf_algorithm: PBKDF2_HMAC_SHA512
  
Encryption Key Storage:
  Android: Android Keystore → hardware-backed AES-256
  iOS: Keychain + Secure Enclave → hardware-backed AES-256
  
Key Derivation:
  User PIN/biometric → PBKDF2 → 256-bit key → SQLCipher
  
  [User enters PIN]
       │
       ▼
  [PBKDF2(password, salt, 256000 iterations)]
       │
       ▼
  [256-bit encryption key]
       │
       ▼
  [SQLCipher opens database with this key]
```

---

## 4. Presentation Layer Design

### 4.1 Technology Decision: **Jetpack Compose** (Android) + **SwiftUI** (iOS)

**Chosen:** Native declarative UI frameworks

**Alternatives Considered:**
- Flutter — rejected: adds 5-8ms rendering overhead on critical path; Dart VM overhead
- React Native — rejected: JavaScript bridge latency; not suitable for 60fps keyboard rendering
- XML Views (Android) / UIKit (iOS) — considered as fallback for critical rendering path

**Justification:**
- Modern declarative paradigm matches reactive state management
- Excellent performance for keyboard UI (with fallback to custom Canvas/CALayer for critical path)
- Strong tooling and community support
- Hot reload accelerates development

**Risk:** Compose/SwiftUI may not meet <8ms on low-end devices. **Mitigation:** Critical key rendering path uses custom `Canvas` (Android) / `CALayer` (iOS) — bypassing Compose/SwiftUI for the key press visual feedback only.

### 4.2 Android Architecture (Jetpack Compose)

```
┌─────────────────────────────────────────────────────────────┐
│                    KeyboardService                           │
│  (extends InputMethodService)                               │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ KeyboardComposeView                                   │   │
│  │ (ComposeView hosted in InputMethodService)            │   │
│  │                                                        │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ KeyboardScreen                                   │  │   │
│  │  │                                                  │  │   │
│  │  │  ┌──────────────┐  ┌──────────────────────┐   │  │   │
│  │  │  │ CandidateBar │  │ KeyGrid              │   │  │   │
│  │  │  │ (LazyRow)    │  │ (LazyGrid)           │   │  │   │
│  │  │  │              │  │                      │   │  │   │
│  │  │  │ word1│word2  │  │ ┌──┬──┬──┬──┬──┐   │   │  │   │
│  │  │  │      │word3  │  │ │ক │খ │গ │ঘ │ঙ │   │   │  │   │
│  │  │  └──────────────┘  │ ├──┼──┼──┼──┼──┤   │   │  │   │
│  │  │                     │ │চ │ছ │জ │ঝ │ঞ │   │   │  │   │
│  │  │  ┌──────────────┐  │ ├──┼──┼──┼──┼──┤   │   │  │   │
│  │  │  │ UtilityBar   │  │ │...               │   │   │  │   │
│  │  │  │ 🎤 │ ⚙️ │ 📋 │  │ └──┴──┴──┴──┴──┘   │   │  │   │
│  │  │  └──────────────┘  └──────────────────────┘   │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ KeyboardViewModel                                     │   │
│  │                                                        │   │
│  │  StateFlow<KeyboardUiState>                           │   │
│  │    ├── layout: LayoutType                             │   │
│  │    ├── candidates: List<CandidateWord>                │   │
│  │    ├── compositionText: String                        │   │
│  │    ├── shiftState: ShiftState                         │   │
│  │    └── theme: ThemeConfig                             │   │
│  │                                                        │   │
│  │  onKeyPress(KeyEvent) → calls native engine           │   │
│  │  onSelectCandidate(index)                              │   │
│  │  onLayoutChange(LayoutType)                            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ NativeEngineBridge (JNI)                              │   │
│  │                                                        │   │
│  │  external fun nativeProcessKey(keyCode: Int,          │   │
│  │                                  unicode: Int,          │   │
│  │                                  timestamp: Long):     │   │
│  │                                  Array<OutputAction>   │   │
│  │                                                        │   │
│  │  external fun nativeGetCandidates(): Array<Candidate>  │   │
│  │  external fun nativeSetLayout(layoutId: String)        │   │
│  │  external fun nativeReset()                            │   │
│  │  external fun nativeSetIncognito(enabled: Boolean)     │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**State Management:**

```kotlin
// KeyboardViewModel.kt
@HiltViewModel
class KeyboardViewModel @Inject constructor(
    private val engineBridge: NativeEngineBridge,
    private val settingsRepository: SettingsRepository,
    private val themeRepository: ThemeRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(KeyboardUiState())
    val uiState: StateFlow<KeyboardUiState> = _uiState.asStateFlow()

    fun onKeyPress(keyEvent: KeyEvent) {
        viewModelScope.launch(Dispatchers.Default) {
            val actions = engineBridge.processKey(keyEvent)
            actions.forEach { action ->
                when (action) {
                    is OutputAction.CommitText -> inputConnection.commitText(action.text)
                    is OutputAction.UpdateComposition -> updateComposition(action.buffer)
                    is OutputAction.Backspace -> inputConnection.deleteSurroundingText(action.count, 0)
                    is OutputAction.UpdateCandidates -> updateCandidates(action.candidates)
                    is OutputAction.Nothing -> { /* no-op */ }
                }
            }
        }
    }
}

data class KeyboardUiState(
    val layout: LayoutType = LayoutType.Phonetic,
    val candidates: List<CandidateWord> = emptyList(),
    val compositionText: String = "",
    val shiftState: ShiftState = ShiftState.None,
    val theme: ThemeConfig = ThemeConfig.DefaultDark,
    val isIncognito: Boolean = false,
)
```

**Critical Rendering Path (Custom Canvas):**

```kotlin
// KeyView.kt — Custom Canvas for <8ms key feedback
@Composable
fun KeyView(
    key: KeyDefinition,
    onPress: () -> Unit,
    onRelease: () -> Unit,
) {
    var isPressed by remember { mutableStateOf(false) }
    
    Canvas(
        modifier = Modifier
            .pointerInput(Unit) {
                detectTapGestures(
                    onPress = {
                        isPressed = true
                        onPress()
                        tryAwaitRelease()
                        isPressed = false
                        onRelease()
                    }
                )
            }
    ) {
        // Direct Canvas rendering — bypasses Compose layout
        // Target: <2ms for this draw call
        drawRoundRect(
            color = if (isPressed) key.pressedColor else key.normalColor,
            cornerRadius = CornerRadius(8.dp.toPx()),
        )
        drawText(
            textLayoutResult = key.textLayout,
            color = key.textColor,
        )
    }
}
```

### 4.3 iOS Architecture (SwiftUI)

```
┌─────────────────────────────────────────────────────────────┐
│                KeyboardViewController                        │
│  (UIInputViewController for custom keyboard)                │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ KeyboardView (SwiftUI)                                │   │
│  │                                                        │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ CandidateBar                                     │  │   │
│  │  │ ScrollView(.horizontal) { HStack { ... } }      │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  │                                                        │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ KeyGrid                                          │  │   │
│  │  │ LazyVGrid(columns: ...) { ... }                 │  │   │
│  │  │                                                  │  │   │
│  │  │  ┌──┬──┬──┬──┬──┐                              │  │   │
│  │  │  │ক │খ │গ │ঘ │ঙ │                              │  │   │
│  │  │  ├──┼──┼──┼──┼──┤                              │  │   │
│  │  │  │চ │ছ │জ │ঝ │ঞ │                              │  │   │
│  │  │  └──┴──┴──┴──┴──┘                              │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  │                                                        │   │
│  │  ┌────────────────────────────────────────────────┐  │   │
│  │  │ UtilityBar                                       │  │   │
│  │  │ HStack { Button("🎤") { } Button("⚙️") { } }  │  │   │
│  │  └────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ KeyboardViewModel (ObservableObject)                   │   │
│  │                                                        │   │
│  │  @Published var uiState: KeyboardUiState              │   │
│  │                                                        │   │
│  │  func onKeyPress(_ event: KeyEvent)                   │   │
│  │  func onSelectCandidate(at index: Int)                │   │
│  │  func onLayoutChange(_ layout: LayoutType)            │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ NativeEngineBridge (FFI)                              │   │
│  │                                                        │   │
│  │  func processKey(_ event: KeyEvent) -> [OutputAction] │   │
│  │  func getCandidates() -> [CandidateWord]              │   │
│  │  func setLayout(_ layoutId: String)                   │   │
│  │  func reset()                                         │   │
│  │  func setIncognito(_ enabled: Bool)                   │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

**State Management:**

```swift
// KeyboardViewModel.swift
import Combine

class KeyboardViewModel: ObservableObject {
    @Published var uiState = KeyboardUiState()
    
    private let engineBridge: NativeEngineBridge
    private var cancellables = Set<AnyCancellable>()
    
    init(engineBridge: NativeEngineBridge) {
        self.engineBridge = engineBridge
    }
    
    func onKeyPress(_ event: KeyEvent) {
        DispatchQueue.global(qos: .userInteractive).async { [weak self] in
            guard let self = self else { return }
            let actions = self.engineBridge.processKey(event)
            
            DispatchQueue.main.async {
                actions.forEach { action in
                    switch action {
                    case .commitText(let text):
                        self.textDocumentProxy.insertText(text)
                    case .updateComposition(let buffer):
                        self.updateComposition(buffer)
                    case .backspace(let count):
                        for _ in 0..<count {
                            self.textDocumentProxy.deleteBackward()
                        }
                    case .updateCandidates(let candidates):
                        self.uiState.candidates = candidates
                    case .nothing:
                        break
                    }
                }
            }
        }
    }
}

struct KeyboardUiState {
    var layout: LayoutType = .phonetic
    var candidates: [CandidateWord] = []
    var compositionText: String = ""
    var shiftState: ShiftState = .none
    var theme: ThemeConfig = .defaultDark
    var isIncognito: Bool = false
}
```

### 4.4 FFI Bridge Design

```rust
// core/src/ffi.rs — C-compatible FFI exports

// Android JNI
#[cfg(target_os = "android")]
#[no_mangle]
pub extern "system" fn Java_com_keybroad_core_NativeEngineBridge_nativeProcessKey(
    env: JNIEnv,
    _class: JClass,
    key_code: jint,
    unicode: jint,
    timestamp: jlong,
) -> jobjectArray {
    // Convert JNI types to Rust types
    // Call engine.process_key()
    // Convert OutputAction[] back to JNI array
}

// iOS FFI
#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn keybroad_process_key(
    key_code: u32,
    unicode: u32,
    timestamp: u64,
) -> *mut FFIOutputActions {
    // Convert C types to Rust types
    // Call engine.process_key()
    // Return boxed array of output actions
}

#[cfg(target_os = "ios")]
#[no_mangle]
pub extern "C" fn keybroad_free_actions(actions: *mut FFIOutputActions) {
    // Free the allocated action array
    unsafe { Box::from_raw(actions); }
}
```

---

## 5. On-Device AI Strategy

### 5.1 Technology Decision: **Hybrid N-gram + Quantized Transformer**

**Chosen:** Two-phase approach

| Phase | Model | Size | Latency | Accuracy |
|---|---|---|---|---|
| MVP (Phase 1) | Custom N-gram (Trigram) | <1MB | <1ms | ~75% top-3 |
| Phase 2 | Quantized Transformer (DistilBERT-Bengali) | 8-12MB | <5ms | >90% top-3 |

**Alternatives Considered:**
- Full Transformer — rejected for MVP: too large (100MB+), too slow on low-end devices
- Pure N-gram — rejected for Phase 2: no contextual understanding
- Federated Learning only — rejected: cold-start problem; need baseline model

### 5.2 MVP: N-gram Prediction Engine

```rust
// prediction/src/ngram.rs

pub struct NgramPredictor {
    /// Trigram frequency table: (w1, w2) → [(w3, freq)]
    trigrams: HashMap<(String, String), Vec<(String, u32)>>,
    /// Bigram frequency table: (w1) → [(w2, freq)]
    bigrams: HashMap<String, Vec<(String, u32)>>,
    /// Unigram frequencies: word → freq
    unigrams: HashMap<String, u32>,
    /// Total word count for normalization
    total_words: u32,
}

impl NgramPredictor {
    pub fn new(dictionary_path: &str) -> Self {
        // Load pre-computed n-gram frequencies from bundled JSON
        // Source: Bengali Wikipedia + news corpora (public domain)
    }

    pub fn predict(
        &self,
        context: &[String],  // Previous 1-2 words
        max_candidates: usize,
    ) -> Vec<CandidateWord> {
        if context.len() >= 2 {
            // Trigram: P(w3 | w1, w2)
            let key = (context[context.len()-2].clone(), context[context.len()-1].clone());
            self.trigrams.get(&key)
                .map(|candidates| self.rank(candidates, max_candidates))
                .unwrap_or_else(|| self.bigram_predict(&context[context.len()-1], max_candidates))
        } else if context.len() == 1 {
            self.bigram_predict(&context[0], max_candidates)
        } else {
            self.unigram_predict(max_candidates)
        }
    }

    fn rank(&self, candidates: &[(String, u32)], max: usize) -> Vec<CandidateWord> {
        let mut sorted = candidates.to_vec();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(max).map(|(word, freq)| {
            CandidateWord {
                word: word.as_bytes().to_owned(),
                score: freq as f32 / self.total_words as f32,
                source: WordSource::Dictionary,
            }
        }).collect()
    }
}
```

**Training Data:**
- Bengali Wikipedia dump (public domain) — ~500MB text
- Bengali news corpora (Prothom Alo, BBC Bangla — public articles)
- Pre-processed into trigram/bigram/unigram frequency tables
- Bundled as compressed JSON in the app (~500KB)

### 5.3 Phase 2: Quantized Transformer

```
Model Architecture:
  Input: token sequence [CLS] + last 128 tokens
  Architecture: DistilBERT-Bengali (6 layers, 768 hidden, 12 heads)
  Output: next-token probability distribution
  
  Quantization: INT8 post-training quantization
  Original size: ~65MB → Quantized: ~12MB
  Inference time: <5ms on Snapdragon 720G / A13 Bionic

  Format:
    Android: TensorFlow Lite (.tflite)
    iOS: CoreML (.mlmodelc)
    
  Conversion pipeline:
    PyTorch → ONNX → TFLite (INT8) / CoreML (INT8)
```

**Training Pipeline (offline):**
```
1. Collect Bengali text corpus (Wikipedia + news + books)
2. Tokenize with custom Bengali tokenizer (SentencePiece)
3. Train DistilBERT-Bengali on next-token prediction
4. Fine-tune on Bengali typing patterns (simulated)
5. Post-training quantization (INT8)
6. Export to TFLite and CoreML formats
7. Bundle in app (downloaded on first launch if needed)
```

**Federated Learning (Phase 2):**
```
Device Training:
  1. User types → local typing history collected
  2. Periodically (weekly), train local model delta
  3. Compute weight updates (gradients)
  4. Encrypt weight updates with device key
  5. Send ONLY encrypted weights to server (never text)

Server Aggregation:
  1. Receive encrypted weights from N devices
  2. Decrypt with respective device keys
  3. Federated Averaging: weighted_mean(weights)
  4. Encrypt aggregated model
  5. Broadcast updated model to all devices
  
Privacy Guarantee:
  - No raw text ever leaves any device
  - Only model gradients (numbers) are transmitted
  - Gradients are encrypted in transit
  - Server cannot reconstruct individual user data
```

### 5.4 Cold-Start Strategy

```
New User Experience:
  1. On first launch: load bundled N-gram model (pre-trained on public corpus)
  2. User gets decent predictions immediately (75% accuracy)
  3. As user types, local history builds up
  4. After ~500 words typed: switch to personalized N-gram
  5. After Phase 2 release: download quantized Transformer model
  6. After ~1000 words: Transformer predictions surpass N-gram
  
  Fallback chain:
    Transformer (if available + enough context)
      ↓
    Personalized N-gram (if enough history)
      ↓
    Static N-gram (bundled baseline)
```

---

## 6. Security & Privacy Design

### 6.1 Threat Model & Countermeasures

| Threat | Severity | Countermeasure |
|---|---|---|
| Malicious app reads keyboard memory | Critical | Process isolation, no shared memory, encrypted DB |
| Network sniffing during sync | High | E2EE, SSL Pinning, certificate transparency |
| APK reverse-engineering | High | ProGuard/R8 obfuscation, native code encryption, anti-tamper |
| Keylogger injection | Critical | Incognito mode, input sanitization, no third-party SDKs in keyboard process |
| Dictionary data extraction | Medium | SQLCipher encryption, hardware-backed key storage |
| Clipboard snooping | High | Clipboard history encrypted, auto-clear after timeout |

### 6.2 Process Isolation

```xml
<!-- AndroidManifest.xml -->
<service
    android:name=".KeyboardService"
    android:label="Keybroad"
    android:permission="android.permission.BIND_INPUT_METHOD"
    android:exported="true"
    android:process=":keyboard">
    <intent-filter>
        <action android:name="android.view.InputMethod" />
    </intent-filter>
    <meta-data
        android:name="android.view.im"
        android:resource="@xml/method" />
</service>

<!-- Keyboard process has NO internet permission -->
<!-- Only the main app process has INTERNET permission -->
<uses-permission android:name="android.permission.INTERNET"
    android:maxSdkVersion="28"
    tools:node="remove" />
```

```
Process Architecture (Android):
┌─────────────────────┐     ┌─────────────────────┐
│   Main App Process   │     │  Keyboard Process    │
│   (com.keybroad)     │     │  (com.keybroad:kb)   │
│                      │     │                      │
│  ✓ INTERNET          │     │  ✗ NO INTERNET       │
│  ✓ Settings UI       │     │  ✓ Keyboard UI       │
│  ✓ Theme Store       │     │  ✓ Core Engine       │
│  ✓ Cloud Sync        │     │  ✓ Local Database    │
│  ✓ Billing           │     │  ✓ AI Inference      │
│                      │     │                      │
│  Communicates via:   │ ←→ │  AIDL / ContentProvider│
│  IPC only            │     │  (encrypted)         │
└─────────────────────┘     └─────────────────────┘
```

### 6.3 Incognito Mode

```
Detection Mechanism:
  1. Check EditorInfo.inputType for:
     - TYPE_TEXT_VARIATION_PASSWORD
     - TYPE_TEXT_VARIATION_VISIBLE_PASSWORD
     - TYPE_TEXT_VARIATION_WEB_PASSWORD
     - TYPE_NUMBER_VARIATION_PASSWORD
  2. Check field hints for keywords: "password", "otp", "pin", "secret"
  3. Check accessibility node info (if available)

When Incognito Activated:
  ┌─────────────────────────────────────────┐
  │ INCognito MODE ACTIVE                    │
  │                                          │
  │  ✓ No dictionary lookup                  │
  │  ✓ No prediction/candidates shown        │
  │  ✓ No learning from input                │
  │  ✓ No clipboard history saved            │
  │  ✓ No typing history recorded            │
  │  ✓ Composition buffer flushed after use  │
  │  ✓ Visual indicator: "Incognito" badge   │
  └─────────────────────────────────────────┘
```

### 6.4 Encryption Architecture

```
Data Encryption Layers:
┌──────────────────────────────────────────────────┐
│ Layer 3: Network (E2EE)                          │
│   - TLS 1.3 with certificate pinning            │
│   - Payload encrypted with device public key     │
├──────────────────────────────────────────────────┤
│ Layer 2: Database (SQLCipher)                     │
│   - AES-256-CBC page encryption                  │
│   - Key derived from user PIN via PBKDF2         │
│   - Hardware-backed key storage                  │
├──────────────────────────────────────────────────┤
│ Layer 1: Key Storage (Hardware)                   │
│   - Android: StrongBox Keystore                  │
│   - iOS: Secure Enclave                          │
│   - Biometric-gated key access                   │
└──────────────────────────────────────────────────┘
```

### 6.5 Code Obfuscation

```
Android:
  - ProGuard/R8 with custom rules
  - Native code (.so) string encryption
  - Class/method name obfuscation
  - Anti-tamper checks (signature verification)
  - Root/jailbreak detection

iOS:
  - Swift compiler optimization (RELEASE builds)
  - Bitcode (if applicable)
  - Jailbreak detection
  - Runtime integrity checks

Both:
  - SSL Pinning on all network calls
  - Certificate transparency logging
  - No analytics/tracking SDKs in keyboard process
```

---

## 7. Backend & Cloud Sync Design

### 7.1 Technology Decision: **Custom Node.js + Fastify** (NOT Firebase)

**Chosen:** Custom Node.js server with Fastify framework

**Alternatives Considered:**
- Firebase Firestore — rejected: no native CRDT support; limited sync control; vendor lock-in
- Supabase — rejected: PostgreSQL-based but lacks real-time CRDT propagation
- Go/Rust backend — overkill for MVP; Node.js sufficient for sync API

**Justification:**
- Full control over CRDT merge logic
- WebSocket support for real-time sync
- Fastify: 2-3x faster than Express, TypeScript support
- Easy deployment (Docker → any cloud)
- Can migrate to Go/Rust later if needed

### 7.2 Server Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Fastify Server                          │
│                                                          │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐ │
│  │ REST API     │  │ WebSocket    │  │ FL Aggregator │ │
│  │ (Auth,       │  │ (CRDT Sync)  │  │ (Model Weight │ │
│  │  Billing,    │  │              │  │  Aggregation) │ │
│  │  Themes)     │  │              │  │               │ │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘ │
│         │                 │                   │          │
│  ┌──────▼─────────────────▼───────────────────▼───────┐ │
│  │                   Service Layer                      │ │
│  │  AuthService | SyncService | BillingService |       │ │
│  │  ThemeService | FLService                            │ │
│  └──────────────────────┬──────────────────────────────┘ │
│                          │                                │
│  ┌──────────────────────▼──────────────────────────────┐ │
│  │                   Data Layer                          │ │
│  │  PostgreSQL (users, encrypted blobs, themes)         │ │
│  │  Redis (sessions, rate limiting, pub/sub)            │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### 7.3 API Endpoints

```yaml
# Authentication
POST   /api/v1/auth/register          # Email + phone registration
POST   /api/v1/auth/login             # Login with JWT
POST   /api/v1/auth/refresh           # Refresh token
DELETE /api/v1/auth/logout            # Invalidate session

# Cloud Sync (CRDT)
GET    /api/v1/sync/pull              # Get latest sync state
       Params: last_timestamp, table
       Returns: CRDT operations since timestamp
       
POST   /api/v1/sync/push              # Push local CRDT operations
       Body: { operations: CrdtOp[] }
       Returns: { accepted: bool, conflicts: CrdtOp[] }

WebSocket /ws/v1/sync                 # Real-time CRDT propagation
       Events:
         - sync:push (device → server)
         - sync:pull (server → device)
         - sync:conflict (server → device, if any)

# Billing
POST   /api/v1/billing/subscribe      # Create subscription
POST   /api/v1/billing/cancel         # Cancel subscription
GET    /api/v1/billing/status         # Get subscription status
POST   /api/v1/billing/webhook        # Stripe/bKash webhook

# Theme Store
GET    /api/v1/themes                 # List themes (with pagination)
GET    /api/v1/themes/:id             # Get theme details
POST   /api/v1/themes/purchase        # Purchase theme
POST   /api/v1/themes/publish         # Creator publishes theme

# Federated Learning
POST   /api/v1/fl/upload-weights      # Upload encrypted model weights
GET    /api/v1/fl/download-model      # Download latest aggregated model
GET    /api/v1/fl/status              # Get FL training status
```

### 7.4 Sync Data Flow

```
Device → Server Push:
  1. Device collects local CRDT operations
  2. Encrypt operations with device key
  3. Send via WebSocket (or REST fallback)
  4. Server stores encrypted blob
  5. Server broadcasts to other devices (via WebSocket)

Server → Device Pull:
  1. Device requests operations since last_synced_timestamp
  2. Server returns encrypted operations
  3. Device decrypts and merges via CRDT algorithm
  4. Device updates local database
  5. Device sends acknowledgment

Conflict Resolution:
  - CRDT guarantees eventual consistency
  - No manual conflict resolution needed
  - Mathematical proof of convergence
  - Server is just a relay + storage (dumb pipe)
```

### 7.5 Authentication

```
Auth Flow:
  1. User registers with email + phone
  2. Server generates JWT (RS256)
  3. Device stores JWT in Keychain/Keystore
  4. All API calls include: Authorization: Bearer <jwt>
  5. JWT expires in 24h; refresh token valid for 30d
  
  Device Key Pair (for E2EE):
  - Generated on device during registration
  - Private key stored in Secure Enclave / StrongBox
  - Public key sent to server
  - Used to encrypt sync payloads
  - Server never has access to private key
```

---

## 8. Testing Strategy

### 8.1 Testing Pyramid

```
                    ┌──────────┐
                    │   E2E    │   10 tests (critical paths)
                    │  Tests   │
                    ├──────────┤
                  │   UI Tests  │   50 tests (per platform)
                  │             │
                  ├──────────────┤
                │ Integration    │   100 tests (API, DB, sync)
                │ Tests          │
                ├────────────────┤
              │   Unit Tests     │   500+ tests (engine, prediction)
              │                  │
              ├──────────────────┤
            │ Property-Based     │   1M+ random inputs (engine)
            │ Tests              │
            ├────────────────────┤
          │  Golden File Tests   │   10,000+ Bengali words (engine)
          │                      │
          └──────────────────────┘
```

### 8.2 Core Engine Testing (Rust)

```rust
// tests/engine_tests.rs

use keybroad_core::{BengaliEngine, KeyEvent, LayoutType};

#[test]
fn test_phonetic_basic_typing() {
    let mut engine = BengliEngine::new("layouts/").unwrap();
    
    // Type "k" → should produce "ক"
    let actions = engine.process_key(KeyEvent {
        key_code: 107,  // 'k'
        unicode: 107,
        is_down: true,
        timestamp_ms: 0,
    }).unwrap();
    
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        OutputAction.CommitText(text) => assert_eq!(text, "ক"),
        _ => panic!("Expected CommitText"),
    }
}

#[test]
fn test_conjunct_character() {
    let mut engine = BengliEngine::new("layouts/").unwrap();
    
    // Type "k" + "h" → should produce "খ"
    engine.process_key(key_event(107, 0)).unwrap(); // k
    let actions = engine.process_key(key_event(104, 0)).unwrap(); // h
    
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        OutputAction.CommitText(text) => assert_eq!(text, "খ"),
        _ => panic!("Expected CommitText for conjunct"),
    }
}

// Property-based testing
proptest! {
    #[test]
    fn engine_never_crashes_on_random_input(
        keys in prop::collection::vec(0u32..256, 0..1000)
    ) {
        let mut engine = BengliEngine::new("layouts/").unwrap();
        for key_code in keys {
            let _ = engine.process_key(KeyEvent {
                key_code,
                unicode: key_code,
                is_down: true,
                timestamp_ms: 0,
            });
        }
    }
    
    #[test]
    fn engine_output_always_valid_unicode(
        keys in prop::collection::vec(0u32..256, 1..100)
    ) {
        let mut engine = BengliEngine::new("layouts/").unwrap();
        for key_code in keys {
            if let Ok(actions) = engine.process_key(key_event(key_code, 0)) {
                for action in &actions {
                    if let OutputAction.CommitText(text) = action {
                        // Verify all characters are valid Unicode
                        assert!(text.chars().all(|c| !c.is_control()));
                    }
                }
            }
        }
    }
}
```

### 8.3 Golden File Testing

```rust
// tests/golden_files.rs

use std::fs;
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenWord {
    input: String,          // Keystroke sequence
    expected_output: String, // Expected Bengali output
    layout: String,
}

#[test]
fn test_golden_words() {
    let words: Vec<GoldenWord> = serde_json::from_str(
        &fs::read_to_string("tests/golden_words.json").unwrap()
    ).unwrap();
    
    for word in words {
        let mut engine = BengaliEngine::new("layouts/").unwrap();
        engine.set_layout(&word.layout);
        
        for ch in word.input.chars() {
            engine.process_key(KeyEvent {
                key_code: ch as u32,
                unicode: ch as u32,
                is_down: true,
                timestamp_ms: 0,
            }).unwrap();
        }
        
        let state = engine.get_state();
        let output = state.composition_buffer.to_string();
        assert_eq!(
            output, word.expected_output,
            "Failed for input '{}' in layout '{}'",
            word.input, word.layout
        );
    }
}
```

**Golden file generation (from Bengali Wiktionary):**
```python
# scripts/generate_golden_files.py
# Extract 10,000+ Bengali words from Wiktionary
# Map each word to its keystroke sequence for each layout
# Output: tests/golden_words.json
```

### 8.4 Performance Benchmarks

```
Benchmark Suite:
  1. Key-press-to-render latency
     - Tool: Custom profiler (Android: Systrace, iOS: Instruments)
     - Target: < 8ms
     - Test: 1000 rapid key presses, measure 99th percentile
  
  2. Prediction latency
     - Tool: Rust criterion benchmarks
     - Target: < 5ms
     - Test: 10,000 prediction calls, measure average
  
  3. Memory allocation
     - Tool: valgrind (Android), Instruments (iOS)
     - Target: 0 allocations per keystroke
     - Test: 10,000 keystrokes, count malloc calls
  
  4. App startup time
     - Tool: Platform profilers
     - Target: < 500ms to interactive keyboard
     - Test: Cold start from process creation
  
  5. Database query latency
     - Tool: Custom benchmark
     - Target: < 10ms for dictionary lookup
     - Test: 10,000 random word lookups
```

### 8.5 CI/CD Test Pipeline

```yaml
# .github/workflows/test.yml
name: Test Pipeline

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Rust unit tests
        run: cargo test --manifest-path core/Cargo.toml
      - name: Run property-based tests
        run: cargo test --manifest-path core/Cargo.toml prop_
      - name: Run golden file tests
        run: cargo test --manifest-path core/Cargo.toml golden_
      - name: Run benchmarks
        run: cargo bench --manifest-path core/Cargo.toml

  android-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Android
        run: ./gradlew assembleDebug
      - name: Run unit tests
        run: ./gradlew testDebugUnitTest
      - name: Run instrumentation tests
        run: ./gradlew connectedDebugAndroidTest

  ios-tests:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build iOS
        run: xcodebuild build -scheme Keybroad -sdk iphoneos
      - name: Run tests
        run: xcodebuild test -scheme Keybroad -sdk iphonesimulator

  performance-regression:
    runs-on: ubuntu-latest
    needs: [rust-tests]
    steps:
      - name: Check performance budgets
        run: cargo bench --manifest-path core/Cargo.toml -- --threshold 1.1
        # Fails if any benchmark regresses by >10%
```

---

## 9. Performance Budgets

| Operation | Budget | Implementation | Fallback |
|---|---|---|---|
| Key press → visual feedback | < 8ms | Custom Canvas/CALayer | Compose/SwiftUI (non-critical) |
| Core engine processing | < 2ms | Rust native | — |
| Prediction calculation | < 5ms | N-gram lookup / Transformer | Pre-computed cache |
| Gesture recognition | < 5ms | Precomputed curve tables | Reduced point sampling |
| Memory allocation per keystroke | 0 bytes | Arena allocator | Object pool |
| Dictionary lookup | < 1ms | Trie traversal | LRU cache |
| Database read (settings) | < 5ms | SQLCipher + indexed query | In-memory cache |
| CRDT merge (per operation) | < 3ms | Rust CRDT engine | Batch merge |
| Background RAM usage | < 50MB | Lean Rust core | Feature flags to disable |

### Latency Measurement

```
Profiler Integration:

Android:
  - Custom FrameMetricsAggregator for keyboard window
  - Systrace markers for: input_received, engine_start, engine_end, render_start, render_end
  - Logs FrameTime every 1000 keystrokes
  - Dev mode: on-screen FPS counter

iOS:
  - CADisplayLink for frame timing
  - os_signpost for custom instrumentation
  - Instruments Time Profiler for detailed analysis
  - Dev mode: on-screen FPS counter

Both:
  - Performance regression test in CI
  - Baseline: Pixel 5 (Android) / iPhone 11 (iOS)
  - Fail CI if any benchmark regresses > 10%
```

---

## 10. Build & CI/CD

### 10.1 Repository Structure

```
keybroad/
├── core/                          # Rust core engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs                 # Public API
│   │   ├── engine.rs              # TypeEngine implementation
│   │   ├── trie.rs                # Trie data structure
│   │   ├── layout.rs              # Layout loading/parsing
│   │   ├── prediction.rs          # N-gram prediction
│   │   ├── crdt.rs                # CRDT sync engine
│   │   ├── ffi.rs                 # FFI exports (JNI/C)
│   │   └── arena.rs               # Arena allocator
│   ├── tests/
│   │   ├── engine_tests.rs
│   │   ├── golden_files.rs
│   │   └── prop_tests.rs
│   ├── benches/
│   │   └── engine_bench.rs
│   └── layouts/                   # Layout JSON files
│       ├── phonetic.json
│       ├── jatiya.json
│       ├── probhat.json
│       ├── unijoy.json
│       └── english.json
│
├── android/                       # Android app
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── java/com/keybroad/
│   │   │   │   ├── KeyboardService.kt
│   │   │   │   ├── KeyboardViewModel.kt
│   │   │   │   ├── ui/
│   │   │   │   │   ├── KeyboardScreen.kt
│   │   │   │   │   ├── CandidateBar.kt
│   │   │   │   │   ├── KeyGrid.kt
│   │   │   │   │   └── UtilityBar.kt
│   │   │   │   ├── data/
│   │   │   │   │   ├── SettingsRepository.kt
│   │   │   │   │   ├── ThemeRepository.kt
│   │   │   │   │   └── SyncRepository.kt
│   │   │   │   └── bridge/
│   │   │   │       └── NativeEngineBridge.kt
│   │   │   ├── jni/               # JNI bridge
│   │   │   └── AndroidManifest.xml
│   │   └── build.gradle.kts
│   └── gradle/
│
├── ios/                           # iOS app
│   ├── Keybroad/
│   │   ├── Keyboard/
│   │   │   ├── KeyboardViewController.swift
│   │   │   ├── KeyboardViewModel.swift
│   │   │   ├── Views/
│   │   │   │   ├── KeyboardView.swift
│   │   │   │   ├── CandidateBar.swift
│   │   │   │   ├── KeyGrid.swift
│   │   │   │   └── UtilityBar.swift
│   │   │   ├── Bridge/
│   │   │   │   └── NativeEngineBridge.swift
│   │   │   └── Info.plist
│   │   └── Keybroad.xcodeproj
│
├── server/                        # Backend
│   ├── src/
│   │   ├── index.ts               # Fastify entry point
│   │   ├── routes/
│   │   │   ├── auth.ts
│   │   │   ├── sync.ts
│   │   │   ├── billing.ts
│   │   │   └── themes.ts
│   │   ├── services/
│   │   │   ├── AuthService.ts
│   │   │   ├── SyncService.ts
│   │   │   ├── BillingService.ts
│   │   │   └── FLService.ts
│   │   └── db/
│   │       ├── schema.sql
│   │       └── migrations/
│   ├── package.json
│   └── tsconfig.json
│
├── data/                          # Training data & dictionaries
│   ├── dictionary/
│   │   └── bn_words.json          # 100,000+ Bengali words
│   ├── ngrams/
│   │   └── bn_trigrams.json       # Trigram frequencies
│   └── golden/
│       └── golden_words.json      # 10,000+ test words
│
├── scripts/                       # Build & utility scripts
│   ├── build-rust-android.sh
│   ├── build-rust-ios.sh
│   ├── generate_golden_files.py
│   └── quantize_model.py
│
├── .github/
│   └── workflows/
│       ├── test.yml
│       ├── build-android.yml
│       └── build-ios.yml
│
├── PROJECT_CONTEXT.md
├── PROJECT_PROGRESS.md
├── ARCHITECTURE.md
└── README.md
```

### 10.2 Build Commands

```bash
# Core engine (Rust)
cargo build --target aarch64-linux-android --release  # Android ARM64
cargo build --target aarch64-apple-ios --release       # iOS ARM64
cargo build --target wasm32-unknown-unknown --release  # Future web

# Android
./gradlew assembleDebug    # Debug build
./gradlew assembleRelease  # Release build (signed)

# iOS
xcodebuild build -scheme Keybroad -sdk iphoneos -configuration Release

# Server
npm run build              # TypeScript compilation
npm run start              # Start server

# Testing
cargo test                 # Rust tests
cargo bench                # Rust benchmarks
./gradlew test             # Android tests
xcodebuild test            # iOS tests
```

### 10.3 Release Pipeline

```
1. Code Push → GitHub
2. CI runs: lint, test, benchmark
3. If all pass → Build artifacts
4. Android: AAB → Play Store (internal testing)
5. iOS: IPA → TestFlight
6. Manual QA on physical devices
7. Promote to production track
8. Monitor Crashlytics for 48h
9. If stable → Public release
```

---

## 11. Risk Register

| # | Risk | Probability | Impact | Mitigation | Owner |
|---|---|---|---|---|---|
| 1 | Rust learning curve delays Phase 1 | Medium | High | Start with Kotlin fallback; migrate to Rust after MVP | Tech Lead |
| 2 | Compose/SwiftUI latency >8ms on low-end | Medium | High | Custom Canvas/CALayer for critical path | UI Lead |
| 3 | Bengali conjunct bugs in engine | High | Critical | Property-based testing; 10,000+ golden files | QA Lead |
| 4 | CRDT sync data corruption | Low | Critical | Formal verification; multi-device test matrix | Backend Lead |
| 5 | AI model too large for 40MB APK | Medium | Medium | Download on first launch; feature flags | ML Lead |
| 6 | Voice typing WER >5% | Medium | Medium | Collect more dialect data; iterate | ML Lead |
| 7 | App store rejection (accessibility perms) | Low | High | Early compliance review; clear documentation | Product |
| 8 | Ridmik releases AI features first | Low | Low | Focus on privacy + performance differentiators | Product |
| 9 | Team scaling (need Rust + ML experts) | High | Medium | Training program; hire specialists | Management |
| 10 | Scope creep from Phase 2/3 | High | High | Strict feature flags; MVP-only in Phase 1 | Product |

---

## Appendix A: Open Questions (Resolved)

| Question | Resolution |
|---|---|
| Core engine language? | **Rust** — cross-platform, zero-cost abstractions, no GC |
| Cloud backend? | **Custom Node.js + Fastify** — full CRDT control, no vendor lock-in |
| AI model for MVP? | **N-gram (Trigram)** — <1MB, <1ms, good enough for cold start |
| UI framework? | **Jetpack Compose + SwiftUI** — with custom Canvas fallback for critical path |
| Local database? | **SQLCipher** — encrypted, FTS5 support, cross-platform |
| CRDT library? | **Custom implementation** — no suitable Rust CRDT crate exists |
| Process isolation? | **Android: separate process, no internet** — OS-enforced |
| Testing approach? | **Property-based + Golden files + Benchmarks** — 1M+ random inputs |

---

*This document is the authoritative architecture reference for the Modern Bengali Keyboard project. All implementation must follow these decisions unless a formal architecture review approves a change.*

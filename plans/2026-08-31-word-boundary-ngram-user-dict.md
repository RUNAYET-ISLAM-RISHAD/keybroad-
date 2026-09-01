# Implementation Plan: Word Boundary Detection, N-gram Prediction & User Dictionary

**Date:** 2026-08-31
**Status:** PLAN MODE — Awaiting Approval
**Scope:** Extend core engine with word boundary detection, basic n-gram next-word prediction, and runtime user dictionary

---

## 1. Objective

Extend the Bengali keyboard core engine to:
1. **Detect word boundaries** — track the word being typed and finalize it when space/punctuation is pressed
2. **Predict next words** — use a simple bigram/trigram model to suggest what the user will type next
3. **Allow user dictionary** — let users add custom words at runtime that participate in suggestions

This prepares the engine for the UI suggestion bar layer without implementing any UI code.

---

## 2. Files to Create

### 2.1 `core/data/corpus.txt` — Bengali Sentence Corpus
- **Purpose:** Seed data for building bigram/trigram frequency tables
- **Format:** One sentence per line, UTF-8, Bengali script
- **Size:** 50-100 hand-crafted sentences covering common conversational patterns
- **Content categories:** Greetings, questions, daily activities, directions, food, family, weather, work, school, shopping
- **Loaded at compile time** via `include_str!` (same pattern as words.txt)

### 2.2 `core/src/ngram.rs` — N-gram Prediction Model
- **Struct:** `NgramModel`
- **Fields:**
  - `bigrams: HashMap<String, Vec<(String, u32)>>` — maps prev_word → [(next_word, count)]
  - `trigrams: HashMap<(String, String), Vec<(String, u32)>>` — maps (prev1, prev2) → [(next_word, count)]
- **Methods:**
  - `new() -> Self` — empty model
  - `load_embedded() -> Self` — parse corpus.txt, build frequency tables
  - `train_sentence(&mut self, words: &[String])` — add a sentence to the model
  - `predict_next(&self, context: &[String], limit: usize) -> Vec<CandidateWord>` — predict next words
- **Algorithm:**
  1. If context has ≥2 words → check trigram `(context[n-2], context[n-1])`
  2. If trigram exists → return top `limit` by count, score = count / max_count
  3. Else → fallback to bigram `context[n-1]`
  4. If bigram exists → return top `limit` by count
  5. Else → return empty vec
- **Score normalization:** `score = count as f32 / max_count_in_group as f32` (0.0 to 1.0)
- **Source tag:** `WordSource::AiPrediction`

---

## 3. Files to Modify

### 3.1 `core/src/types.rs` — Extend EngineState

Add three new fields to `EngineState`:

```rust
/// The word currently being typed (accumulates characters until word boundary)
pub current_word: String,

/// History of previously completed words (capped at 20)
pub history: Vec<String>,

/// The most recently committed word (for n-gram context lookup)
pub last_committed_word: Option<String>,
```

Update `Default` implementation to initialize these:
- `current_word: String::new()`
- `history: Vec::new()`
- `last_committed_word: None`

### 3.2 `core/src/engine.rs` — Integrate Everything

**New fields on `BengaliEngine`:**
```rust
/// N-gram prediction model
ngram: NgramModel,

/// User-added words (runtime, mutable)
user_words: HashSet<String>,
```

**Modified `new()` constructor:**
- Initialize `ngram: NgramModel::load_embedded()`
- Initialize `user_words: HashSet::new()`

**Modified `process_key()` logic:**
After the existing key processing (layout lookup + composition), add word boundary detection:

```
If output_str is space or punctuation (.,!?;:):
  1. If current_word is non-empty:
     - Push current_word to history (front, capped at 20)
     - Set last_committed_word = Some(current_word.clone())
     - Clear current_word
  2. Then process the character normally (CommitText)

If output_str is a regular character:
  1. Append each character to current_word
  2. Process the character normally (existing logic)
```

**Punctuation set:** `['.', ',', '!', '?', ';', ':', ' ', '\n']`

**New public methods:**

```rust
/// Manually finalize the current word (useful for UI trigger like "send" button)
pub fn finalize_word(&mut self);

/// Add a word to the user dictionary
pub fn add_user_word(&mut self, word: &str);

/// Get next-word suggestions based on n-gram context
pub fn get_next_word_suggestions(&self, limit: usize) -> Vec<CandidateWord>;

/// Check if a word is in the user dictionary
pub fn is_user_word(&self, word: &str) -> bool;
```

**Modified `get_suggestions()` method:**
- Before checking main dictionary, check `user_words` first
- If `current_word` is a prefix of any user word, add those as high-score candidates
- Then proceed with existing dictionary logic

**Modified `reset()` method:**
- Also clear `current_word`, `history`, `last_committed_word`

### 3.3 `core/src/lib.rs` — Add Module

```rust
pub mod ngram;
pub use ngram::NgramModel;
```

---

## 4. Test Plan

### 4.1 Unit Tests in `core/src/ngram.rs`

| Test | Purpose |
|------|---------|
| `test_ngram_load_embedded` | Verify corpus loads without error, model is non-empty |
| `test_ngram_empty_model` | Empty model returns empty predictions |
| `test_ngram_bigram_prediction` | After "আমি", predict words like "যাব", "বলেছি", etc. |
| `test_ngram_trigram_prediction` | After "আমি আজকে", predict "বাড়ি", "যাব", etc. |
| `test_ngram_fallback_to_bigram` | Unknown trigram falls back to bigram |
| `test_ngram_unknown_context` | Unknown word returns empty predictions |
| `test_ngram_limit` | Limit parameter caps results |
| `test_ngram_train_sentence` | Adding a sentence updates predictions |
| `test_ngram_score_range` | All scores are between 0.0 and 1.0 |

### 4.2 Integration Tests in `core/tests/engine_tests.rs` (or new file)

| Test | Purpose |
|------|---------|
| `test_word_boundary_space` | Type "আমি" + space → history contains "আমি", current_word cleared |
| `test_word_boundary_punctuation` | Type "ভালো" + "." → history contains "ভালো" |
| `test_word_boundary_multiple_words` | Type "আমি যাব" → history has ["আমি", "যাব"] in correct order |
| `test_history_cap` | Type 25 words → history only has last 20 |
| `test_next_word_suggestions` | After "আমি" + space, get_next_word_suggestions returns predictions |
| `test_trigram_context` | After "আমি আজকে" + space, trigram predictions are returned |
| `test_finalize_word` | finalize_word() clears current_word and updates history |
| `test_add_user_word` | add_user_word("রিকশা") → is_user_word("রিকশা") returns true |
| `test_user_word_in_suggestions` | add_user_word, then get_suggestions includes user word |
| `test_user_word_not_in_dict` | User word not in main dictionary still appears in suggestions |
| `test_incognito_no_history` | In incognito mode, history is not updated |
| `test_reset_clears_history` | reset() clears current_word, history, last_committed_word |
| `test_existing_103_tests_still_pass` | All pre-existing tests remain green |

### 4.3 Test File Strategy
- Add n-gram unit tests in `core/src/ngram.rs` (inline `#[cfg(test)]` module)
- Add engine integration tests in `core/tests/engine_tests.rs` (extend existing file)
- OR create `core/tests/ngram_integration_tests.rs` (new file, keeps concerns separate)

**Recommendation:** Create `core/tests/ngram_integration_tests.rs` to avoid bloating the existing 579-line engine_tests.rs file.

---

## 5. Implementation Order

Execute in this exact sequence to maintain green tests at each step:

### Step 1: Create `core/data/corpus.txt`
- Write 50-100 Bengali sentences
- Verify UTF-8 encoding

### Step 2: Create `core/src/ngram.rs`
- Implement `NgramModel` struct and all methods
- Include inline unit tests
- Verify: `cargo test ngram` passes

### Step 3: Update `core/src/lib.rs`
- Add `pub mod ngram; pub use ngram::NgramModel;`
- Verify: `cargo build` succeeds

### Step 4: Extend `EngineState` in `core/src/types.rs`
- Add 3 new fields: `current_word`, `history`, `last_committed_word`
- Update `Default` implementation
- Verify: `cargo test` — all 103 existing tests still pass (new fields have defaults)

### Step 5: Extend `BengaliEngine` in `core/src/engine.rs`
- Add `ngram: NgramModel` and `user_words: HashSet<String>` fields
- Update `new()` to initialize them
- Update `reset()` to clear new state
- Modify `process_key` / `process_single_char` for word boundary detection
- Add `finalize_word()`, `add_user_word()`, `get_next_word_suggestions()`, `is_user_word()`
- Modify `get_suggestions()` to check user dictionary
- Verify: `cargo test` — all tests pass

### Step 6: Create integration tests
- Create `core/tests/ngram_integration_tests.rs`
- Run full suite, verify all pass

### Step 7: Update memory files
- `PROJECT_PROGRESS.md` — mark features complete
- `PROJECT_CONTEXT.md` — update architecture state

---

## 6. Design Decisions & Rationale

### 6.1 Why HashSet for user_words instead of a second Dictionary/Trie?
- **Simplicity:** User dictionary is small (dozens to hundreds of words), not thousands
- **Mutable at runtime:** HashSet supports O(1) insert and lookup
- **No prefix matching needed for user words:** They are exact matches added by the user
- **Can upgrade later:** If user dictionary grows large, migrate to a Trie

### 6.2 Why not modify process_key for n-gram training?
- The architecture specifies the engine should be "pure and deterministic"
- Training on every keystroke would add latency
- Instead: train when `finalize_word()` is called or when space/punctuation is pressed
- This keeps the hot path clean

### 6.3 History cap at 20 words
- Balances memory usage with context richness
- Trigram needs 2 words of context; 20 words provides ~10 bigram/trigram lookups
- Matches ARCHITECTURE.md suggestion of bounded history

### 6.4 Word boundary triggers
- Space (key_code 62 in phonetic/english layouts)
- Punctuation: `.`, `,`, `!`, `?`, `;`, `:`
- Enter key (key_code 66) — also finalizes the word
- NOT backspace — backspace should not finalize a word

### 6.5 Score normalization
- Bigram scores: `count / max_bigram_count_for_this_context`
- Trigram scores: `count / max_trigram_count_for_this_context`
- This gives scores in [0.0, 1.0] range, compatible with existing `CandidateWord.score`

---

## 7. Performance Considerations

| Operation | Budget | Expected | Notes |
|-----------|--------|----------|-------|
| Word boundary detection | < 0.1ms | ~0.01ms | String comparison + HashSet insert |
| N-gram lookup | < 5ms | ~0.1ms | HashMap lookup, no iteration over large sets |
| User dictionary check | < 1ms | ~0.01ms | HashSet::contains is O(1) |
| Corpus parsing (one-time) | < 100ms | ~10ms | 50-100 sentences, runs once at init |
| Memory per keystroke | 0 bytes | ~0 bytes | No allocation in hot path (strings are reused) |

---

## 8. Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| Bengali word segmentation is complex (no spaces between some words) | Medium | Start with space/punctuation as boundaries; Phase 2 can add ML-based segmentation |
| N-gram model too small for useful predictions | Low | Seed with 50-100 sentences; can expand corpus later |
| User dictionary not persisted across sessions | Low | In-memory only for now; Phase 2 adds SQLite persistence |
| Existing tests break from EngineState changes | Medium | New fields have Default values; run full suite after each step |
| current_word accumulates garbage if user deletes via backspace | Medium | Also decrement current_word on backspace when it's non-empty |

---

## 9. What This Plan Does NOT Include (Explicit Exclusions)

- No UI integration (suggestion bar rendering)
- No federated learning or adaptive personalization
- No persistent storage of user dictionary (in-memory only)
- No ML-based word segmentation
- No next-word prediction from cloud/API
- No modification to the dictionary.txt or words.txt files
- No changes to the conjunct handling logic

---

## 10. Expected Outcome

After implementation:
- **Test count:** ~120+ tests (103 existing + ~17 new)
- **New modules:** `ngram.rs` (~150 lines)
- **New data file:** `corpus.txt` (~100 lines)
- **Modified files:** `types.rs` (+15 lines), `engine.rs` (+80 lines), `lib.rs` (+2 lines)
- **New test file:** `ngram_integration_tests.rs` (~200 lines)

---

*This plan is ready for review. Once approved, I will execute each step sequentially, running tests after each change to ensure no regressions.*

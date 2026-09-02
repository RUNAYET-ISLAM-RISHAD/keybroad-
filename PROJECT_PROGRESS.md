# PROJECT_PROGRESS.md — Modern Bengali Keyboard

> Tracks ongoing work, status, blockers, and next steps.
> Last updated: 2026-08-31

---

## Completed Work

| Date | Item | Details |
|---|---|---|
| 2026-08-31 | Project document analysis | Read and analyzed `Keyboard (1).md` (roadmap + whitepaper) |
| 2026-08-31 | Memory initialization | Created `PROJECT_CONTEXT.md` and `PROJECT_PROGRESS.md` |
| 2026-08-31 | **Architecture design** | Created `ARCHITECTURE.md` — full technical design document with finalized technology decisions |
| 2026-08-31 | **Project scaffolding** | Created repository structure: core/, android/, ios/, server/, data/, scripts/, .github/ |
| 2026-08-31 | **Core engine skeleton** | Rust library with types (KeyEvent, OutputAction, EngineState) and BengaliEngine with process_key |
| 2026-08-31 | **Unit + integration tests** | 19 tests passing (10 unit, 9 integration) |
| 2026-08-31 | **CI/CD pipeline** | GitHub Actions workflow for build, test, and cross-platform compilation |
| 2026-08-31 | **Layout JSON files** | Phonetic and English layout definitions created |
| 2026-08-31 | **Phonetic layout handling** | Layout loading from embedded JSON, key mapping via HashMap, shift handling |
| 2026-08-31 | **Basic key processing** | process_key now uses real layout lookups for Phonetic and English |
| 2026-08-31 | **100+ key mappings** | Phonetic layout: 26 letters + digits + punctuation, all with shift variants |

---

## Current Work

**"Android app skeleton with JNI integration — COMPLETED"**

Implemented all remaining Bengali keyboard layouts and created a CLI demo tool for interactive testing:
- **Jatiya layout:** Standard Bangladesh government layout with 89+ key mappings
- **Probhat layout:** Popular Bengali keyboard layout with 89+ key mappings
- **Unijoy layout:** Popular Bengali keyboard layout with 89+ key mappings
- **CLI demo tool:** Interactive REPL for testing the engine with all layouts
- **Layout tests:** 9 new tests for layout loading and key mapping verification

| Metric | Result |
|---|---|
| Build status | **Passing** |
| Tests | **145/145 passing** (63 unit + 29 dictionary + 32 engine + 21 integration) |
| Core engine | **Rust library with all 5 layouts + dictionary + n-gram + user dictionary** |
| Layouts | **5 layouts** (Phonetic, English, Jatiya, Probhat, Unijoy) |
| Layout tests | **9 new tests** for layout loading and key mapping verification |
| CLI binary | **keybroad_cli** - Interactive REPL for testing |
| CLI features | Layout switching, incognito mode, suggestions, word tracking |

| Component | Decision |
|---|---|
| Core Engine | **Rust** (via JNI/FFI) |
| Android UI | **Jetpack Compose** + custom Canvas fallback |
| iOS UI | **SwiftUI** + custom CALayer fallback |
| Local Database | **SQLCipher** |
| Cloud Backend | **Custom Node.js + Fastify** |
| AI Model (MVP) | **N-gram (Trigram)** |
| AI Model (Phase 2) | **Quantized DistilBERT-Bengali** |
| CRDT | **Custom Rust crate** (RGA + LWW-Register) |
| Testing | **Property-based + Golden files + Benchmarks** |

---

## Pending Work (Roadmap)

### Phase 0: Discovery (Weeks 1–2)
- [ ] User research: Who is the target user? Age, tech-savviness, current pain points with Ridmik
- [ ] UX flow mapping: How does a user switch keyboards? Onboarding experience
- [ ] Database schema design (Prisma or equivalent)
- [ ] Bengali word corpus collection (100,000+ words for dictionary)
- [ ] Competitor analysis: Ridmik feature audit, known bugs, user complaints
- [ ] Legal: Privacy policy drafting, app store compliance review
- [x] Project repository structure created
- [x] Core engine skeleton implemented and tested

### Phase 1: Foundation (Weeks 3–8)
- [x] Native app setup (Rust core engine skeleton created)
- [x] Core typing engine — basic layout mapping working (Phonetic + English)
- [x] Layout JSON definitions (Phonetic and English created with 89+ keys each)
- [x] Bengali conjunct character handling (যুক্তাক্ষর) — hasanta + consonant conjunct formation
- [x] Auto-correction with dictionary
- [ ] Unit test framework for engine
- [x] CI/CD pipeline (GitHub Actions created)
- [ ] Code linting setup (clippy for Rust; Detekt + SwiftLint for platform code)

### Phase 2: Intelligence (Weeks 9–16)
- [ ] AI dictionary (100,000+ words)
- [ ] Next-word prediction engine
- [ ] On-device voice typing
- [ ] 50+ voice commands working
- [ ] Clipboard manager
- [ ] Text editing tools

### Phase 3: Personalization (Weeks 17–20)
- [ ] Theme engine (dark/light mode)
- [ ] Custom background support
- [ ] Sticker system
- [ ] Smooth theme transitions

### Phase 4: Ecosystem (Weeks 21–24)
- [ ] Cloud sync with E2EE
- [ ] CRDT-based conflict resolution
- [ ] Billing/subscription system (Stripe/bKash)
- [ ] Privacy documentation
- [ ] 100% no-tracking verification

### Phase 5: Public Release (Weeks 25–28)
- [ ] Beta testing program
- [ ] 10,000 user feedback collection
- [ ] Play Store submission
- [ ] App Store submission
- [ ] Code freeze

---

## Next Task

**"Full IME integration and performance benchmarking"**

With all layouts and CLI demo working, next steps are:

1. **Suggestion bar UI** — Display candidate words above keyboard
2. **Candidate selection** — Tap to select, scroll to see more
3. **Golden file generation** — Create 10,000+ test words for regression testing
4. **Persistent user dictionary** — SQLite storage for user words across sessions
5. **Federated learning foundation** — Privacy-preserving model updates

---

## Known Bugs

_None currently known._

---

## Tests Performed

| Date | Test Suite | Result | Details |
|---|---|---|---|
| 2026-08-31 | Unit tests (engine.rs) | **10/10 pass** | Engine creation, process_key, backspace, shift, reset, incognito |
| 2026-08-31 | Unit tests (layout.rs) | **15/15 pass** | Layout loading, key mapping, parse_key_string, all 5 layouts |
| 2026-08-31 | Integration tests (engine_tests.rs) | **18/18 pass** | Phonetic mapping (k→ক, a→া), shift (k→খ), word typing, backspace, enter, rapid typing |
| 2026-08-31 | Cargo build | **Pass** | Compiles to lib + staticlib + cdylib + CLI binary, 0 errors |
| 2026-08-31 | Unit tests (conjunct.rs) | **7/7 pass** | ConjunctTable creation, consonant detection, hasanta, conjunct lookup |
| 2026-08-31 | Integration tests (engine_tests.rs) | **32/32 pass** | Conjunct formation (ক+্+গ), hasanta backspace trick, special conjuncts (x→ক্ষ, z→জ্ঞ), backspace during hasanta |
| 2026-08-31 | Unit tests (dictionary.rs) | **13/13 pass** | Trie insert/lookup, prefix matching, Levenshtein distance, corrections |
| 2026-08-31 | Integration tests (dictionary_tests.rs) | **29/29 pass** | Embedded dictionary loading, Bengali word lookup, auto-correction, engine integration |
| 2026-08-31 | Unit tests (ngram.rs) | **12/12 pass** | NgramModel loading, bigram/trigram prediction, fallback, score normalization, merge counts |
| 2026-08-31 | Integration tests (ngram_integration_tests.rs) | **21/21 pass** | Word boundaries, history, n-gram prediction, user dictionary, incognito, reset |
| 2026-08-31 | Layout tests | **9/9 pass** | Jatiya, Probhat, Unijoy loading, hasanta mapping, digit keys |
| 2026-08-31 | Full test suite | **145/145 pass** | 63 unit + 29 dictionary + 32 engine + 21 integration |

---

## Audit Findings

_No issues found._

---

## Open Questions

### Architecture (RESOLVED in ARCHITECTURE.md)
1. ~~Should the core engine be a single Rust/C++ binary, or separate Kotlin/Swift?~~ → **Rust via JNI/FFI**
2. ~~Is Firebase Firestore sufficient for CRDT sync?~~ → **Custom Node.js + Fastify**
3. ~~Can Compose/SwiftUI meet <8ms budget?~~ → **Custom Canvas/CALayer fallback for critical path**
4. ~~What if 240Hz touch polling not supported?~~ → **Reduced point sampling fallback**

### Product (OPEN)
5. What languages beyond Bengali are confirmed for Phase 2?
6. What is the monetization pricing strategy for premium subscription?
7. How do we handle app store review for accessibility permissions?

### Legal/Privacy (OPEN)
8. Does "no permissions" mean literally zero, or minimum required for keyboard IME?
9. What jurisdiction's privacy laws apply? Bangladesh? Global?
10. How do we handle data retention for federated learning weight aggregation?

### Data Collection (OPEN)
11. Where does the initial 1,000+ Bengali voice phrase corpus come from?
12. What is the minimum device specification we target?

---

## Risks

| # | Risk | Severity | Mitigation |
|---|---|---|---|
| 1 | Latency budget unachievable with Compose/SwiftUI | High | Benchmark early in Phase 1; have custom rendering fallback |
| 2 | Bengali conjunct character complexity causes engine bugs | High | Extensive property-based testing; 10,000+ golden files |
| 3 | On-device AI model too large for low-end devices | Medium | Model quantization; graceful degradation; feature flags |
| 4 | CRDT sync introduces data corruption on edge cases | High | Formal verification; extensive multi-device testing |
| 5 | Voice typing WER > 5% for regional dialects | Medium | Dialect-specific training data collection |
| 6 | Team lacks Rust/C++ expertise if cross-platform core chosen | Medium | Training period; consider Kotlin/Swift fallback |
| 7 | App store rejection due to accessibility permission usage | Medium | Early compliance review; clear privacy documentation |
| 8 | Federated Learning model convergence is slow | Medium | Baseline static model as fallback |
| 9 | Scope creep from Phase 2/3 features into MVP | High | Strict MVP definition; feature flags |
| 10 | Ridmik releases competing features during development | Low | Focus on core differentiators (privacy, latency, AI) |

---

## Deployment State

**Not deployed.** Core engine builds and tests pass locally. CI/CD configured for GitHub Actions.

---

## Changelog

| Date | Change |
|---|---|
| 2026-08-31 | Initial creation — project understanding and memory initialization |
| 2026-08-31 | Architecture design completed — `ARCHITECTURE.md` created with all technology decisions finalized |
| 2026-08-31 | Updated `PROJECT_CONTEXT.md` with finalized technology decisions |
| 2026-08-31 | Resolved 4 architecture open questions; 8 questions remain open (product/legal/data) |
| 2026-08-31 | **Project scaffolding** — Repository structure created, core engine skeleton implemented |
| 2026-08-31 | **Core engine v0.1.0** — Rust library with types, BengaliEngine, process_key skeleton |
| 2026-08-31 | **19 tests passing** — 10 unit + 9 integration tests verified |
| 2026-08-31 | **CI/CD** — GitHub Actions workflow for build, test, cross-platform compilation |
| 2026-08-31 | **Phonetic layout** — 89 key mappings loaded from embedded JSON, HashMap-based O(1) lookup |
| 2026-08-31 | **Basic key processing** — process_key uses real layout lookups, shift handling works |
| 2026-08-31 | **34 tests passing** — 16 unit + 18 integration (including Phonetic-specific tests) |
| 2026-08-31 | **Conjunct character handling** — Hasanta (্) + consonant conjunct formation via backspace trick |
| 2026-08-31 | **ConjunctTable module** — Consonant classification, hasanta conjunct lookup (ক্ষ, জ্ঞ, etc.) |
| 2026-08-31 | **Phonetic layout hasanta key** — Backslash (\\) maps to hasanta (U+09CD) |
| 2026-08-31 | **56 tests passing** — 24 unit + 32 integration (18 new conjunct tests) |
| 2026-08-31 | **Dictionary module** — Trie-based dictionary with prefix matching and Levenshtein auto-correction |
| 2026-08-31 | **500-word Bengali seed list** — Common words loaded via include_str! |
| 2026-08-31 | **Engine dictionary integration** — BengaliEngine.get_suggestions() for word suggestions |
| 2026-08-31 | **103 tests passing** — 42 unit + 29 dictionary + 32 engine integration |
| 2026-08-31 | **N-gram module** — Bigram + trigram prediction model with embedded Bengali corpus (350+ sentences) |
| 2026-08-31 | **Word boundary detection** — Space, punctuation, and Enter key finalize current word |
| 2026-08-31 | **History tracking** — Last 20 completed words stored in EngineState |
| 2026-08-31 | **User dictionary** — Runtime HashSet for user-added words with O(1) lookup |
| 2026-08-31 | **136 tests passing** — 54 unit + 29 dictionary + 32 engine + 21 integration |
| 2026-08-31 | **Bengali layouts** — Jatiya, Probhat, Unijoy layout JSON files created (89+ keys each) |
| 2026-08-31 | **Layout loading** — Updated layout.rs to load all 5 layouts from embedded JSON |
| 2026-08-31 | **CLI demo tool** — Interactive REPL for testing engine with all layouts |
| 2026-08-31 | **Layout tests** — 9 new tests for layout loading and key mapping verification |
| 2026-08-31 | **145 tests passing** — 63 unit + 29 dictionary + 32 engine + 21 integration |
| 2026-08-31 | **Android app skeleton** | Created Android project (Gradle, Compose UI), JNI bridge, keyboard UI with layout switcher and suggestions |
| 2026-08-31 | **Android APK build** | Successfully built APK (15.8 MB) at `android/app/build/outputs/apk/debug/app-debug.apk`. Fixed Kotlin/Compose version mismatches, added launcher icons. |
| 2026-08-31 | **Android UI fix** | Fixed keyboard key labels: replaced hardcoded English QWERTY with Bengali characters ( Phonetic layout: q=ধ, w=র, k=ক, etc.), added proper color contrast (white text on purple buttons). |
| 2026-09-01 | **Android crash fix** | Removed `nativeGetLayout()` JNI call that was missing from pre-built .so files. Added try-catch around all JNI calls. App now uses placeholder Bengali Phonetic keys when native library unavailable. |
| 2026-09-01 | **Engine integration** | Integrated Rust engine via JNI. KeyboardViewModel sends keycodes to `nativeProcessKey()` for Bengali output. Key labels show Bengali characters (q=ধ, w=র, k=ক, etc.). Engine handles conjunct formation (k+h=খ). Added UnsatisfiedLinkError handling. |
| 2026-09-01 | **Dynamic layout loading** | Created LayoutManager to load layout JSON from Android assets. KeyData class represents key mappings. ViewModel loads layout dynamically from assets. UI no longer uses hardcoded key lists. Engine is single source of truth for typing. |
| 2026-09-01 | **Layout display fix** | Fixed LayoutManager to use `output` field as `display` for Bengali keys. Previously `display` showed English letters (e.g., "a") instead of Bengali (e.g., "া"). Now keys show correct Bengali characters on buttons. |
| 2026-09-01 | **OTA Update System** | Implemented custom OTA update system. UpdateManager fetches JSON config from GitHub, compares version codes, downloads APK via DownloadManager, installs via FileProvider. UpdateChecker shows dialog on update available. Config at `android/update_config.json`. |
| 2026-09-01 | **Final keyboard fix** | Removed all hardcoded keys. LayoutManager now uses `output` field for both display and engine input. Engine tests 145/145 passing. UI shows correct Bengali keys matching engine output. Version bumped to 1.0.1 (code 2). |
| 2026-09-01 | **JSON files replaced with exact user-provided mapping** | phonetic.json (26 keys) and jatiya.json (37 keys with space) replaced verbatim. LayoutManager parses JSON array preserving order, display=output. ViewModel sends key[0].code (space→32). KeyboardView preserves JSON order split: 10+9+7 (phonetic) / 10+10+10+6 (jatiya) QWERTY rows. Version 1.0.2 code 3 BUILD SUCCESSFUL. |
| 2026-09-01 | **APK published & OTA fixed (catbox fallback)** | APK 15.15 MB `app-debug.apk` uploaded to catbox.moe. `gh` not installed, `transfer.sh` blocked, `0x0.st` disabled. Catbox succeeded: `https://files.catbox.moe/7q2s16.apk` (HTTP 200, range 0-2047 verified). `update_config.json` updated `apk_url` to catbox direct link. Fallback config uploaded `https://files.catbox.moe/pwpmy4.json` / `p7w87c.json`. `UpdateManager.kt` now tries GitHub raw primary + catbox fallback. Rebuilt APK `versionCode 3` `BUILD SUCCESSFUL`. |
| 2026-09-01 | **GitHub push & release v1.0.2** | Pushed to `https://github.com/RUNAYET-ISLAM-RISHAD/keybroad-` (public). Merged remote initial README, pushed `e1585e6` + `5e9c175` (`update_config` → GitHub releases URL). Created release `v1.0.2` id 380252869, uploaded `app-debug.apk` 15.90 MB `https://github.com/RUNAYET-ISLAM-RISHAD/keybroad-/releases/download/v1.0.2/app-debug.apk` (verified 302 → S3, raw config 200). Repo made public. `update_config.json` now points to GitHub releases direct link. Workflow `.github/workflows/android-build.yml` created locally but push blocked by token lacking `workflow` scope — requires manual push via GitHub UI or PAT with `workflow` scope. |

| 2026-09-01 | **v1.1.0 - Smart Bengali Input System** | Bengali-first visual layout (5-row grouping with stable QWERTY logical IDs), Smart Conjunct Engine with dedicated য়ুত্ (join) key and Join Mode states (Normal/JoinPending/JoinActive), conjunct dictionary (core/data/conjuncts.json, 51 high-frequency conjuncts), Smart Kar System with কার popup key, context-aware suggestion bar (join suggestions + word prediction), grapheme-aware backspace, fixed critical typing bug (key_code 100/101 colliding with ASCII d/e on English layout, JNI now returns full composition text). All 145 Rust tests pass. versionCode 5. |
| 2026-09-01 | **v1.1.1 - Stable signing + PackageInstaller OTA** | Permanent release keystore (keybroad-release.jks) with GitHub Secrets, both debug/release signed with same key (SHA256 d715e5bf...87c8), PackageInstaller Session API for OTA (validates package_name, detects INSTALL_FAILED_UPDATE_INCOMPATIBLE), one-time bootstrap dialog. VersionCode 6, CI Build Android APK success. |
| 2026-09-01 | **v1.2.0 - Fix scrambled Bengali typing (JNI state)** | Fixed scrambled/reversed output: moved join/kar magic keycodes 100/101 → 1000/1001 to eliminate collision with Jatiya keys d(100)→ি and e(101)→ড. Added engine.process_char() + JNI nativeProcessChar/nativeApplySuggestion for direct Bengali char input (kar/suggestions now bypass layout lookup, engine single source of truth). Fixed JNI get_suggestions to use current_word, added detailed KeyboardEngine logging, fixed ViewModel text handling (text = engine.get_text()). Added 10-test jni_typing_tests.rs (k+h→খ, ami→আমি, join/kar, backspace grapheme). All 184 tests pass, assembleDebug success. |

# PROJECT_CONTEXT.md — Modern Bengali Keyboard

> Master context file for the "Modern Bengali Keyboard" project.
> Source: `Keyboard (1).md` — Product roadmap & Technical whitepaper v1.0

---

## 1. Project Identity

| Field | Value |
|---|---|
| **Project Name** | Modern Bengali Keyboard |
| **Codename** | KEYBROAD |
| **Domain** | Mobile Input / Language Technology |
| **Target Competitor** | Ridmik Keyboard (10+ year incumbent) |
| **Document Version** | 1.0 |
| **Status** | In Development — Core engine v0.3.0 with all layouts and CLI demo |

---

## 2. Vision

**"We are not building another keyboard — we are redefining the Bengali digital experience."**

The project aims to create a world-class Bengali keyboard that combines zero-latency input, 100% on-device privacy, and contextual AI suggestions. The goal is to make users who switch from Ridmik never want to go back.

---

## 3. Mission

Build a **real-time, privacy-first, AI-powered Bengali keyboard** that:

- Achieves near-zero latency on every keystroke
- Runs all AI inference on-device (Edge AI) — no data ever leaves the phone
- Provides contextual, personalized predictions that learn from user behavior
- Supports Bengali conjunct characters, phonetic input, and multiple layouts with perfect accuracy
- Syncs data across devices using CRDT-based conflict-free synchronization
- Works 90% offline with dictionary and AI models stored locally

---

## 4. Scope

### 4.1 MVP — Phase 1 (Months 1–4): "Nothing Missing, Nothing Extra"

Core layouts:
- Phonetic (English-to-Bangla transliteration)
- Jatiya (National)
- Probhat
- Unijoy
- English

Core features:
- Smart auto-correction with built-in dictionary
- Basic next-word suggestion (static dictionary-based)
- On-device voice typing (custom model or offline speech framework — NOT Google API)
- Multi-device cloud sync (backup & restore)
- Clipboard manager and text editing tools
- Basic themes (dark/light, custom background)
- 100% privacy policy — no unnecessary permissions

**MVP Success Criterion:** If a user can replace Ridmik with these features, the MVP is successful.

### 4.2 Phase 2 (Months 4–8): "Entering the AI Era"

- Contextual AI prediction (learns user typing patterns)
- Real-time grammar and spell checker
- In-keyboard translator (Bengali ↔ English)
- AI writing assistant (tone switching — Formal/Informal)
- Custom voice model for regional dialect pronunciation support
- Add-on support for Arabic and Chakma languages

### 4.3 Phase 3 (Months 8–12+): "Ecosystem & Revenue"

- **Theme & Sticker Marketplace:** Designers sell themes/stickers, company takes commission
- **User-Generated Stickers:** Create memes/stickers from personal photos
- **Digital Currency (Coins):** Integration with bKash/Nagad
- **B2B SDK:** Secure keyboard licensing for bank/corporate apps
- **Split-keyboard** for tablets/foldable phones

---

## 5. Requirements

### 5.1 Functional Requirements

| ID | Requirement | Phase |
|---|---|---|
| FR-01 | Support 5 keyboard layouts (Phonetic, Jatiya, Probhat, Unijoy, English) | MVP |
| FR-02 | Auto-correction with built-in Bengali dictionary | MVP |
| FR-03 | Next-word suggestion (basic N-gram) | MVP |
| FR-04 | On-device voice typing | MVP |
| FR-05 | Multi-device cloud sync (E2EE) | MVP |
| FR-06 | Clipboard manager | MVP |
| FR-07 | Dark/light theme with custom background | MVP |
| FR-08 | Contextual AI prediction (personalized) | Phase 2 |
| FR-09 | Grammar and spell checker | Phase 2 |
| FR-10 | Bengali ↔ English in-keyboard translator | Phase 2 |
| FR-11 | AI writing assistant with tone control | Phase 2 |
| FR-12 | Custom voice model for regional dialects | Phase 2 |
| FR-13 | Arabic and Chakma language support | Phase 2 |
| FR-14 | Theme/sticker marketplace (creator economy) | Phase 3 |
| FR-15 | User-generated stickers | Phase 3 |
| FR-16 | Digital currency (bKash/Nagad) integration | Phase 3 |
| FR-17 | B2B SDK for banking/corporate apps | Phase 3 |
| FR-18 | Split-keyboard for tablets/foldables | Phase 3 |

### 5.2 Non-Functional Requirements

| ID | Requirement | Target |
|---|---|---|
| NFR-01 | Input-to-render latency | < 10ms |
| NFR-02 | Key press → visual feedback | < 8ms |
| NFR-03 | Prediction calculation | < 5ms |
| NFR-04 | Gesture/slide recognition | < 5ms |
| NFR-05 | Memory allocation per keystroke | 0 (zero allocation) |
| NFR-06 | Background RAM usage (native core) | < 50MB |
| NFR-07 | Prediction accuracy (top-3) | > 90% |
| NFR-08 | Crash-free sessions | > 99.5% |
| NFR-09 | Data sync conflict rate | < 0.1% |
| NFR-10 | Voice typing WER (Bengali) | < 5% |
| NFR-11 | App size (Android APK) | < 40MB |
| NFR-12 | Rendering framerate | 60fps minimum (120fps target) |
| NFR-13 | On-device AI model size | 5–15MB (INT8 quantized) |
| NFR-14 | Dictionary size | 100,000+ words |
| NFR-15 | Offline feature availability | 90% without internet |
| NFR-16 | Voice commands supported | 50+ |

---

## 6. Architecture

### 6.1 High-Level Architecture: Event-Driven, Domain-Oriented, Layered & Reactive

```
┌─────────────────────────────────────────────┐
│              SERVICE LAYER                   │
│  Cloud Sync | Billing | Theme Store | API   │
│  (Stripe/bKash)                             │
├─────────────────────────────────────────────┤
│           PRESENTATION LAYER                │
│  Android: Jetpack Compose                    │
│  iOS: SwiftUI                               │
│  (ONLY touches user, never core engine)     │
├─────────────────────────────────────────────┤
│              DATA LAYER                     │
│  SQLCipher (Android) / Keychain+CoreData    │
│  (Themes, Stickers, Clipboard, Settings)    │
├─────────────────────────────────────────────┤
│              CORE LAYER                     │
│  Typing Logic | Layout Mapping | Dictionary │
│  AI Inference | Pure Kotlin/Pure Swift       │
│  (NO UI framework, NO database, NO network) │
└─────────────────────────────────────────────┘
```

### 6.2 Layer Descriptions

#### Core Layer ("The Brain")
- Pure functional and deterministic
- Typing logic, layout mapping, dictionary, AI inference
- **Language:** Pure Kotlin (Android) / Pure Swift (iOS) — or cross-platform via Rust/C++ (JNI/FFI)
- Zero UI framework dependency — portable to Web/Desktop
- Composition buffer uses Arena allocator (no per-keystroke heap allocation)
- Key mapping stored as JSON, loaded at runtime
- Single-pass processing; no backtracking loops
- Bengali conjunct characters (ক্ষ, জ্ঞ, চ্ছ) stored in optimized Trie data structure
- Core signature: `(current_state, keystroke) → (new_state, output_text)`

#### Data Layer ("The Memory")
- Local storage: SQLCipher (Android) / Keychain + CoreData (iOS)
- CRDT-based offline-first sync using RGA (Replicated Growable Array)
- All local data encrypted with AES-256-GCM
- No plain text ever reaches the server — only encrypted blobs

#### Presentation Layer ("The Face")
- Android: Jetpack Compose (declarative UI)
- iOS: SwiftUI (declarative UI)
- This layer only communicates with the user; never touches the core engine directly
- State management via Kotlin Flow (Android) / Combine (iOS)
- Reactive backpressure system: if user types too fast, UI drops frames but core engine keeps processing

#### Service Layer ("The Backend")
- Cloud sync (Firebase Firestore or custom Node.js + WebSocket)
- Premium subscription (Stripe / bKash)
- Theme store (marketplace)
- API endpoints only — **keyboard typing data NEVER goes to the server**
- Keyboard process runs in separate OS process with no internet permission (Android: `android:process=":keyboard"`)

### 6.3 State Management

- Keyboard state (current mode, active layout, shift, caps lock, prediction states) flows via immutable state streams
- Immer-like immutable state pattern
- Kotlin Flow (Android) / Combine (iOS)
- Reactive backpressure: if user types too fast, UI drops frames but core engine continues processing without delay

---

## 7. Technology Decisions

### 7.1 Recommended Stack (Baseline)

| Component | Recommendation | Alternative | Rationale |
|---|---|---|---|
| Core Engine | **Rust/C++** | Kotlin Native | Performance + portability across platforms |
| Android UI | **Jetpack Compose** | XML Views | Modern, declarative |
| iOS UI | **SwiftUI** | UIKit | Modern, declarative |
| Local DB | **SQLCipher** | Realm | Encryption built-in |
| Cloud Sync | **Firebase Firestore** | Supabase | Real-time + CRDT support potential |
| On-Device AI | **TFLite / CoreML** | OnnxRuntime | Ecosystem maturity |
| Backend API | **Node.js + Fastify** | Python FastAPI | Non-blocking I/O |
| CI/CD | **GitHub Actions** | Bitrise | Free + scalable |
| Code Linting | **Detekt** (Kotlin) / **SwiftLint** (Swift) | — | Code quality |
| AI Model Format | **INT8 Quantized** | FP16 | Smaller size (5-15MB), faster inference |
| Serialization | **serde + serde_json** | bincode | Human-readable JSON for layouts |

### 7.2 Technology Decisions (Finalized in ARCHITECTURE.md)

| # | Component | Decision | Rationale |
|---|---|---|---|
| 1 | Core Engine | **Rust** (via JNI/FFI) | Zero-cost abstractions, no GC, cross-platform, WASM target |
| 2 | Cloud Backend | **Custom Node.js + Fastify** | Full CRDT control, WebSocket support, no vendor lock-in |
| 3 | AI Model (MVP) | **N-gram (Trigram)** | <1MB, <1ms, 75% accuracy — sufficient for cold start |
| 4 | AI Model (Phase 2) | **Quantized DistilBERT-Bengali** | 8-12MB, <5ms, >90% accuracy, INT8 quantized |
| 5 | Local Database | **SQLCipher** | AES-256 encryption, FTS5 for dictionary, cross-platform |
| 6 | CRDT Implementation | **Custom Rust crate** | No suitable existing library; RGA + LWW-Register |
| 7 | Android UI | **Jetpack Compose** + custom Canvas fallback | Declarative; Canvas for <8ms critical path |
| 8 | iOS UI | **SwiftUI** + custom CALayer fallback | Declarative; CALayer for <8ms critical path |
| 9 | Process Isolation | **Separate OS process, no internet** | Android: `android:process=":keyboard"` |
| 10 | Testing | **Property-based + Golden files + Benchmarks** | 1M+ random inputs, 10,000+ Bengali words |

*See `ARCHITECTURE.md` for full justifications and implementation details.*

---

## 8. Design Principles

1. **Engineering First:** "We are not cloning Ridmik. We are building a Language Intelligence Layer."
2. **Performance is Non-Negotiable:** Every decision must prove it meets the latency budget. No feature ships without a benchmark.
3. **Privacy by Default:** 100% on-device processing. No data collection. No tracking. "Type anything; your data never leaves your phone."
4. **Modularity:** Core engine and UI layer are completely separate. New layouts/features can be added without touching the core.
5. **Offline-First:** 90% of features work without internet. Dictionary, data, and AI models live on the device.
6. **Team Autonomy:** The whitepaper is a canvas, not a specification. Teams should challenge assumptions and innovate beyond suggestions.
7. **Test-Driven:** Property-based testing, golden file regression, latency profiling — "We test 1M+ keystroke combinations before every release."
8. **Determinism:** Pure functional core means same input always produces same output. Makes testing trivial and behavior predictable.

---

## 9. Business Logic

### 9.1 Revenue Model
- **Freemium:** Basic keyboard free; premium features behind subscription
- **Theme/Sticker Marketplace:** Designers publish and sell; company takes commission
- **Digital Currency (Coins):** In-app currency for purchasing themes/stickers, integrated with bKash/Nagad
- **B2B SDK Licensing:** Secure keyboard SDK for banks and corporate apps

### 9.2 User Acquisition Strategy
- Position against Ridmik's weaknesses: no AI, no cloud sync, no privacy-first architecture
- Marketing weapon: "100% privacy — no data ever leaves your phone"
- Target: 10,000 beta users before public release

### 9.3 Creator Economy
- Phase 3 introduces marketplace where designers earn from themes/stickers
- User-generated sticker creation from personal photos
- Company commission model on marketplace transactions

---

## 10. Security Rules

### 10.1 Zero-Trust Architecture
- Every feature treated with suspicion
- Keyboard runs in a **sandboxed environment**
- Even if one layer fails, others protect

### 10.2 Hardware-Backed Encryption
- **Android:** Keystore + EncryptedSharedPreferences
- **iOS:** Keychain + Secure Enclave
- Dictionary and clipboard data encrypted with **AES-256-GCM**

### 10.3 Incognito Mode (Automatic)
- Detect password/OTP fields via accessibility APIs (or OS hints)
- When detected: core engine switches to "secure state"
- In secure state: no dictionary learning, no clipboard history, no prediction
- Data flushed from RAM after input

### 10.4 Anti-Tampering
- Android: ProGuard + R8 with custom rules + native code encryption (hiding string constants)
- iOS: Swift obfuscation + jailbreak detection
- SSL Pinning on all network calls (prevent MITM attacks)

### 10.5 Network Isolation
- Keyboard process runs in **separate OS process** (Android: `android:process=":keyboard"`)
- This process has **no internet permission** by default
- Only the companion "Settings/Store" app has network access

### 10.6 Threat Model
- Malicious apps reading keyboard memory
- Network sniffing during cloud sync
- Reverse-engineering of APK to extract dictionary data

---

## 11. Development Rules

1. **No Blind Coding:** Every sprint must answer: "Did we make the fastest and safest decision for the user?" — only proceed if the answer is "yes."
2. **Test-First:** Property-based testing (QuickCheck/Proptest) for the engine. Golden file regression testing for Bengali word output.
3. **Benchmark Everything:** No feature ships without a benchmark proving it's better than baseline.
4. **Performance Budget Enforcement:**
   - Key press → visual feedback: < 8ms
   - Prediction calculation: < 5ms
   - Gesture recognition: < 5ms
   - Memory allocation: 0 per keystroke
5. **Latency Profiling:** Built-in profiler (Systrace/Xcode Instruments) measuring every key-press "Time-to-Render" — must stay within 16ms.
6. **CI/CD Mandatory:** GitHub Actions with automated performance regression tests (baseline on Pixel 5 / iPhone 11).
7. **Code Quality:** Detekt (Kotlin) + SwiftLint (Swift) enforced in CI.
8. **10,000+ Golden Files:** Precomputed expected output for common Bengali words from Wiktionary — versioned JSON, compared on every CI run.
9. **Voice Testing:** 1,000+ recorded Bangla phrases (including regional dialects) — WER < 5%.
10. **Challenge Assumptions:** Team must question every suggestion in the whitepaper and prove their own decisions with data.

---

## 12. Known Constraints

1. **Mobile-First:** Primary targets are Android and iOS. Web/Desktop are future considerations.
2. **Embedded System Constraints:** RAM and CPU are limited — algorithms must be memory-bound and CPU-bound optimized.
3. **Real-Time System:** Keyboard is not a CRUD app — it's a real-time system requiring timing and determinism.
4. **Bengali Complexity:** Bengali script has complex conjunct characters (যুক্তাক্ষর) requiring optimized Trie structures.
5. **No Google API for Voice:** Voice typing must use custom model or offline framework — not Google's cloud API.
6. **Privacy Regulation:** All typing data must remain on-device. No exceptions.
7. **App Size:** Android APK must be under 40MB.
8. **AI Model Size:** On-device models must be 5–15MB (INT8 quantized) to fit within memory constraints.
9. **Team Needs:** Requires expertise in Rust/C++ (core engine), Kotlin/Swift (platform), ML/AI (on-device models), and distributed systems (CRDT sync).

---

## 13. Important Decisions

| # | Decision | Status | Notes |
|---|---|---|---|
| 1 | Native development over cross-platform (Flutter/RN) | **Decided** | Jetpack Compose + SwiftUI with Canvas/CALayer fallback |
| 2 | Rust for cross-platform core engine | **Decided** | Via JNI (Android) / FFI (iOS); WASM target for future web |
| 3 | CRDT for cloud sync (RGA-based) | **Decided** | Custom Rust crate with RGA + LWW-Register |
| 4 | On-device AI with Federated Learning | **Decided** | Privacy-first, only weights leave device |
| 5 | Zero-allocation policy for keystroke handling | **Decided** | Arena allocator / object pooling |
| 6 | Property-based testing for engine | **Decided** | Random input stress testing |
| 7 | Golden file regression testing | **Decided** | 10,000+ Bengali words |
| 8 | Network isolation for keyboard process | **Decided** | Separate OS process, no internet permission |
| 9 | Custom Node.js + Fastify backend | **Decided** | Full CRDT control, no Firebase vendor lock-in |
| 10 | SQLCipher for local database | **Decided** | AES-256, FTS5, cross-platform |
| 11 | N-gram for MVP, Transformer for Phase 2 | **Decided** | Hybrid approach: fast cold start → personalized AI |
| 12 | 240Hz touch polling for gesture detection | Under consideration | Stylus-friendly; needs device support validation |

---

## 14. Dependencies

### 14.1 External Dependencies
- **Firebase:** Cloud sync, Crashlytics, Analytics (if chosen)
- **Stripe / bKash / Nagad:** Payment processing
- **Google Play Store / Apple App Store:** Distribution
- **TFLite / CoreML:** On-device AI inference
- **SQLCipher:** Encrypted local database
- **Wiktionary:** Source for Bengali golden file test data

### 14.2 Internal Dependencies
- Core engine must be built before any UI work
- Data layer (CRDT sync) requires core engine data structures
- AI models require training data (user typing patterns — collected via federated learning only)
- Voice model requires recorded Bengali phrases (1,000+ including regional dialects)
- Marketplace (Phase 3) requires subscription/billing system

### 14.3 Platform Dependencies
- **Android:** Keystore, EncryptedSharedPreferences, Accessibility API (for password field detection)
- **iOS:** Keychain, Secure Enclave, CoreML, Accessibility API
- **Both:** OS-provided hints for secure input field detection

---

## 15. Current Architectural State

**Status: Core engine with dictionary and auto-correction implemented. Android app skeleton with JNI integration is complete.**

- Architecture document: `ARCHITECTURE.md` — complete
- Technology decisions: Finalized (see Section 7.2)
- Core engine: Rust library with layout loading, conjunct handling, and dictionary
  - `types.rs`: Layout, KeyMapping, KeyEvent, OutputAction, EngineState, CandidateWord structs
  - `engine.rs`: BengaliEngine with process_key, conjunct handling, dictionary integration
  - `layout.rs`: Layout loading from embedded JSON, HashMap-based key mapping
  - `conjunct.rs`: ConjunctTable with consonant classification and hasanta conjunct lookup
  - `dictionary.rs`: Trie-based dictionary with prefix matching and Levenshtein auto-correction
- Layout JSON files: Phonetic (89 keys) and English (89 keys) implemented
- Dictionary: 500+ Bengali words loaded via include_str! (core/data/words.txt)
- Tests: 103/103 passing (42 unit + 29 dictionary + 32 engine integration)
- Dictionary module: Trie, Levenshtein distance, prefix matching, auto-correction
- This is the **Foundation Phase** (Weeks 3–8 of roadmap). Android integration is now part of the foundation.

---

## 16. Engineering KPIs

| Metric | Target | Measurement |
|---|---|---|
| Input-to-render latency | < 10ms | Custom profiler on-device |
| Prediction accuracy (top-3) | > 90% | Offline evaluation dataset |
| Crash-free sessions | > 99.5% | Firebase Crashlytics |
| Data sync conflict rate | < 0.1% | Server-side analytics |
| Voice typing WER (Bengali) | < 5% | Benchmark test suite |
| App size (Android) | < 40MB | APK analyzer |

---

## 17. Engineering Evolution Roadmap

| Phase | Focus | Key Deliverable | Status |
|---|---|---|---|
| Phase 1 (Focus) | Runtime Logic + State Machine | Working typing engine with 3 layouts + dictionary | **In Progress** |
| Phase 2 (Scale) | Cross-Platform Core + CRDT Sync | Rust engine, offline-first sync | **Planned** |
| Phase 3 (Intelligence) | Federated Learning + Contextual AI | On-device personalized predictions | **Planned** |
| Phase 4 (Ecosystem) | Publisher Marketplace + B2B SDK | Creator economy, bank licensing | **Planned** |

*Architecture designed 2026-08-31. Core engine with conjunct handling and dictionary module implemented. 103 tests passing.*

---

*This file serves as the single source of truth for the Modern Bengali Keyboard project context. Update as decisions are made and architecture evolves.*

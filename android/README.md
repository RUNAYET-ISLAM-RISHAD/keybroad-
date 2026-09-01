# Keybroad Android App

Android frontend for the Modern Bengali Keyboard, integrating the Rust core engine via JNI.

## Prerequisites

- Android Studio (latest)
- Android SDK (API 26+)
- Rust (stable) with cargo-ndk: `cargo install cargo-ndk`
- Android NDK (r25+)

## Build Instructions

1. Build the Rust core for Android:
   ```
   cd core
   cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 build --release
   ```

2. Copy the compiled `.so` libraries to `android/app/src/main/jniLibs/`:
   ```
   cp -r target/release/*.so ../android/app/src/main/jniLibs/
   ```

3. Build the Android app:
   ```
   cd android
   ./gradlew assembleDebug
   ```

4. Install on emulator/device:
   ```
   ./gradlew installDebug
   ```

## Running

Open the app from the launcher. Type using the on-screen keyboard. Switch layouts using the layout buttons. Suggestions appear above the keyboard.
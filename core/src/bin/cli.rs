/// CLI demo tool for testing the Bengali keyboard engine interactively.
///
/// This binary provides a simple REPL (read-eval-print-loop) that allows
/// interactive typing to test the engine's behavior.
///
/// Usage:
///   keybroad_cli [layout_name]
///
/// Commands:
///   /layout <name>  - Switch to a different layout (phonetic, jatiya, probhat, unijoy, english)
///   /reset          - Reset the engine state
///   /incognito      - Toggle incognito mode
///   /suggest        - Show suggestions for current word
///   /word           - Show current word being typed
///   /history        - Show word history
///   /exit           - Exit the program
///
/// Input:
///   - Regular characters are processed as keystrokes
///   - Enter (newline) commits the current word
///   - Backspace character (0x08 or \u{8}) simulates backspace
///   - Space character commits the current word and adds a space

use std::io::{self, Write};

use keybroad_core::{BengaliEngine, LayoutType, KeyEvent, OutputAction};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Determine initial layout from command line args
    let initial_layout = if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "phonetic" | "p" => LayoutType::Phonetic,
            "jatiya" | "j" | "national" => LayoutType::Jatiya,
            "probhat" | "pr" => LayoutType::Probhat,
            "unijoy" | "u" => LayoutType::Unijoy,
            "english" | "e" | "en" => LayoutType::English,
            _ => {
                eprintln!("Unknown layout '{}'. Using phonetic.", args[1]);
                LayoutType::Phonetic
            }
        }
    } else {
        LayoutType::Phonetic
    };

    let mut engine = BengaliEngine::new(initial_layout);

    println!("=== Bengali Keyboard CLI Demo ===");
    println!("Layout: {:?}", engine.get_state().layout);
    println!("Commands: /layout <name>, /reset, /incognito, /suggest, /word, /history, /exit");
    println!("Type characters to test. Press Enter to commit word. Use Ctrl+C to exit.");
    println!("================================");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let input = input.trim_end_matches('\n').trim_end_matches('\r');

                // Handle commands
                if input.starts_with('/') {
                    handle_command(&mut engine, input);
                    continue;
                }

                // Process each character as a keystroke
                process_input(&mut engine, input);
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    println!("\nGoodbye!");
}

/// Handle a command starting with '/'
fn handle_command(engine: &mut BengaliEngine, command: &str) {
    let parts: Vec<&str> = command.splitn(2, ' ').collect();
    let cmd = parts[0].to_lowercase();

    match cmd.as_str() {
        "/layout" => {
            if let Some(layout_name) = parts.get(1) {
                let layout = match layout_name.to_lowercase().as_str() {
                    "phonetic" | "p" => Some(LayoutType::Phonetic),
                    "jatiya" | "j" | "national" => Some(LayoutType::Jatiya),
                    "probhat" | "pr" => Some(LayoutType::Probhat),
                    "unijoy" | "u" => Some(LayoutType::Unijoy),
                    "english" | "e" | "en" => Some(LayoutType::English),
                    _ => None,
                };

                if let Some(layout) = layout {
                    engine.set_layout(layout);
                    println!("Layout switched to: {:?}", layout);
                } else {
                    println!("Unknown layout. Available: phonetic, jatiya, probhat, unijoy, english");
                }
            } else {
                println!("Current layout: {:?}", engine.get_state().layout);
                println!("Available layouts: phonetic, jatiya, probhat, unijoy, english");
            }
        }
        "/reset" => {
            engine.reset();
            println!("Engine state reset.");
        }
        "/incognito" => {
            let current = engine.is_incognito();
            engine.set_incognito(!current);
            println!("Incognito mode: {}", if current { "OFF" } else { "ON" });
        }
        "/suggest" => {
            let suggestions = engine.get_suggestions(&engine.get_state().current_word);
            if suggestions.is_empty() {
                println!("No suggestions for current word.");
            } else {
                println!("Suggestions:");
                for (i, s) in suggestions.iter().enumerate() {
                    println!("  {}. {} (score: {:.2}, source: {:?})", i + 1, s.word, s.score, s.source);
                }
            }
        }
        "/word" => {
            let word = &engine.get_state().current_word;
            if word.is_empty() {
                println!("No word being typed.");
            } else {
                println!("Current word: {}", word);
            }
        }
        "/history" => {
            let history = &engine.get_state().history;
            if history.is_empty() {
                println!("No word history.");
            } else {
                println!("Word history (most recent first):");
                for (i, w) in history.iter().enumerate() {
                    println!("  {}. {}", i + 1, w);
                }
            }
        }
        "/exit" | "/quit" | "/q" => {
            println!("Goodbye!");
            std::process::exit(0);
        }
        "/help" | "/?" => {
            println!("Commands:");
            println!("  /layout [name]  - Switch layout or show current");
            println!("  /reset          - Reset engine state");
            println!("  /incognito      - Toggle incognito mode");
            println!("  /suggest        - Show suggestions for current word");
            println!("  /word           - Show current word being typed");
            println!("  /history        - Show word history");
            println!("  /exit           - Exit the program");
        }
        _ => {
            println!("Unknown command: {}. Type /help for help.", cmd);
        }
    }
}

/// Process a line of input as keystrokes
fn process_input(engine: &mut BengaliEngine, input: &str) {
    for ch in input.chars() {
        // Handle backspace (0x08 or \u{8})
        if ch == '\u{8}' || ch == '\u{7f}' {
            let event = KeyEvent::down(67, 0); // key_code 67 = backspace
            let actions = engine.process_key(event).unwrap_or_default();
            display_actions(&actions);
            continue;
        }

        // Handle enter/newline - finalize word
        if ch == '\n' || ch == '\r' {
            engine.finalize_word();
            continue;
        }

        // Regular character - process as keystroke
        let event = KeyEvent::down(ch as u32, ch as u32);
        let actions = engine.process_key(event).unwrap_or_default();
        display_actions(&actions);
    }

    // Show current state after processing
    display_state(engine);
}

/// Display the output actions from a keystroke
fn display_actions(actions: &[OutputAction]) {
    for action in actions {
        match action {
            OutputAction::CommitText(text) => {
                print!("{}", text);
                io::stdout().flush().unwrap();
            }
            OutputAction::Backspace(count) => {
                // Print backspaces to terminal
                for _ in 0..*count {
                    print!("\u{8}");
                }
                io::stdout().flush().unwrap();
            }
            OutputAction::UpdateCandidates(candidates) => {
                if !candidates.is_empty() {
                    // Show first 3 candidates
                    let display: Vec<&str> = candidates.iter().take(3).map(|c| c.word.as_str()).collect();
                    print!(" [{}]", display.join(", "));
                    io::stdout().flush().unwrap();
                }
            }
            _ => {}
        }
    }
}

/// Display the current engine state
fn display_state(engine: &BengaliEngine) {
    let state = engine.get_state();

    // Show current word if not empty
    if !state.current_word.is_empty() {
        print!(" (word: {})", state.current_word);
        io::stdout().flush().unwrap();
    }

    // Show composition buffer if not empty
    if !state.composition_buffer.is_empty() {
        let composition: String = state.composition_buffer
            .iter()
            .filter_map(|g| char::from_u32(g.unicode))
            .collect();
        print!(" [comp: {}]", composition);
        io::stdout().flush().unwrap();
    }

    println!();
}
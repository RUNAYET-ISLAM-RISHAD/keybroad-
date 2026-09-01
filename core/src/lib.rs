pub mod conjunct;
pub mod dictionary;
pub mod engine;
pub mod layout;
pub mod ngram;
pub mod phonetic;
pub mod types;

pub use conjunct::ConjunctTable;
pub use dictionary::{levenshtein_distance, Dictionary};
pub use engine::BengaliEngine;
pub use layout::load_layout;
pub use ngram::NgramModel;
pub use types::*;

#[cfg(target_os = "android")]
pub mod android_jni;

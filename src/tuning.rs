//! Ranking constants kept separate so the golden corpus can tune behaviour
//! without changing retrieval code.
pub const W_TOOL: f32 = 3.0;
pub const W_ACTION: f32 = 2.0;
pub const W_OBJECT: f32 = 1.5;
pub const W_TRIGGER: f32 = 3.0;
pub const W_OPTION: f32 = 0.5;
pub const W_COVERAGE: f32 = 2.0;
pub const W_LEFTOVER: f32 = 1.5;
/// A complete section alias is authored vocabulary, not an incidental word
/// overlap from a neighbouring command or note.
pub const W_EXACT_ALIAS: f32 = 8.0;
pub const OPT_CAP: usize = 4;
pub const T_MIN: f32 = 4.0;
pub const T_MARGIN: f32 = 1.5;
pub const MAX_CHOICES: usize = 4;
/// Keep a focused answer short even if a future catalog entry has many
/// variants or optional flags.
pub const MAX_VARIANTS: usize = 3;
pub const MAX_OPTION_HINTS: usize = 3;

use std::env;

#[derive(Debug, Clone)]
pub struct TerminalCaps {
    pub true_color: bool,
    pub kitty_graphics: bool,
    pub osc8_hyperlinks: bool,
    pub is_ghostty: bool,
}

impl TerminalCaps {
    /// Detect terminal capabilities from environment variables.
    pub fn detect() -> Self {
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let colorterm = env::var("COLORTERM").unwrap_or_default();
        let is_ghostty = term_program == "ghostty";

        Self {
            true_color: colorterm == "truecolor" || colorterm == "24bit" || is_ghostty,
            kitty_graphics: is_ghostty || term_program == "kitty",
            osc8_hyperlinks: is_ghostty || term_program == "kitty" || term_program == "WezTerm",
            is_ghostty,
        }
    }
}

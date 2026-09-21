//! Startup banner product-discovery lines (Issue 26072710436325).
//!
//! Command tokens (`roamari`, `ah`) are painted **cyan** in the banner
//! to match Nu default `color_config.shape_external` (first external command
//! on the prompt), not green.

/// Unstyled product-hint lines for Full/Short banners (after Shift+Tab / mode hint).
pub const PRODUCT_HINT_LINES: &[&str] = &[
    "Type roamari to browse the web.",
    "Type ah then Tab to see more commands.",
];

/// Explicit AI placeholder. Not a working agent.
pub const AI_HINT: &str = "The AI mode is unfinished; input is never executed.";

/// Same SGR as the live `[ai]` prompt label (`nu_cli` `AI_MODE_SGR`).
pub const AI_LABEL_SGR: &str = "\x1b[35m";

/// Banner line with `AI` painted like `[ai]`; `fg` restored after the word.
pub fn ai_hint_line(fg: &str, reset: &str) -> String {
    let painted = AI_HINT.replace(" AI ", &format!(" {AI_LABEL_SGR}AI{fg} "));
    format!("{fg}{painted}{reset}")
}

/// ANSI SGR for Nu default `shape_external: cyan` (`Color::Cyan.normal()`).
pub const SHAPE_EXTERNAL_ANSI: &str = "\x1b[36m";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_hints_mention_roamari_and_tab() {
        let joined = PRODUCT_HINT_LINES.join("\n");
        assert!(joined.contains("roamari"));
        assert!(joined.contains("browse the web"));
        assert!(!joined.contains("ahhelp"));
        assert!(joined.contains("ah then Tab") || joined.contains("Tab"));
        assert!(joined.contains("more commands"));
        assert_eq!(PRODUCT_HINT_LINES.len(), 2);
    }

    #[test]
    fn ai_hint_does_not_claim_an_agent() {
        assert!(AI_HINT.contains("never executed"));
        assert!(AI_HINT.to_ascii_lowercase().contains("unfinished"));
        assert!(!AI_HINT.to_ascii_lowercase().contains("agent"));
    }

    #[test]
    fn ai_word_in_banner_uses_ai_label_magenta() {
        let line = ai_hint_line("\x1b[37m", "\x1b[0m");
        assert!(
            line.contains(&format!("{AI_LABEL_SGR}AI\x1b[37m")),
            "{line:?}"
        );
        assert_eq!(AI_LABEL_SGR, "\x1b[35m");
        assert!(line.contains("unfinished"));
    }

    #[test]
    fn shape_external_ansi_is_cyan_not_green() {
        // nu-ansi-term Color::Cyan.normal() → SGR 36
        assert_eq!(SHAPE_EXTERNAL_ANSI, "\x1b[36m");
        assert_ne!(SHAPE_EXTERNAL_ANSI, "\x1b[32m");
    }
}

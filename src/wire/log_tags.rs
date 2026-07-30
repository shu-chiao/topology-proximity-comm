//! Colored log prefixes for router and peer watch lines.

use std::io::{self, IsTerminal};

const RESET: &str = "\x1b[0m";

#[derive(Clone, Copy)]
pub enum Watch {
    Router,
    Peer,
}

#[derive(Clone, Copy)]
pub enum ActionLog {
    Goal,
    Result,
    Feedback,
}

impl ActionLog {
    const fn ansi(self) -> &'static str {
        match self {
            ActionLog::Goal => "\x1b[1;32m",     // bold green
            ActionLog::Result => "\x1b[1;34m",   // bold blue
            ActionLog::Feedback => "\x1b[1;33m", // bold yellow
        }
    }

    const fn label(self) -> &'static str {
        match self {
            ActionLog::Goal => "(Goal)",
            ActionLog::Result => "(Result)",
            ActionLog::Feedback => "(Feedback)",
        }
    }
}

impl Watch {
    const fn ansi(self) -> &'static str {
        match self {
            Watch::Router => "\x1b[1;36m", // bold cyan
            Watch::Peer => "\x1b[1;35m",   // bold magenta
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Watch::Router => "(router watch)",
            Watch::Peer => "(peer watch)",
        }
    }
}

fn want_color(for_stdout: bool) -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if for_stdout {
        io::stdout().is_terminal()
    } else {
        io::stderr().is_terminal()
    }
}

/// Format a watch tag, with color when stdout/stderr is a TTY.
pub fn format_tag(w: Watch, for_stdout: bool) -> String {
    if want_color(for_stdout) {
        format!("{}{}{}", w.ansi(), w.label(), RESET)
    } else {
        w.label().to_string()
    }
}

/// Format an action client log tag (`(Goal)`, `(Result)`, `(Feedback)`).
pub fn format_action_tag(phase: ActionLog, for_stdout: bool) -> String {
    if want_color(for_stdout) {
        format!("{}{}{}", phase.ansi(), phase.label(), RESET)
    } else {
        phase.label().to_string()
    }
}

/// Format a bridge mode label, with color when enabled.
pub fn format_bridge_mode_tag(label: &str, for_stdout: bool) -> String {
    const ORANGE: &str = "\x1b[38;5;208m";
    if want_color(for_stdout) {
        format!("{}{}{}", ORANGE, label, RESET)
    } else {
        label.to_string()
    }
}

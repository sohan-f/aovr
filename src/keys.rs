pub struct Hint {
    pub key: &'static str,
    pub desc: &'static str,
}

const fn hint(key: &'static str, desc: &'static str) -> Hint {
    Hint { key, desc }
}

pub const TARGETS_NARROW: &[Hint] = &[
    hint("j/k", "move"),
    hint("r", "refresh"),
    hint("⏎", "open"),
    hint("/", "search"),
    hint("q", "quit"),
];

pub const TARGETS_WIDE: &[Hint] = &[
    hint("j/k", "move"),
    hint("⏎", "open"),
    hint("r", "refresh"),
    hint("/", "search"),
    hint("a", "about"),
    hint("q", "quit"),
];

pub const DETAIL_NARROW: &[Hint] = &[
    hint("j/k", "move"),
    hint("Spc", "toggle"),
    hint("/", "search"),
    hint("Esc", "back"),
];

pub const DETAIL_WIDE: &[Hint] = &[
    hint("j/k", "move"),
    hint("Spc", "toggle"),
    hint("⏎", "apply+back"),
    hint("/", "search"),
    hint("Esc", "back"),
    hint("q", "quit"),
];

pub const ABOUT: &[Hint] = &[
    hint("j/k", "scroll"),
    hint("g/G", "top/bottom"),
    hint("a / Esc", "back"),
    hint("q", "quit"),
];

pub const SEARCHING: &[Hint] = &[hint("⏎", "apply"), hint("esc", "cancel")];

pub const EMPTY: &[Hint] = &[hint("esc", "clear"), hint("q", "quit")];

pub const REFERENCE: &[Hint] = &[
    hint("j / k  ↑ / ↓", "move cursor / scroll"),
    hint("g / G", "jump to first / last  (Home / End)"),
    hint("PgUp / PgDn", "page up / down  (Ctrl-u / Ctrl-d: half)"),
    hint("h / l  ← / →", "go back / open"),
    hint("Space", "toggle overlay state"),
    hint("Enter", "open detail / apply changes"),
    hint("r", "refresh overlay list"),
    hint("/", "smart-case filter (click bar or press /)"),
    hint("Esc / a", "clear search / go back"),
    hint("q", "quit AOVR"),
];

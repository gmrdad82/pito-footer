use ratatui::style::{Modifier, Style};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Styles {
    pub accent: Style,
    pub muted: Style,
    pub alert: Style,
    pub ink: Style,
    pub rule: Style,
}

impl Styles {
    pub const fn new() -> Self {
        Styles {
            accent: Style::new(),
            muted: Style::new(),
            alert: Style::new(),
            ink: Style::new(),
            rule: Style::new(),
        }
    }

    pub const fn accent(mut self, style: Style) -> Self {
        self.accent = style;
        self
    }

    pub const fn muted(mut self, style: Style) -> Self {
        self.muted = style;
        self
    }

    pub const fn alert(mut self, style: Style) -> Self {
        self.alert = style;
        self
    }

    pub const fn ink(mut self, style: Style) -> Self {
        self.ink = style;
        self
    }

    pub const fn rule(mut self, style: Style) -> Self {
        self.rule = style;
        self
    }

    pub(crate) fn lit(&self) -> Style {
        self.accent.add_modifier(Modifier::BOLD)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tone {
    Ink,
    #[default]
    Muted,
    Accent,
    Alert,
}

impl Tone {
    pub(crate) fn style(self, styles: &Styles) -> Style {
        match self {
            Tone::Ink => styles.ink,
            Tone::Muted => styles.muted,
            Tone::Accent => styles.lit(),
            Tone::Alert => styles.alert,
        }
    }
}

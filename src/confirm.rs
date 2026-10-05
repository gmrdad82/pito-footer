use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    widgets::Widget,
};

use crate::key::Key;
use crate::styles::{Styles, Tone};
use crate::text::{self, Pen};

const MARK: &str = "▸ ";
const BEFORE: u16 = 2;
const BETWEEN: u16 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmKeys {
    pub yes: &'static [Key],
    pub no: &'static [Key],
    pub choose: bool,
}

impl ConfirmKeys {
    pub const HEY: ConfirmKeys = ConfirmKeys {
        yes: &[Key::Char('y'), Key::Char('Y')],
        no: &[Key::Char('n'), Key::Char('N'), Key::Esc],
        choose: true,
    };

    pub const ENTER: ConfirmKeys = ConfirmKeys {
        yes: &[Key::Enter],
        no: &[Key::Esc],
        choose: false,
    };
}

impl Default for ConfirmKeys {
    fn default() -> Self {
        ConfirmKeys::HEY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confirm {
    yes: bool,
    keys: ConfirmKeys,
}

impl Default for Confirm {
    fn default() -> Self {
        Confirm::new()
    }
}

impl Confirm {
    pub const fn new() -> Self {
        Confirm {
            yes: false,
            keys: ConfirmKeys::HEY,
        }
    }

    pub const fn keys(mut self, keys: ConfirmKeys) -> Self {
        self.keys = keys;
        self
    }

    pub const fn start(mut self, answer: Answer) -> Self {
        self.yes = matches!(answer, Answer::Yes);
        self
    }

    pub fn yes(&self) -> bool {
        self.yes
    }

    pub fn choosing(&self) -> bool {
        self.keys.choose
    }

    pub fn key(&mut self, key: Key) -> Option<Answer> {
        if self.keys.yes.contains(&key) {
            return Some(Answer::Yes);
        }
        if self.keys.no.contains(&key) {
            return Some(Answer::No);
        }
        if !self.keys.choose {
            return None;
        }
        match key {
            Key::Enter if self.yes => Some(Answer::Yes),
            Key::Enter => Some(Answer::No),
            Key::Left | Key::Right | Key::Tab | Key::BackTab | Key::Char('h' | 'l') => {
                self.yes = !self.yes;
                None
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Words<'a> {
    pub yes: &'a str,
    pub no: &'a str,
    pub hint: Option<&'a str>,
}

impl<'a> Words<'a> {
    pub const fn new(yes: &'a str, no: &'a str) -> Self {
        Words {
            yes,
            no,
            hint: None,
        }
    }

    pub const fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmBar<'a> {
    question: &'a str,
    confirm: Confirm,
    words: Words<'a>,
    rule: bool,
    inline: bool,
    tone: Tone,
    styles: Styles,
}

impl<'a> ConfirmBar<'a> {
    pub fn new(question: &'a str, confirm: &Confirm, words: Words<'a>) -> Self {
        ConfirmBar {
            question,
            confirm: *confirm,
            words,
            rule: true,
            inline: false,
            tone: Tone::Ink,
            styles: Styles::new(),
        }
    }

    pub fn rule(mut self, rule: bool) -> Self {
        self.rule = rule;
        self
    }

    pub fn inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn styles(mut self, styles: Styles) -> Self {
        self.styles = styles;
        self
    }

    pub fn height(&self) -> u16 {
        let hint = !self.inline && self.words.hint.is_some();
        1 + u16::from(self.rule) + u16::from(hint)
    }

    fn choices_width(&self) -> u16 {
        if !self.confirm.keys.choose {
            return 0;
        }
        BEFORE
            + text::width(MARK)
            + text::width(self.words.yes)
            + BETWEEN
            + text::width(self.words.no)
    }

    fn question_style(&self) -> Style {
        self.tone.style(&self.styles).add_modifier(Modifier::BOLD)
    }

    fn draw_choices(&self, pen: &mut Pen) {
        if !self.confirm.keys.choose {
            return;
        }
        pen.skip(BEFORE);
        let lit = self.styles.lit();
        let muted = self.styles.muted;
        if self.confirm.yes {
            pen.put(MARK, lit);
            pen.put(self.words.yes, lit);
            pen.skip(BETWEEN);
            pen.put(self.words.no, muted);
        } else {
            pen.put(self.words.yes, muted);
            pen.skip(BETWEEN);
            pen.put(MARK, lit);
            pen.put(self.words.no, lit);
        }
    }
}

impl Widget for ConfirmBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl Widget for &ConfirmBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        let mut y = area.y;
        if self.rule {
            text::rule(buf, area, self.styles.rule);
            y += 1;
        }
        let muted = self.styles.muted;
        if let Some(mut pen) = Pen::new(buf, area, area.x, y) {
            let room = area.width.saturating_sub(self.choices_width());
            pen.clip(self.question, self.question_style(), room);
            self.draw_choices(&mut pen);
            if self.inline
                && let Some(hint) = self.words.hint
            {
                pen.indented(hint, muted);
                return;
            }
            y += 1;
        }
        if let Some(hint) = self.words.hint
            && let Some(mut pen) = Pen::new(buf, area, area.x, y)
        {
            pen.indented(hint, muted);
        }
    }
}

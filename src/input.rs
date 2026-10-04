use std::fmt;

use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Modifier, Style},
    widgets::Widget,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::key::Key;
use crate::styles::{Styles, Tone};
use crate::text::{self, ELLIPSIS, Pen};

const RULE: &str = "─";
const MASK: &str = "•";
const SHOWN: usize = 4;
const GAP: u16 = 1;
const INDENT: &str = "  ";
const CARET: &str = " ";

#[derive(Clone, PartialEq, Eq)]
pub enum Edit {
    Pass,
    Held,
    Changed,
    Submit(String),
    Cancel,
}

impl fmt::Debug for Edit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Edit::Pass => f.write_str("Pass"),
            Edit::Held => f.write_str("Held"),
            Edit::Changed => f.write_str("Changed"),
            Edit::Submit(value) => write!(f, "Submit(<{} bytes>)", value.len()),
            Edit::Cancel => f.write_str("Cancel"),
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Input {
    value: String,
    cursor: usize,
    masked: bool,
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = f.debug_struct("Input");
        if self.masked {
            out.field("value", &format_args!("<masked>"));
        } else {
            out.field("value", &self.value);
        }
        out.field("cursor", &self.cursor())
            .field("masked", &self.masked)
            .finish()
    }
}

impl Input {
    pub fn new() -> Self {
        Input::default()
    }

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn with(mut self, value: &str) -> Self {
        self.set(value);
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn cursor(&self) -> usize {
        self.value
            .grapheme_indices(true)
            .take_while(|(at, _)| *at < self.cursor)
            .count()
    }

    pub fn set(&mut self, value: &str) {
        self.value = clean(value);
        self.cursor = self.value.len();
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    pub fn key(&mut self, key: Key) -> Edit {
        match key {
            Key::Enter => Edit::Submit(self.value.clone()),
            Key::Esc => Edit::Cancel,
            Key::Left => self.go(self.prev()),
            Key::Right => self.go(self.next()),
            Key::Home | Key::Ctrl('a') => self.go(0),
            Key::End | Key::Ctrl('e') => self.go(self.value.len()),
            Key::Backspace => self.cut(self.prev(), self.cursor),
            Key::Delete => self.cut(self.cursor, self.next()),
            Key::Ctrl('u') => self.cut(0, self.cursor),
            Key::Ctrl('w') => self.cut(self.word(), self.cursor),
            Key::Char(c) if !c.is_control() => self.insert(c.encode_utf8(&mut [0; 4])),
            _ => Edit::Pass,
        }
    }

    pub fn paste(&mut self, text: &str) -> Edit {
        self.insert(&clean(text))
    }

    fn go(&mut self, at: usize) -> Edit {
        self.cursor = at;
        Edit::Held
    }

    fn cut(&mut self, from: usize, to: usize) -> Edit {
        if from >= to {
            return Edit::Held;
        }
        self.value.replace_range(from..to, "");
        self.cursor = from;
        self.snap();
        Edit::Changed
    }

    fn insert(&mut self, text: &str) -> Edit {
        if text.is_empty() {
            return Edit::Held;
        }
        self.value.insert_str(self.cursor, text);
        self.cursor += text.len();
        self.snap();
        Edit::Changed
    }

    fn snap(&mut self) {
        self.cursor = self
            .value
            .grapheme_indices(true)
            .map(|(at, _)| at)
            .chain([self.value.len()])
            .find(|at| *at >= self.cursor)
            .unwrap_or(self.value.len());
    }

    fn prev(&self) -> usize {
        self.value
            .grapheme_indices(true)
            .rev()
            .find(|(at, _)| *at < self.cursor)
            .map_or(0, |(at, _)| at)
    }

    fn next(&self) -> usize {
        self.value
            .grapheme_indices(true)
            .map(|(at, grapheme)| at + grapheme.len())
            .find(|end| *end > self.cursor)
            .unwrap_or(self.value.len())
    }

    fn word(&self) -> usize {
        let mut at = self.cursor;
        let mut word = false;
        for (start, grapheme) in self
            .value
            .grapheme_indices(true)
            .rev()
            .skip_while(|(start, _)| *start >= self.cursor)
        {
            let space = grapheme.chars().all(char::is_whitespace);
            if space && word {
                break;
            }
            word |= !space;
            at = start;
        }
        at
    }

    fn units<'a>(&'a self, mask: &'a str) -> impl Iterator<Item = (&'a str, bool)> + Clone {
        let count = if self.masked {
            self.value.graphemes(true).count()
        } else {
            0
        };
        let open = count > SHOWN * 2;
        self.value
            .graphemes(true)
            .enumerate()
            .map(move |(index, grapheme)| {
                let hidden = self.masked && !(open && (index < SHOWN || index >= count - SHOWN));
                if hidden {
                    (mask, true)
                } else {
                    (grapheme, false)
                }
            })
    }
}

fn clean(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines().filter(|line| !line.is_empty()) {
        if !out.is_empty() {
            out.push(' ');
        }
        for c in line.chars() {
            if c == '\t' {
                out.push(' ');
            } else if !c.is_control() {
                out.push(c);
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputBar<'a> {
    label: &'a str,
    input: &'a Input,
    placeholder: &'a str,
    hint: Option<&'a str>,
    mask: &'a str,
    rule: bool,
    caret: bool,
    tone: Tone,
    styles: Styles,
}

struct Field {
    line: Rect,
    x: u16,
    start: usize,
    stop: usize,
    count: usize,
    cursor: usize,
}

impl<'a> InputBar<'a> {
    pub fn new(label: &'a str, input: &'a Input) -> Self {
        InputBar {
            label,
            input,
            placeholder: "",
            hint: None,
            mask: MASK,
            rule: true,
            caret: true,
            tone: Tone::Ink,
            styles: Styles::new(),
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn mask(mut self, mask: &'a str) -> Self {
        self.mask = mask;
        self
    }

    pub fn rule(mut self, rule: bool) -> Self {
        self.rule = rule;
        self
    }

    pub fn caret(mut self, caret: bool) -> Self {
        self.caret = caret;
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
        1 + u16::from(self.rule) + u16::from(self.hint.is_some())
    }

    pub fn cursor(&self, area: Rect) -> Option<Position> {
        let field = self.field(area)?;
        let x = field.x
            + u16::from(field.start > 0)
            + self.span(field.start, field.cursor.max(field.start));
        (x < field.line.right()).then_some(Position::new(x, field.line.y))
    }

    fn label_width(&self, width: u16) -> u16 {
        text::width(self.label).min(width.saturating_sub(GAP) / 2)
    }

    fn widths(&self) -> impl Iterator<Item = u16> + Clone {
        self.input
            .units(self.mask)
            .map(|(unit, _)| text::width(unit))
    }

    fn span(&self, from: usize, to: usize) -> u16 {
        self.widths()
            .skip(from)
            .take(to.saturating_sub(from))
            .fold(0u16, u16::saturating_add)
    }

    fn field(&self, area: Rect) -> Option<Field> {
        let y = area.y.checked_add(u16::from(self.rule))?;
        if area.is_empty() || y >= area.bottom() {
            return None;
        }
        let line = Rect::new(area.x, y, area.width, 1);
        let label = self.label_width(line.width);
        let x = if label > 0 {
            line.x.saturating_add(label).saturating_add(GAP)
        } else {
            line.x
        };
        let room = line.right().saturating_sub(x);
        let count = self.widths().count();
        let cursor = self.input.cursor();
        let caret = self.widths().nth(cursor).unwrap_or(1);
        if self.span(0, count) + u16::from(cursor == count) <= room {
            return Some(Field {
                line,
                x,
                start: 0,
                stop: count,
                count,
                cursor,
            });
        }
        let tail = u16::from(cursor + 1 < count);
        let mut start = 0;
        while start < cursor
            && u16::from(start > 0) + self.span(start, cursor) + caret + tail > room
        {
            start += 1;
        }
        let lead = u16::from(start > 0);
        let mut stop = (cursor + 1).min(count);
        while stop < count
            && lead + self.span(start, stop + 1) + u16::from(stop + 1 < count) <= room
        {
            stop += 1;
        }
        Some(Field {
            line,
            x,
            start,
            stop,
            count,
            cursor,
        })
    }

    fn draw_field(&self, buf: &mut Buffer, field: &Field) {
        let Some(mut pen) = Pen::new(buf, field.line, field.x, field.line.y) else {
            return;
        };
        let ink = self.styles.ink;
        let muted = self.styles.muted;
        let caret = |style: Style| {
            if self.caret {
                style.add_modifier(Modifier::REVERSED)
            } else {
                style
            }
        };
        if field.count == 0 {
            let mut graphemes = self.placeholder.graphemes(true);
            match graphemes.next() {
                Some(first) => {
                    pen.put(first, caret(muted));
                    let room = pen.room();
                    pen.clip(graphemes.as_str(), muted, room);
                }
                None => pen.put(CARET, caret(ink)),
            }
            return;
        }
        if field.start > 0 {
            pen.put(ELLIPSIS, muted);
        }
        for (index, (unit, hidden)) in self
            .input
            .units(self.mask)
            .enumerate()
            .skip(field.start)
            .take(field.stop - field.start)
        {
            let style = if hidden { muted } else { ink };
            if index == field.cursor {
                pen.put(unit, caret(style));
            } else {
                pen.put(unit, style);
            }
        }
        if field.stop < field.count {
            pen.put(ELLIPSIS, muted);
        } else if field.cursor == field.count {
            pen.put(CARET, caret(ink));
        }
    }
}

impl Widget for InputBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        (&self).render(area, buf);
    }
}

impl Widget for &InputBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.is_empty() {
            return;
        }
        if self.rule
            && let Some(mut pen) = Pen::new(buf, area, area.x, area.y)
        {
            pen.fill(RULE, self.styles.rule);
        }
        let Some(field) = self.field(area) else {
            return;
        };
        if let Some(mut pen) = Pen::new(buf, field.line, field.line.x, field.line.y) {
            let style = self.tone.style(&self.styles).add_modifier(Modifier::BOLD);
            pen.clip(self.label, style, self.label_width(field.line.width));
        }
        self.draw_field(buf, &field);
        if let Some(hint) = self.hint
            && let Some(mut pen) = Pen::new(buf, area, area.x, field.line.y.saturating_add(1))
        {
            let muted = self.styles.muted;
            pen.put(INDENT, muted);
            let room = pen.room();
            pen.clip(hint, muted, room);
        }
    }
}

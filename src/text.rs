use ratatui::{buffer::Buffer, layout::Rect, style::Style};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub(crate) const ELLIPSIS: &str = "…";

pub(crate) fn width(text: &str) -> u16 {
    u16::try_from(text.width()).unwrap_or(u16::MAX)
}

pub(crate) fn flow(buf: &mut Buffer, area: Rect, text: &str, style: Style) {
    let mut rest = text.trim_start();
    for y in area.top()..area.bottom() {
        if rest.is_empty() {
            return;
        }
        let Some(mut pen) = Pen::new(buf, area, area.x, y) else {
            return;
        };
        let room = pen.room();
        if y + 1 == area.bottom() || width(rest) <= room {
            pen.clip(rest, style, room);
            return;
        }
        let mut cut = 0;
        for (at, _) in rest.match_indices(' ') {
            if width(&rest[..at]) > room {
                break;
            }
            cut = at;
        }
        if cut == 0 {
            let mut used = 0u16;
            for (at, grapheme) in rest.grapheme_indices(true) {
                used = used.saturating_add(width(grapheme));
                if used > room {
                    break;
                }
                cut = at + grapheme.len();
            }
        }
        if cut == 0 {
            return;
        }
        pen.put(&rest[..cut], style);
        rest = rest[cut..].trim_start();
    }
}

pub(crate) struct Pen<'a> {
    buf: &'a mut Buffer,
    pub(crate) x: u16,
    y: u16,
    right: u16,
}

impl<'a> Pen<'a> {
    pub(crate) fn new(buf: &'a mut Buffer, area: Rect, x: u16, y: u16) -> Option<Self> {
        let area = area.intersection(buf.area);
        if area.is_empty() || y < area.top() || y >= area.bottom() {
            return None;
        }
        Some(Pen {
            buf,
            x: x.clamp(area.left(), area.right()),
            y,
            right: area.right(),
        })
    }

    pub(crate) fn room(&self) -> u16 {
        self.right.saturating_sub(self.x)
    }

    pub(crate) fn put(&mut self, text: &str, style: Style) {
        let room = self.room();
        if room == 0 || text.is_empty() {
            return;
        }
        let (end, _) = self
            .buf
            .set_stringn(self.x, self.y, text, usize::from(room), style);
        self.x = end;
    }

    pub(crate) fn clip(&mut self, text: &str, style: Style, room: u16) {
        let room = room.min(self.room());
        if width(text) <= room {
            return self.put(text, style);
        }
        if room == 0 {
            return;
        }
        let stop = self.x + room - 1;
        let (end, _) = self
            .buf
            .set_stringn(self.x, self.y, text, usize::from(room - 1), style);
        self.x = end.min(stop);
        self.put(ELLIPSIS, style);
    }

    pub(crate) fn fill(&mut self, symbol: &str, style: Style) {
        while self.x < self.right {
            self.buf[(self.x, self.y)]
                .set_symbol(symbol)
                .set_style(style);
            self.x += 1;
        }
    }

    pub(crate) fn skip(&mut self, cells: u16) {
        self.x = self.x.saturating_add(cells).min(self.right);
    }
}

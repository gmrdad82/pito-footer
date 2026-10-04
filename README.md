# pito-footer

The bottom of a pito terminal app, as a small ratatui 0.30 crate: key hints
that fit the width, a notice line, a y/n confirm, and a quit guard. The look
is in the style of HEY's terminal UI. It has no app logic and no words of its
own: the app passes in every hint, every word, every key and every style, so
any language works.

```toml
pito-footer = { git = "https://github.com/gmrdad82/pito-footer", tag = "v0.1.1" }
```

Turn on the `crossterm` feature for `Key::from(crossterm::event::KeyEvent)`
(crossterm 0.29); without it the crate has no backend dependency.

## What it does

- **Key hints, declared per screen.** Each `Hint` is a key, a label, an
  optional lead word ("Press ctrl+c again to quit") and a rank. When the
  width runs out the highest rank drops first (the later one on a tie),
  order is kept, and a pinned hint goes last of all.
- **Or wrap:** `wrap(true)` flows every hint over as many lines as it
  needs; `height(width)` says how many rows to give it.
- **`?` shows or hides the hints** through `Help` (any key can be the
  toggle); pinned hints and the notice stay. An app whose `?` opens a help
  page instead just doesn't use `Help`.
- **A notice line** under the hints for a legend, a result or a warning, in
  the app's ink, muted, accent, alert or good. A notice marked `over()` that's too
  long for one row takes the hint rows' place and wraps across them.
- **A good tone** for success: `Tone::Good` and `Notice::good(text)` use
  `Styles::good`, and fall back to the ink style until the app sets one.
- **An indent:** `indent(n)` insets the hints, the notice and the confirm by
  `n` cells while the rule stays full width; hints drop and wrap within the
  inset width.
- **Text keeps its own style:** the rules are rows of their own, so no hint,
  notice, confirm or segment ever picks up the rule's modifiers, such as DIM.
- **Keys follow display:** `shown(key, width)` says whether a key's hint is
  on screen now, so an app can accept only the keys it shows.
- **Footer segments with drop priority:** `Footer::status` lays out text
  and hint segments on one row, left or right aligned; the highest rank
  drops first, shrinkable segments are clipped to the space left, and the
  hints are just one segment among them.
- **A confirm, in the bottom bar or a notice row.** `Confirm` holds the
  choice (No unless the app starts it on Yes); `ConfirmKeys` sets the accept
  and cancel keys (`HEY`: y/n, arrows, enter accepts, esc cancels; `ENTER`:
  enter accepts, esc cancels). `ConfirmBar` draws "Question?  Yes   ▸ No"
  over a hint line, or inline on one row, in the app's words, and the footer
  can carry it in place of the hints. It returns the answer and never runs
  anything.
- **A quit guard with modes:** `Twice` (the first press arms and shows the
  app's "again" notice, a second inside the window quits, any other key
  disarms), `Ask` (the same, but while the app says work is running the
  second press asks through a confirm first) and `Once` (quit at once). The
  trigger keys, the window (2 s by default) and the wording are the app's.
  A screen that passes every key through (an embedded terminal) simply
  doesn't hand its keys to the guard.
- **Nothing is taken behind the app's back:** every `key` call returns what
  happened (`Guard::Pass` means the key is the app's), and the crate never
  reads input, the clock or the mouse itself. `deadline()` tells the event
  loop when the armed notice ends.
- **Widths by cell:** Unicode widths throughout, so diacritics (ă, î, ș, ț),
  "…" and wide glyphs measure and clip cleanly.
- **Cheap:** drawing writes straight into the buffer, with no allocation;
  well under a millisecond at 150×40 (`cargo run --release --example
  bench`).

## The API

```text
pub enum Key { Char(char), Ctrl(char), Tab, BackTab, Enter, Esc, Backspace,
               Left, Right, Up, Down, Other }
pub struct Styles { accent, muted, alert, ink, rule, good }   // all Style::new() by default
                                         // good: Option<Style>, ink until set
pub enum Tone { Ink, Muted, Accent, Alert, Good }
Hint::new(key, label).rank(u8).lead(text).pinned()
Notice::new(text, tone) | legend(text) | accent(text) | alert(text) | good(text); .over()
Help::new(open).toggle_on(Key); open(), toggle(), wants(Key), key(Key) -> bool
Segment::text(text, style) | hints(&[Hint]); .rank(u8).right().shrink()
Footer::new(&[Hint]) | status(&[Segment])
  .notice(Option<Notice>).confirm(Option<ConfirmBar>).separator(..).gap(u16)
  .open(bool).help(&Help).rule(bool).wrap(bool).indent(u16).styles(..)
  height(width), shown(key, width)
ConfirmKeys { yes, no: &'static [Key], choose }; ConfirmKeys::HEY, ConfirmKeys::ENTER
Confirm::new().keys(..).start(Answer); yes(), choosing(), key(Key) -> Option<Answer>
Words::new(yes, no).hint(text)
ConfirmBar::new(question, &confirm, words).rule(bool).inline(bool).tone(Tone).styles(..)
Wording::new(again).ask(question)
QuitGuard::new(wording).mode(Mode).window(Duration).triggers(&[Key]).confirm(Confirm)
  key(Key, now, busy) -> Guard { Pass, Held, Quit }
  tick(now) -> bool, deadline(), armed(), notice(), asking(), bar(words), cancel()
```

## Example

```rust,standalone_crate
use std::time::Instant;
use pito_footer::{Footer, Guard, Help, Hint, Key, Mode, Notice, QuitGuard, Styles, Words, Wording};
use ratatui::{Frame, layout::Rect, style::{Color, Modifier, Style}};

const HINTS: [Hint; 4] = [
    Hint::new("↑↓", "move"),
    Hint::new("enter", "open").rank(1),
    Hint::new("/", "search").rank(2),
    Hint::new("ctrl+c", "twice quit").pinned(),
];

fn guard() -> QuitGuard {
    QuitGuard::new(Wording::new("ctrl+c again to quit").ask("Work is running. Quit anyway?"))
        .mode(Mode::Ask)
}

fn key(guard: &mut QuitGuard, help: &mut Help, key: Key, busy: bool) -> bool {
    match guard.key(key, Instant::now(), busy) {
        Guard::Quit => true,
        Guard::Held => false,
        Guard::Pass => {
            help.key(key);
            false
        }
    }
}

fn draw(frame: &mut Frame, guard: &QuitGuard, help: &Help) {
    let dim = Style::new().add_modifier(Modifier::DIM);
    let styles = Styles::new().accent(Style::new().fg(Color::Magenta)).muted(dim).rule(dim);
    let words = Words::new("Yes", "No").hint("y/n choose · enter accept · esc cancel");
    let footer = Footer::new(&HINTS).styles(styles).help(help)
        .notice(guard.notice().map(Notice::accent))
        .confirm(guard.bar(words));
    let area = frame.area();
    let height = footer.height(area.width);
    frame.render_widget(footer, Rect { y: area.bottom().saturating_sub(height), height, ..area });
}
```

## Development

`bin/gate` runs `cargo fmt --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo test --all-features` (the tests draw
through ratatui's `TestBackend`, and this README's example compiles as a
doctest) and builds the bench.

## Licence

MIT, see [LICENSE](LICENSE). The look is in the style of HEY's terminal UI;
see [NOTICE.md](NOTICE.md).

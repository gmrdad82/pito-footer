# pito-footer

[![CI](https://github.com/gmrdad82/pito-footer/actions/workflows/ci.yml/badge.svg)](https://github.com/gmrdad82/pito-footer/actions/workflows/ci.yml)

The bottom of a pito terminal app, as a small ratatui 0.30 crate: key hints
that fit the width, a notice line, a y/n confirm, a one-line input, and a
quit guard. The look
is in the style of HEY's terminal UI. It has no app logic and no words of its
own: the app passes in every hint, every word, every key and every style, so
any language works.

```toml
pito-footer = { git = "https://github.com/gmrdad82/pito-footer", tag = "v0.4.0" }
```

Turn on the `crossterm` feature for `Key::from(crossterm::event::KeyEvent)`
(crossterm 0.29); without it the crate has no backend dependency. The
conversion keeps Alt apart (`Key::Alt('y')` is not `Key::Char('y')`, so Alt+y
does not answer Yes), turns Alt, Super, Meta or Hyper on any other key into
`Key::Other`, and passes Ctrl+Alt plus a character on as that character, which
is how AltGr arrives on some platforms, so diacritics can still be typed.

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
  long for one row takes the hint rows' place and wraps across them; the
  pinned hints stay on the last row under it, `height(width)` counts that row,
  and `shown(key, width)` still answers true for a pinned hint.
- **A good tone** for success: `Tone::Good` and `Notice::good(text)` use
  `Styles::good`, and fall back to the ink style until the app sets one.
- **An indent:** `indent(n)` insets the hints, the notice and the confirm by
  `n` cells while the rule stays full width; hints drop and wrap within the
  inset width.
- **Text keeps its own style:** the rules are rows of their own, so no hint,
  notice, confirm or segment ever picks up the rule's modifiers, such as DIM.
- **Control characters take no cell:** a newline, tab or carriage return
  inside a text is measured and drawn as one space between its two sides, and
  one at either end is dropped, so a multi-line message reads as one line.
- **Keys follow display:** `shown(key, width)` says whether a key's hint is
  on screen now, so an app can accept only the keys it shows.
- **Footer segments with drop priority:** `Footer::status` lays out text
  and hint segments on one row, left or right aligned; the highest rank
  drops first, shrinkable segments are clipped to the space left, and the
  hints are just one segment among them.
- **Status spans:** `Segment::spans(&[(text, Style)])` is one text segment in
  several styles, so a percentage or page dots carry their own colour; it
  measures, drops and shrinks as one segment and clips with one ellipsis in
  the style of the span it cuts. `Segment::text(text, style)` still works.
- **The app's version:** `version(name, version)` draws `name v1.2.3` in the
  muted style at the right end of the row of keys: the hint row (the last one
  when hints wrap, the status row of `Footer::status` right of every segment),
  or the hint line of a carried confirm or input, else that bar's only row.
  It takes only the room nothing else needs: it is never clipped and never
  pushes out a hint or a segment, so it is the first thing to go, whole, when
  the width runs out. When hints wrap, the last hint moves down a row rather
  than leave it no room; when `?` hides every hint, it keeps a row of its own;
  under an over notice it stays on the last row. `height(width)` counts those
  rows and `shown` is unchanged. A version that starts with `v` keeps its own,
  an empty name draws just `v1.2.3`, and without a version nothing is drawn.
  A right-aligned `Segment::text` can show a version too, but it takes its
  cells from the hints by rank, or is clipped when it shrinks.
- **A confirm, in the bottom bar or a notice row.** `Confirm` holds the
  choice (No unless the app starts it on Yes); `ConfirmKeys` sets the yes and
  no keys, the keys that switch the choice (`toggle`) and the keys that accept
  it (`accept`) (`HEY`: y/n, arrows, tab, h/l and enter, esc cancels; `ENTER`:
  enter accepts, esc cancels). `ConfirmBar` draws "Question?  Yes   ▸ No"
  over a hint line, or inline on one row, in the app's words, and the footer
  can carry it in place of the hints, keeping the bar's own styles if it has
  any. When the width is short the question is clipped last: the choices go
  before it does. It returns the answer and never runs anything.
- **A one-line input.** `Input` holds the text and the cursor; the app
  feeds it keys and gets an `Edit` back: `Changed` when the text changed,
  `Held` for a key it used without a change (a move), `Submit(text)` on
  enter, `Cancel` on esc, and `Pass` for every key that isn't the input's,
  such as ctrl+c or tab. It never submits, clears or closes anything itself.
  Editing is by grapheme and cell: left/right, home/end and ctrl+a/ctrl+e,
  backspace and delete, ctrl+u (to the start) and ctrl+w (the word before the
  cursor), so diacritics, combining marks and wide glyphs move and delete
  whole. `paste(text)` takes a bracketed paste and keeps it on one line (a
  newline, a carriage return or a pair of them each become one space).
  `limit(chars)` caps the value's length: a paste or key past it is cut or
  held, and `set` truncates.
- **An input bar.** `InputBar` draws the app's label, then the text with a
  reversed caret cell, or the app's placeholder while empty, scrolling with
  "…" at either cut end to keep the caret in view; an optional rule above and
  hint line below, like the confirm. `cursor(area)` says where the caret is,
  for an app that shows the terminal's own cursor (`caret(false)` drops the
  drawn one). The footer can carry it in place of the hints.
- **A masked input for keys:** `Input::masked(true)` draws every character as
  `•` (or the app's `mask`), and never reveals anything while the user types
  or edits. A stored value shown at rest can opt in to its ends:
  `InputBar::reveal_ends(n)` draws the first and last `n` characters of an
  input that was set with `with` or `set` and not edited since
  (`Input::edited()`), and only when it has at least four times `n` characters,
  so the revealed share stays a quarter or less. The value stays whole for
  `Submit`, and neither `Input`'s nor `Edit`'s `Debug` prints it.
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
pub enum Key { Char(char), Ctrl(char), Alt(char), Tab, BackTab, Enter, Esc, Backspace,
               Left, Right, Up, Down, Home, End, Delete, Other }   // non_exhaustive
pub struct Styles { accent, muted, alert, ink, rule, good }   // non_exhaustive; Styles::new().accent(..)
                                         // good: Option<Style>, ink until set
pub enum Tone { Ink, Muted, Accent, Alert, Good }   // non_exhaustive
Hint::new(key, label).rank(u8).lead(text).pinned()   // Hint, Notice, Segment, Part, Words,
                                                      // Wording, Edit, Answer, Mode, Guard: non_exhaustive
Notice::new(text, tone) | legend(text) | accent(text) | alert(text) | good(text); .over()
Help::new(open).toggle_on(Key); open(), toggle(), wants(Key), key(Key) -> bool
Segment::text(text, style) | spans(&[(text, Style)]) | hints(&[Hint]);
  .rank(u8).right().shrink()
Footer::new(&[Hint]) | status(&[Segment])
  .notice(Option<Notice>).confirm(Option<ConfirmBar>).input(Option<InputBar>)
  .separator(..).gap(u16).open(bool).help(&Help).rule(bool).wrap(bool)
  .indent(u16).styles(..).version(name, version)   // "name v1.2.3", dim, right end
  height(width), shown(key, width), cursor(area) -> Option<Position>
ConfirmKeys { yes, no, toggle, accept: &'static [Key], choose }   // non_exhaustive
  ConfirmKeys::HEY, ConfirmKeys::ENTER; .yes(..) .no(..) .toggle(..) .accept(..) .choose(bool)
Confirm::new().keys(..).start(Answer); yes(), choosing(), key(Key) -> Option<Answer>
Words::new(yes, no).hint(text)
ConfirmBar::new(question, &confirm, words).rule(bool).inline(bool).tone(Tone).styles(..)
pub enum Edit { Pass, Held, Changed, Submit(String), Cancel }
Input::new().masked(bool).limit(chars).with(text); value(), cursor(), is_empty(),
  is_masked(), edited(), set(text), clear(), key(Key) -> Edit, paste(text) -> Edit
InputBar::new(label, &input).placeholder(text).hint(text).mask(glyph).reveal_ends(n)
  .rule(bool).caret(bool).tone(Tone).styles(..); height(), cursor(area)
Wording::new(again).ask(question)
QuitGuard::new(wording).mode(Mode).window(Duration).triggers(&[Key]).confirm(Confirm)
  key(Key, now, busy) -> Guard { Pass, Held, Quit }
  tick(now) -> bool, deadline(), armed(), notice(), asking(), bar(words), cancel()
```

Every public enum and options struct is `#[non_exhaustive]`: match enums with a
wildcard arm, and build `Hint`, `Notice`, `Segment`, `Words`, `Wording`,
`Styles` and `ConfirmKeys` with their constructors and methods, so a later
release can add to them in a minor version.

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
        _ => {
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
        .version("app", env!("CARGO_PKG_VERSION"))
        .notice(guard.notice().map(Notice::accent))
        .confirm(guard.bar(words));
    let area = frame.area();
    let height = footer.height(area.width);
    frame.render_widget(footer, Rect { y: area.bottom().saturating_sub(height), height, ..area });
}
```

## An input prompt

```rust,standalone_crate
use pito_footer::{Edit, Footer, Hint, Input, InputBar, Key, Styles};
use ratatui::{Frame, layout::Rect};

const HINTS: [Hint; 1] = [Hint::new("/", "search")];

fn key(input: &mut Option<Input>, key: Key) -> Option<String> {
    let field = input.as_mut()?;
    match field.key(key) {
        Edit::Submit(text) => {
            *input = None;
            Some(text)
        }
        Edit::Cancel => {
            *input = None;
            None
        }
        _ => None,
    }
}

fn draw(frame: &mut Frame, input: Option<&Input>, styles: Styles) {
    let bar = input.map(|input| {
        InputBar::new("Search", input)
            .placeholder("type to filter")
            .hint("enter search · esc cancel")
    });
    let footer = Footer::new(&HINTS).styles(styles).input(bar);
    let area = frame.area();
    let height = footer.height(area.width);
    let area = Rect { y: area.bottom().saturating_sub(height), height, ..area };
    frame.render_widget(footer, area);
}
```

A bracketed paste arrives as its own event, not a key: hand its text to
`Input::paste`.

## Development

`bin/gate` runs `cargo fmt --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo test --all-features` (the tests draw
through ratatui's `TestBackend`, a counting allocator holds that drawing
allocates nothing, and this README's example compiles as a doctest) and
builds the bench. `bin/gate --fast` leaves the bench build out, and CI runs
it on every push and pull request to main. Each release is listed in
[CHANGELOG.md](CHANGELOG.md).

## Contributing

Issues and pull requests are welcome. Please read the
[code of conduct](CODE_OF_CONDUCT.md) first. A change keeps `bin/gate` green
with no warnings, draws without allocating, and leaves every word, style and
key to the app.

## Licence

The code is MIT licensed: see [LICENSE](LICENSE). The PITO name and its logos
are © Catalin Ilinca, all rights reserved, and are not covered by the MIT
licence. The look is in the style of HEY's terminal UI; see
[NOTICE.md](NOTICE.md).

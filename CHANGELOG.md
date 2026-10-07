# Changelog

Every release of pito-footer, newest first. Versions follow [Semantic
Versioning](https://semver.org/) as Cargo reads it before 1.0: a change in the
middle number may break an app, a change in the last one never does.

## 0.4.0 (2026-10-07)

- `Footer::version(name, version)` draws the app's `name v1.2.3` in the muted
  style at the right end of the row of keys: the hint row (the last one when
  hints wrap), the status row of `Footer::status` right of every segment, or
  the hint line of a carried confirm or input, else that bar's only row.
- It takes only room nothing else needs: it is never clipped and never pushes
  out a hint or a segment, so it goes first, whole, when the width runs out.
- When hints wrap, the last hint moves down a row rather than leave it no
  room; when every hint is hidden, it keeps a row of its own; under an over
  notice it stays on the last row. `height(width)` counts those rows and
  `shown` is unchanged.
- A version that starts with `v` keeps its own, an empty name draws just
  `v1.2.3`, and a footer without a version draws exactly as before.
- The repository gains a code of conduct, a CI workflow behind the README's
  badge, and this changelog.

## 0.3.0 (2026-10-05)

- The input bar lays out in one pass, and `Input::limit` caps a value.
- A masked input reveals nothing while it is edited, and its ends only on
  request at rest (`InputBar::reveal_ends`).
- A pinned hint stays under a notice marked `over()`.
- `Key` carries Alt (`Key::Alt`) and lets AltGr through as a plain character.
- Control characters are measured as they are drawn.
- The confirm's keys (`ConfirmKeys`) and the carried bars' styles are the
  app's.
- Breaking: the public enums and options structs are `#[non_exhaustive]`.

## 0.2.0 (2026-10-05)

- `Input`, a one-line input the app feeds keys and pastes to, editing by
  grapheme and cell, and handing back `Submit` and `Cancel` instead of acting,
  with a masked mode for keys that never reaches `Debug`.
- `InputBar` draws it with the app's label, placeholder, hint and styles, and
  the footer can carry it in place of the hints.
- `Segment::spans`: one status segment in several styles.
- `Key` gains `Home`, `End` and `Delete`.

## 0.1.1 (2026-10-04)

- `Footer::indent` insets the hints, the notice and the confirm while the rule
  stays full width.
- `Tone::Good` and `Notice::good` give success notices their own style, which
  falls back to ink.
- No text picks up a rule's modifiers.

## 0.1.0 (2026-10-04)

- The first release: key hints that fit the width by rank or wrap, a notice
  line, status segments with drop priority, a y/n confirm with the app's own
  keys and words, and a quit guard that arms, asks while busy or quits at
  once.

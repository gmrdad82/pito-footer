#![doc = include_str!("../README.md")]

mod confirm;
mod footer;
mod key;
mod quit;
mod styles;
mod text;

pub use confirm::{Answer, Confirm, ConfirmBar, ConfirmKeys, Words};
pub use footer::{Footer, Help, Hint, Notice, Part, Segment};
pub use key::Key;
pub use quit::{Guard, Mode, QuitGuard, WINDOW, Wording};
pub use styles::{Styles, Tone};

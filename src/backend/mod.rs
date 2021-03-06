use event;
use theme;

#[cfg(feature = "termion")]
mod termion;
#[cfg(feature = "bear-lib-terminal")]
mod blt;
#[cfg(any(feature = "ncurses", feature = "pancurses"))]
mod curses;

#[cfg(feature = "bear-lib-terminal")]
pub use self::blt::*;
#[cfg(any(feature = "ncurses", feature = "pancurses"))]
pub use self::curses::*;
#[cfg(feature = "termion")]
pub use self::termion::*;

pub trait Backend {
    fn init() -> Box<Self> where Self: Sized;
    // TODO: take `self` by value?
    // Or implement Drop?
    fn finish(&mut self);

    fn refresh(&mut self);

    fn has_colors(&self) -> bool;
    fn screen_size(&self) -> (usize, usize);

    /// Main input method
    fn poll_event(&mut self) -> event::Event;

    /// Main method used for printing
    fn print_at(&self, (usize, usize), &str);
    fn clear(&self, color: theme::Color);

    fn set_refresh_rate(&mut self, fps: u32);

    // This sets the Colours and returns the previous colours
    // to allow you to set them back when you're done.
    fn set_color(&self, colors: theme::ColorPair) -> theme::ColorPair;

    fn set_effect(&self, effect: theme::Effect);
    fn unset_effect(&self, effect: theme::Effect);
}

//! Go Away. Internal!

use theme;
use backend;
use vec::Vec2;
use event::Event;
use std::path::Path;
use std::sync::mpsc;

/// Asynchronous callback function trait.
///
/// Every `FnOnce(&mut Cursive) -> () + Send` automatically
/// implements this.
///
/// This is a workaround only because `Box<FnOnce()>` is not
/// working and `FnBox` is unstable.
pub trait CbFunc: Send {
    /// Calls the function.
    fn call_box(self: Box<Self>, &mut Box<Middleware>);
}

impl<F: FnOnce(&mut Box<Middleware>) -> () + Send> CbFunc for F {
    fn call_box(self: Box<Self>, siv: &mut Box<Middleware>) {
        (*self)(siv)
    }
}

type FnMiddleware = Fn(&mut Box<Middleware>) + 'static;


/// Middleware layer connects Backend, Printer, and Root widget
pub trait Middleware {
    /// Instanciates Middleware
    fn new() -> Box<Self> where Self: Sized;

    /// Returns a sink for asynchronous callbacks.
    ///
    /// Returns the sender part of a channel, that allows to send
    /// callbacks to `self` from other threads.
    ///
    /// Callbacks will be executed in the order
    /// of arrival on the next event cycle.
    ///
    /// Note that you currently need to call [`set_fps`] to force cursive to
    /// regularly check for messages.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// # use cursive::*;
    /// # fn main() {
    /// let mut siv = Cursive::new();
    /// siv.set_fps(10);
    ///
    /// // quit() will be called during the next event cycle
    /// siv.cb_sink().send(Box::new(|s: &mut Cursive| s.quit()));
    /// # }
    /// ```
    ///
    /// [`set_fps`]: #method.set_fps
    fn cb_sink(&self) -> &mpsc::Sender<Box<CbFunc>> ;

    /// Returns the currently used theme.
    fn current_theme(&self) -> &theme::Theme ;

    /// Sets the current theme.
    fn set_theme(&mut self, theme: theme::Theme);

    /// Clears the screen.
    ///
    /// Users rarely have to call this directly.
    fn clear(&self) {
        self.get_backend()
            .clear(self.current_theme()
                .palette[theme::PaletteColor::Background]
            );
    }

   /// Loads a theme from the given file.
    ///
    /// `filename` must point to a valid toml file.
    fn load_theme_file(&mut self, filename: &Path) -> Result<(), theme::Error> {
        self.set_theme(try!(theme::load_theme_file(filename)));
        Ok(())
    }

    /// Loads a theme from the given string content.
    ///
    /// Content must be valid toml.
    fn load_theme(&mut self, content: &str) -> Result<(), theme::Error> {
        self.set_theme(try!(theme::load_theme(content)));
        Ok(())
    }

    /// Sets the refresh rate, in frames per second.
    ///
    /// Regularly redraws everything, even when no input is given.
    ///
    /// You currently need this to regularly check
    /// for events sent using [`cb_sink`].
    ///
    /// Between 0 and 1000. Call with `fps = 0` to disable (default value).
    ///
    /// [`cb_sink`]: #method.cb_sink
    fn set_fps(&mut self, fps: u32) {
        self.get_backend().set_refresh_rate(fps)
    }

    /// Adds a global callback.
    ///
    /// Will be triggered on the given key press when no view catches it.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// # use cursive::*;
    /// # fn main() {
    /// let mut siv = Cursive::new();
    ///
    /// siv.add_global_callback('q', |s| s.quit());
    /// # }
    /// ```
    fn add_global_callback(&mut self, event: Event, cb: FnMiddleware);

    /// Removes any callback tied to the given event.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// # use cursive::*;
    /// # fn main() {
    /// let mut siv = Cursive::new();
    ///
    /// siv.add_global_callback('q', |s| s.quit());
    /// siv.clear_global_callbacks('q');
    /// # }
    /// ```
    fn clear_global_callbacks(&mut self, event: Event);

    /// Returns the size of the screen, in characters.
    fn screen_size(&self) -> Vec2 {
        let (x, y) = self.get_backend().screen_size();

        Vec2 {
            x: x as usize,
            y: y as usize,
        }
    }

    /// returns mutable copy of the backend.
    fn get_backend(&self) -> &mut backend::Backend;
}


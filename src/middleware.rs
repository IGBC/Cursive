
pub trait Middleware {
    fn new() -> Box<Self> ;

    fn cb_sink(&self) -> &mpsc::Sender<Box<CbFunc>> ;

    /// Returns the currently used theme.
    fn current_theme(&self) -> &theme::Theme ;

    /// Sets the current theme.
    fn set_theme(&mut self, theme: theme::Theme);

    /// Clears the screen.
    ///
    /// Users rarely have to call this directly.
    fn clear(&self);

   /// Loads a theme from the given file.
    ///
    /// `filename` must point to a valid toml file.
    pub fn load_theme_file<P: AsRef<Path>>(
        &mut self, filename: P
    ) -> Result<(), theme::Error> {
        self.set_theme(try!(theme::load_theme_file(filename)));
        Ok(())
    }

    /// Loads a theme from the given string content.
    ///
    /// Content must be valid toml.
    pub fn load_theme(&mut self, content: &str) -> Result<(), theme::Error> {
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
    pub fn set_fps(&mut self, fps: u32) {
        self.backend.set_refresh_rate(fps)
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
    pub fn add_global_callback<F, E: Into<Event>>(&mut self, event: E, cb: F)
    where
        F: Fn(&mut Cursive) + 'static,
    {
        self.global_callbacks
            .entry(event.into())
            .or_insert_with(Vec::new)
            .push(Callback::from_fn(cb));
    }

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
    pub fn clear_global_callbacks<E>(&mut self, event: E)
    where
        E: Into<Event>,
    {
        let event = event.into();
        self.global_callbacks.remove(&event);
    }

    /// Returns the size of the screen, in characters.
    pub fn screen_size(&self) -> Vec2 {
        let (x, y) = self.backend.screen_size();

        Vec2 {
            x: x as usize,
            y: y as usize,
        }
    }
}


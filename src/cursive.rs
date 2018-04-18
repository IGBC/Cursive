use backend;
use backend::Backend;
use event::{Callback, Event, EventResult};
use printer::Printer;
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use theme;
use vec::Vec2;
use view::{self, Finder, View};
use views;

/// Identifies a screen in the cursive root.
pub type ScreenId = usize;

/// Central part of the cursive library.
///
/// It initializes ncurses on creation and cleans up on drop.
/// To use it, you should populate it with views, layouts and callbacks,
/// then start the event loop with run().
///
/// It uses a list of screen, with one screen active at a time.
pub struct Cursive {
    theme: theme::Theme,
    root: views::Classic,
    global_callbacks: HashMap<Event, Vec<Callback>>,

    // Last layer sizes of the stack view.
    // If it changed, clear the screen.
    last_sizes: Vec<Vec2>,

    running: bool,

    backend: Box<backend::Backend>,

    cb_source: mpsc::Receiver<Box<Callback>>,
    cb_sink: mpsc::Sender<Box<Callback>>,
}

new_default!(Cursive);

impl Cursive {
    /// Creates a new Cursive root, and initialize the back-end.
    pub fn new() -> Self {
        let backend = backend::Concrete::init();
        Cursive::with_backend(backend)
    }

    /// This function grows breasts on catgurls
    pub fn with_backend(backend: Box<Backend>) -> Self {
        
        let theme = theme::load_default();
        // theme.activate(&mut backend);
        // let theme = theme::load_theme("assets/style.toml").unwrap();

        let (tx, rx) = mpsc::channel();

        Cursive {
            theme: theme,
            root: views::Classic::new(),
            last_sizes: Vec::new(),
            global_callbacks: HashMap::new(),
            running: true,
            cb_source: rx,
            cb_sink: tx,
            backend: backend,
        }
    }

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
    pub fn cb_sink(&self) -> &mpsc::Sender<Box<Callback>> {
        &self.cb_sink
    }

    /// Returns the currently used theme.
    pub fn current_theme(&self) -> &theme::Theme {
        &self.theme
    }

    /// Sets the current theme.
    pub fn set_theme(&mut self, theme: theme::Theme) {
        self.theme = theme;
        self.clear();
    }

    /// Clears the screen.
    ///
    /// Users rarely have to call this directly.
    pub fn clear(&self) {
        self.backend
            .clear(self.theme.palette[theme::PaletteColor::Background]);
    }

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

    /// Returns a reference to the currently active screen.
    pub fn root(&self) -> &views::Classic {
        &self.root
    }

    /// Returns a mutable reference to the currently active screen.
    pub fn root_mut(&mut self) -> &mut views::Classic {
        &mut self.root
    }

    /// Tries to find the view pointed to by the given selector.
    ///
    /// Runs a closure on the view once it's found, and return the
    /// result.
    ///
    /// If the view is not found, or if it is not of the asked type,
    /// returns None.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// # use cursive::{Cursive, views, view};
    /// # use cursive::traits::*;
    /// # fn main() {
    /// fn main() {
    ///     let mut siv = Cursive::new();
    ///
    ///     siv.add_layer(views::TextView::new("Text #1").with_id("text"));
    ///
    ///     siv.add_global_callback('p', |s| {
    ///         s.call_on(
    ///             &view::Selector::Id("text"),
    ///             |view: &mut views::TextView| {
    ///                 view.set_content("Text #2");
    ///             },
    ///         );
    ///     });
    ///
    /// }
    /// # }
    /// ```
    pub fn call_on<V, F, R>(
        &mut self, sel: &view::Selector, callback: F
    ) -> Option<R>
    where
        V: View + Any,
        F: FnOnce(&mut V) -> R,
    {
        self.root_mut().call_on(sel, callback)
    }

    /// Tries to find the view identified by the given id.
    ///
    /// Convenient method to use `call_on` with a `view::Selector::Id`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// # use cursive::{Cursive, views};
    /// # use cursive::traits::*;
    /// # fn main() {
    /// let mut siv = Cursive::new();
    ///
    /// siv.add_layer(views::TextView::new("Text #1")
    ///                               .with_id("text"));
    ///
    /// siv.add_global_callback('p', |s| {
    ///     s.call_on_id("text", |view: &mut views::TextView| {
    ///         view.set_content("Text #2");
    ///     });
    /// });
    /// # }
    /// ```
    pub fn call_on_id<V, F, R>(&mut self, id: &str, callback: F) -> Option<R>
    where
        V: View + Any,
        F: FnOnce(&mut V) -> R,
    {
        self.call_on(&view::Selector::Id(id), callback)
    }

    /// Convenient method to find a view wrapped in [`IdView`].
    ///
    /// This looks for a `IdView<V>` with the given ID, and return
    /// a [`ViewRef`] to the wrapped view. The `ViewRef` implements
    /// `DerefMut<Target=T>`, so you can treat it just like a `&mut T`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use cursive::Cursive;
    /// # use cursive::views::{TextView, ViewRef};
    /// # let mut siv = Cursive::new();
    /// use cursive::traits::Identifiable;
    ///
    /// siv.add_layer(TextView::new("foo").with_id("id"));
    ///
    /// // Could be called in a callback
    /// let mut view: ViewRef<TextView> = siv.find_id("id").unwrap();
    /// view.set_content("bar");
    /// ```
    ///
    /// [`IdView`]: views/struct.IdView.html
    /// [`ViewRef`]: views/type.ViewRef.html
    pub fn find_id<V>(&mut self, id: &str) -> Option<views::ViewRef<V>>
    where
        V: View + Any,
    {
        self.call_on_id(id, views::IdView::<V>::get_mut)
    }

    /// Moves the focus to the view identified by `id`.
    ///
    /// Convenient method to call `focus` with a `view::Selector::Id`.
    pub fn focus_id(&mut self, id: &str) -> Result<(), ()> {
        self.focus(&view::Selector::Id(id))
    }

    /// Moves the focus to the view identified by `sel`.
    pub fn focus(&mut self, sel: &view::Selector) -> Result<(), ()> {
        self.root_mut().focus_view(sel)
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

    // Handles a key event when it was ignored by the current view
    fn on_event(&mut self, event: Event) {
        let cb_list = match self.global_callbacks.get(&event) {
            None => return,
            Some(cb_list) => cb_list.clone(),
        };
        // Not from a view, so no viewpath here
        for cb in cb_list {
            cb(self);
        }
    }

    /// Returns the size of the screen, in characters.
    pub fn screen_size(&self) -> Vec2 {
        let (x, y) = self.backend.screen_size();

        Vec2 {
            x: x as usize,
            y: y as usize,
        }
    }

    fn layout(&mut self) {
        let size = self.screen_size();
        self.root.layout(size);
    }

    fn draw(&mut self) {
        let sizes = self.root.screen().layer_sizes();
        if self.last_sizes != sizes {
            self.clear();
            self.last_sizes = sizes;
        }

        let printer =
            Printer::new(self.screen_size(), &self.theme, &self.backend);

        self.root.draw(&printer);
    }

    /// Returns `true` until [`quit(&mut self)`] is called.
    ///
    /// [`quit(&mut self)`]: #method.quit
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Runs the event loop.
    ///
    /// It will wait for user input (key presses)
    /// and trigger callbacks accordingly.
    ///
    /// Calls [`step(&mut self)`] until [`quit(&mut self)`] is called.
    ///
    /// After this function returns, you can call
    /// it again and it will start a new loop.
    ///
    /// [`step(&mut self)`]: #method.step
    /// [`quit(&mut self)`]: #method.quit
    pub fn run(&mut self) {
        self.running = true;

        // And the big event loop begins!
        while self.running {
            self.step();
        }
    }

    /// Performs a single step from the event loop.
    ///
    /// Useful if you need tighter control on the event loop.
    /// Otherwise, [`run(&mut self)`] might be more convenient.
    ///
    /// [`run(&mut self)`]: #method.run
    pub fn step(&mut self) {
        while let Ok(cb) = self.cb_source.try_recv() {
            cb(self);
        }

        // Do we need to redraw everytime?
        // Probably, actually.
        // TODO: Do we need to re-layout everytime?
        self.layout();

        // TODO: Do we need to redraw every view every time?
        // (Is this getting repetitive? :p)
        self.draw();
        self.backend.refresh();

        // Wait for next event.
        // (If set_fps was called, this returns -1 now and then)
        let event = self.backend.poll_event();
        if event == Event::Exit {
            self.quit();
        }

        if event == Event::WindowResize {
            self.clear();
        }

        // Event dispatch order:
        // * Root element:
        // * Global callbacks
        
        match self.root_mut().on_event(event.relativized((0, 0))) {
            // If the event was ignored,
            // it is our turn to play with it.
            EventResult::Ignored => self.on_event(event),
            EventResult::Consumed(None) => (),
            EventResult::Consumed(Some(cb)) => cb(self),
        }
    }

    /// Stops the event loop.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Drop for Cursive {
    fn drop(&mut self) {
        self.backend.finish();
    }
}



pub struct CallbackController {
    callbacks: HashMap<Event, Vec<Callback>>,
}

impl CallbackController {
    fn new() -> Self {
        CallbackController {
            callbacks: HashMap::new(),
        }
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
    fn add_callback(&mut self, event: Event, cb: FnMiddleware){

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
    fn clear_callbacks_from(&mut self, event: Event){

    }
}
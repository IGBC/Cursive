use event::{Event, EventResult};
use printer::Printer;
use std::any::Any;

use XY;
use direction::Direction;
use vec::Vec2;
use view::{self, IntoBoxedView, Position, View};
use views::{self, LayerPosition};

/// Implements a Menubar and a stack of StackViews
///
/// TODO: Rename
///
/// Provides all the legacy functionality one would
/// expect from a Cursive root widget.
pub struct Classic {
    /// References to all available views
    screens: Vec<views::StackView>,

    /// Reference to a global menubar
    menubar: views::Menubar,

    /// The currently rendered screen
    active_screen: usize,

    // Last layer sizes of the stack view.
    // If it changed, clear the screen.
    last_sizes: Vec<Vec2>,
}

impl Classic {
    /// Utility function to deref current StackView
    fn screen(&self) -> &views::StackView {
        let id = self.active_screen;
        &self.screens[id]
    }

    /// Selects the menubar.
    pub fn select_menubar(&mut self) {
        self.menubar.take_focus(Direction::none());
    }

    /// Sets the menubar autohide feature.
    ///
    /// * When enabled (default), the menu is only visible when selected.
    /// * When disabled, the menu is always visible and reserves the top row.
    pub fn set_autohide_menu(&mut self, autohide: bool) {
        self.menubar.autohide = autohide;
    }

    /// Moves the focus to the view identified by `id`.
    ///
    /// Convenient method to call `focus` with a `view::Selector::Id`.
    pub fn focus_id(&mut self, id: &str) -> Result<(), ()> {
        self.focus_view(&view::Selector::Id(id))
    }

    // /// Moves the focus to the view identified by `sel`.
    // pub fn focus(&mut self, sel: &view::Selector) -> Result<(), ()> {
    //     self.screen_mut().focus_view(sel)
    // }

    /// Access the menu tree used by the menubar.
    ///
    /// This allows to add menu items to the menubar.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// #
    /// # use cursive::{Cursive, event};
    /// # use cursive::views::{Dialog};
    /// # use cursive::traits::*;
    /// # use cursive::menu::*;
    /// #
    /// # fn main() {
    /// let mut siv = Cursive::new();
    ///
    /// siv.menubar()
    ///    .add_subtree("File",
    ///         MenuTree::new()
    ///             .leaf("New", |s| s.add_layer(Dialog::info("New file!")))
    ///             .subtree("Recent", MenuTree::new().with(|tree| {
    ///                 for i in 1..100 {
    ///                     tree.add_leaf(format!("Item {}", i), |_| ())
    ///                 }
    ///             }))
    ///             .delimiter()
    ///             .with(|tree| {
    ///                 for i in 1..10 {
    ///                     tree.add_leaf(format!("Option {}", i), |_| ());
    ///                 }
    ///             })
    ///             .delimiter()
    ///             .leaf("Quit", |s| s.quit()))
    ///    .add_subtree("Help",
    ///         MenuTree::new()
    ///             .subtree("Help",
    ///                      MenuTree::new()
    ///                          .leaf("General", |s| {
    ///                              s.add_layer(Dialog::info("Help message!"))
    ///                          })
    ///                          .leaf("Online", |s| {
    ///                              s.add_layer(Dialog::info("Online help?"))
    ///                          }))
    ///             .leaf("About",
    ///                   |s| s.add_layer(Dialog::info("Cursive v0.0.0"))));
    ///
    /// siv.add_global_callback(event::Key::Esc, |s| s.select_menubar());
    /// # }
    /// ```
    pub fn menubar(&mut self) -> &mut views::Menubar {
        &mut self.menubar
    }

    /// Returns a mutable reference to the currently active screen.
    pub fn screen_mut(&mut self) -> &mut views::StackView {
        let id = self.active_screen;
        &mut self.screens[id]
    }

    /// Returns the id of the currently active screen.
    pub fn active_screen(&self) -> usize {
        self.active_screen
    }

    /// Get the size of the currently active screen
    pub fn active_size(&self) -> XY<usize> {
        let sizes = self.screen().layer_sizes();
        sizes[self.active_screen]
    }

    /// Adds a new screen, and returns its ID.
    pub fn add_screen(&mut self) -> usize {
        let res = self.screens.len();
        self.screens.push(views::StackView::new());
        res
    }

    /// Convenient method to create a new screen, and set it as active.
    pub fn add_active_screen(&mut self) -> usize {
        let res = self.add_screen();
        self.set_screen(res);
        res
    }

    /// Sets the active screen. Panics if no such screen exist.
    pub fn set_screen(&mut self, screen_id: usize) {
        if screen_id >= self.screens.len() {
            panic!(
                "Tried to set an invalid screen ID: {}, but only {} \
                 screens present.",
                screen_id,
                self.screens.len()
            );
        }
        self.active_screen = screen_id;
    }

    /// Add a layer to the current screen.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # extern crate cursive;
    /// # use cursive::*;
    /// # fn main() {
    /// let mut siv = Cursive::new();
    ///
    /// siv.add_layer(views::TextView::new("Hello world!"));
    /// # }
    /// ```
    pub fn add_layer<T>(&mut self, view: T)
    where
        T: IntoBoxedView,
    {
        self.screen_mut().add_layer(view);
    }

    /// Adds a new full-screen layer to the current screen.
    ///
    /// Fullscreen layers have no shadow.
    pub fn add_fullscreen_layer<T>(&mut self, view: T)
    where
        T: IntoBoxedView,
    {
        self.screen_mut().add_fullscreen_layer(view);
    }

    /// Convenient method to remove a layer from the current screen.
    pub fn pop_layer(&mut self) -> Option<Box<View>> {
        self.screen_mut().pop_layer()
    }

    /// Convenient stub forwarding layer repositioning.
    pub fn reposition_layer(
        &mut self, layer: LayerPosition, position: Position
    ) {
        self.screen_mut()
            .reposition_layer(layer, position);
    }
}

impl View for Classic {
    fn draw(&self, printer: &Printer) {
        let selected = self.menubar.receive_events();

        // Print the stackview background before the menubar
        let offset = if self.menubar.autohide {
            0
        } else {
            1
        };
        let id = self.active_screen;
        let sv_printer = printer.offset((0, offset), !selected);

        self.screens[id].draw_bg(&sv_printer);

        // Draw the currently active screen
        // If the menubar is active, nothing else can be.
        // Draw the menubar?
        if self.menubar.visible() {
            let printer = printer.sub_printer(
                Vec2::zero(),
                printer.size,
                self.menubar.receive_events(),
            );
            self.menubar.draw(&printer);
        }

        // finally draw stackview layers
        // using variables from above
        self.screens[id].draw_fg(&sv_printer);
    }

    fn layout(&mut self, v: Vec2) {
        let sizes = self.screen().layer_sizes();
        if self.last_sizes != sizes {
            self.last_sizes = sizes;
        }

        let size = self.active_size();
        let offset = match self.menubar.autohide {
            true => 0,
            false => 1,
        };

        let size = size.saturating_sub((0, offset));
        self.screen_mut().layout(size);
    }

    fn needs_relayout(&self) -> bool {
        true
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        let _ = constraint;
        Vec2::new(1, 1)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        self.screen_mut().on_event(event)
    }

    fn call_on_any<'a>(
        &mut self, sel: &view::Selector, cb: Box<FnMut(&mut Any) + 'a>
    ) {
        self.screen_mut().call_on_any(sel, cb);
    }

    fn focus_view(&mut self, sel: &view::Selector) -> Result<(), ()> {
        self.screen_mut().focus_view(sel)
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

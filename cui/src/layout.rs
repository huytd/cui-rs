use crate::builder::Container;
use crate::opts::*;
use crate::sys;
use crate::types::Size;
use crate::Ui;
use core::cell::Cell;

/// A row, column or plain box (`cui_row`, `cui_column`, `cui_box`).
pub struct Flex {
    begin: unsafe extern "C" fn(*mut sys::cui_ctx, *const sys::cui_box) -> sys::cui_id,
}

/// A card (`cui_card`).
pub struct Card;

/// A scroll area (`cui_scroll`).
pub struct Scroll;

/// A split view (`cui_split`).
pub struct Split<'v> {
    id: *const core::ffi::c_char,
    ratio: &'v mut f32,
}

/// Containers. Chain options, then `.show(|ui| ...)` builds the content.
impl<'f> Ui<'f> {
    /// Children stacked vertically.
    pub fn column<'a>(&mut self) -> Container<'_, 'f, BoxOpts<'a>, Flex> {
        Container::new(self, BoxOpts::new(), Flex { begin: sys::cui_column_begin })
    }
    /// Children side by side.
    pub fn row<'a>(&mut self) -> Container<'_, 'f, BoxOpts<'a>, Flex> {
        Container::new(self, BoxOpts::new(), Flex { begin: sys::cui_row_begin })
    }
    /// A box laid out by its `dir` option (`cui_box`): column, row, stack or grid.
    pub fn container<'a>(&mut self) -> Container<'_, 'f, BoxOpts<'a>, Flex> {
        Container::new(self, BoxOpts::new(), Flex { begin: sys::cui_box_begin })
    }
    /// A bordered surface with generous padding.
    pub fn card<'a>(&mut self) -> Container<'_, 'f, CardOpts<'a>, Card> {
        Container::new(self, CardOpts::new(), Card)
    }
    /// A scroll area with smooth scrolling and overlay scrollbars.
    pub fn scroll<'a>(&mut self) -> Container<'_, 'f, ScrollOpts<'a>, Scroll> {
        Container::new(self, ScrollOpts::new(), Scroll)
    }
    /// Two panes with a draggable divider. `ratio` is the first pane's share, 0..1.
    pub fn split<'a, 'v>(&mut self, id: &str, ratio: &'v mut f32) -> Container<'_, 'f, SplitOpts<'a>, Split<'v>> {
        let id = self.s(id);
        Container::new(self, SplitOpts::new(), Split { id, ratio })
    }

    /// Scrolls the scroll area with this id to `y`.
    pub fn scroll_to(&mut self, scroll_id: &str, y: f32) {
        let p = self.s(scroll_id);
        unsafe { sys::cui_scroll_to(self.ctx, p, y) }
    }
    /// Flexible space.
    pub fn spacer(&mut self) {
        unsafe { sys::cui_spacer(self.ctx) }
    }
    /// Fixed space.
    pub fn gap(&mut self, size: f32) {
        unsafe { sys::cui_gap(self.ctx, size) }
    }
    /// A hairline that follows the parent's direction.
    pub fn separator(&mut self) {
        unsafe { sys::cui_separator(self.ctx) }
    }
}

impl<'f> Container<'_, 'f, BoxOpts<'_>, Flex> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> R {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        unsafe { (c.begin)(ctx, &o) };
        let r = f(ui);
        unsafe { sys::cui_box_end(ctx) };
        r
    }
}

impl<C> Container<'_, '_, BoxOpts<'_>, C> {
    /// Fill the parent in both directions.
    pub fn fill(self) -> Self {
        self.width(Size::FILL).height(Size::FILL)
    }
    pub fn fill_width(self) -> Self {
        self.width(Size::FILL)
    }
    pub fn fill_height(self) -> Self {
        self.height(Size::FILL)
    }
}

impl<'f> Container<'_, 'f, CardOpts<'_>, Card> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> R {
        self.show_clicked(f).1
    }
    /// Builds the card and returns whether it was clicked (with `hoverable`).
    pub fn clicked<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> bool {
        self.show_clicked(f).0
    }
    /// Whether it was clicked, and the closure's value.
    pub fn show_clicked<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> (bool, R) {
        let Container { ui, opts, .. } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        let clicked = unsafe { sys::cui_card_begin(ctx, &o) };
        let r = f(ui);
        unsafe { sys::cui_card_end(ctx) };
        (clicked, r)
    }
}

impl<C> Container<'_, '_, CardOpts<'_>, C> {
    pub fn fill_width(self) -> Self {
        self.width(Size::FILL)
    }
}

impl<'f> Container<'_, 'f, ScrollOpts<'_>, Scroll> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> R {
        let Container { ui, opts, .. } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        unsafe { sys::cui_scroll_begin(ctx, &o) };
        let r = f(ui);
        unsafe { sys::cui_scroll_end(ctx) };
        r
    }
    pub fn fill(self) -> Self {
        self.width(Size::FILL).height(Size::FILL)
    }
}

impl<'f> Container<'_, 'f, SplitOpts<'_>, Split<'_>> {
    /// Builds the two panes.
    pub fn show<A, B>(self, first: impl FnOnce(&mut Ui<'f>) -> A, second: impl FnOnce(&mut Ui<'f>) -> B) -> (A, B) {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        // CUI keeps the ratio pointer until cui_split_end.
        let cell = Cell::new(*c.ratio);
        unsafe { sys::cui_split_begin(ctx, c.id, cell.as_ptr(), &o) };
        let a = first(ui);
        unsafe { sys::cui_split_next(ctx) };
        let b = second(ui);
        unsafe { sys::cui_split_end(ctx) };
        *c.ratio = cell.get();
        (a, b)
    }
}

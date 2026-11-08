use crate::opts::*;
use crate::sys;
use crate::Ui;
use core::cell::Cell;

/// Containers. Each runs your closure once, between CUI's begin and end.
impl<'f> Ui<'f> {
    /// A box with any options (`cui_box`).
    pub fn box_with<R>(&mut self, opts: BoxOpts, f: impl FnOnce(&mut Self) -> R) -> R {
        let o = opts.raw(self.arena);
        unsafe { sys::cui_box_begin(self.ctx, &o) };
        let r = f(self);
        unsafe { sys::cui_box_end(self.ctx) };
        r
    }

    /// Children side by side.
    pub fn row<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.row_with(BoxOpts::new(), f)
    }
    pub fn row_with<R>(&mut self, opts: BoxOpts, f: impl FnOnce(&mut Self) -> R) -> R {
        let o = opts.raw(self.arena);
        unsafe { sys::cui_row_begin(self.ctx, &o) };
        let r = f(self);
        unsafe { sys::cui_box_end(self.ctx) };
        r
    }

    /// Children stacked vertically.
    pub fn column<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.column_with(BoxOpts::new(), f)
    }
    pub fn column_with<R>(&mut self, opts: BoxOpts, f: impl FnOnce(&mut Self) -> R) -> R {
        let o = opts.raw(self.arena);
        unsafe { sys::cui_column_begin(self.ctx, &o) };
        let r = f(self);
        unsafe { sys::cui_box_end(self.ctx) };
        r
    }

    /// A bordered surface with generous padding.
    pub fn card<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.card_with(CardOpts::new(), f).1
    }
    /// Returns whether the card was clicked (with `hoverable`) and the closure's value.
    pub fn card_with<R>(&mut self, opts: CardOpts, f: impl FnOnce(&mut Self) -> R) -> (bool, R) {
        let o = opts.raw(self.arena);
        let clicked = unsafe { sys::cui_card_begin(self.ctx, &o) };
        let r = f(self);
        unsafe { sys::cui_card_end(self.ctx) };
        (clicked, r)
    }

    /// A scroll area with smooth scrolling and overlay scrollbars.
    pub fn scroll<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scroll_with(ScrollOpts::new(), f)
    }
    pub fn scroll_with<R>(&mut self, opts: ScrollOpts, f: impl FnOnce(&mut Self) -> R) -> R {
        let o = opts.raw(self.arena);
        unsafe { sys::cui_scroll_begin(self.ctx, &o) };
        let r = f(self);
        unsafe { sys::cui_scroll_end(self.ctx) };
        r
    }
    /// Scrolls the scroll area with this id to `y`.
    pub fn scroll_to(&mut self, scroll_id: &str, y: f32) {
        let p = self.s(scroll_id);
        unsafe { sys::cui_scroll_to(self.ctx, p, y) }
    }

    /// Two panes with a draggable divider. `ratio` is the first pane's share, 0..1.
    pub fn split<A, B>(
        &mut self,
        id: &str,
        ratio: &mut f32,
        opts: SplitOpts,
        first: impl FnOnce(&mut Self) -> A,
        second: impl FnOnce(&mut Self) -> B,
    ) -> (A, B) {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        // CUI keeps the pointer until cui_split_end.
        let cell = Cell::new(*ratio);
        unsafe { sys::cui_split_begin(self.ctx, id, cell.as_ptr(), &o) };
        let a = first(self);
        unsafe { sys::cui_split_next(self.ctx) };
        let b = second(self);
        unsafe { sys::cui_split_end(self.ctx) };
        *ratio = cell.get();
        (a, b)
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

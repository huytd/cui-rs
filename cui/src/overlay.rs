use crate::opts::*;
use crate::sys;
use crate::Ui;

/// A queued toast, to ask whether its action was clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ToastHandle(pub i32);

/// An entry of the command palette.
#[derive(Clone, Copy, Debug, Default)]
pub struct Command<'a> {
    pub title: &'a str,
    pub icon: Option<&'a str>,
    pub group: Option<&'a str>,
    pub shortcut: Option<&'a str>,
}

impl<'a> Command<'a> {
    pub fn new(title: &'a str) -> Self {
        Command { title, ..Default::default() }
    }
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn group(mut self, group: &'a str) -> Self {
        self.group = Some(group);
        self
    }
    pub fn shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
}

/// Popovers, menus, dialogs, sheets, toasts and alerts. Closures of
/// containers that may be closed run only while they are showing; the
/// method then returns `Some` with the closure's value.
impl<'f> Ui<'f> {
    /// A popover anchored to the previous component. Show it with
    /// [`Ui::popover_open`] or [`Ui::popover_toggle`].
    pub fn popover<R>(&mut self, id: &str, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        self.popover_with(id, PopoverOpts::new(), f)
    }
    pub fn popover_with<R>(&mut self, id: &str, opts: PopoverOpts, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let id = self.s(id);
        let o = opts.raw(self.arena);
        if !unsafe { sys::cui_popover_begin(self.ctx, id, &o) } {
            return None;
        }
        let r = f(self);
        unsafe { sys::cui_popover_end(self.ctx) };
        Some(r)
    }
    pub fn popover_open(&mut self, id: &str) {
        let id = self.s(id);
        unsafe { sys::cui_popover_open(self.ctx, id) }
    }
    pub fn popover_close(&mut self, id: &str) {
        let id = self.s(id);
        unsafe { sys::cui_popover_close(self.ctx, id) }
    }
    pub fn popover_toggle(&mut self, id: &str) {
        let id = self.s(id);
        unsafe { sys::cui_popover_toggle(self.ctx, id) }
    }
    pub fn popover_is_open(&mut self, id: &str) -> bool {
        let id = self.s(id);
        unsafe { sys::cui_popover_is_open(self.ctx, id) }
    }

    /// A button that opens a menu; build [`Ui::menu_item`]s inside.
    pub fn menu<R>(&mut self, label: &str, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        self.menu_with(label, MenuOpts::new(), f)
    }
    pub fn menu_with<R>(&mut self, label: &str, opts: MenuOpts, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        if !unsafe { sys::cui_menu_begin(self.ctx, l, &o) } {
            return None;
        }
        let r = f(self);
        unsafe { sys::cui_menu_end(self.ctx) };
        Some(r)
    }
    /// A menu that opens on right click over the previous component.
    pub fn context_menu<R>(&mut self, id: &str, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let id = self.s(id);
        if !unsafe { sys::cui_context_menu_begin(self.ctx, id) } {
            return None;
        }
        let r = f(self);
        unsafe { sys::cui_context_menu_end(self.ctx) };
        Some(r)
    }
    /// Returns true when chosen.
    pub fn menu_item(&mut self, label: &str) -> bool {
        self.menu_item_with(label, MenuItemOpts::new())
    }
    pub fn menu_item_with(&mut self, label: &str, opts: MenuItemOpts) -> bool {
        let l = self.s(label);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_menu_item_ex(self.ctx, l, &o) }
    }
    pub fn menu_separator(&mut self) {
        unsafe { sys::cui_menu_separator(self.ctx) }
    }
    pub fn menu_label(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_menu_label(self.ctx, t) }
    }
    /// Tooltip for the previous component.
    pub fn tooltip(&mut self, text: &str) {
        let t = self.s(text);
        unsafe { sys::cui_tooltip(self.ctx, t) }
    }

    /// A modal dialog, shown while `*open` is true. Esc, the close button and
    /// clicks outside set it to false; the closure gets it too, to close the
    /// dialog from its own buttons.
    pub fn dialog<R>(&mut self, title: &str, open: &mut bool, f: impl FnOnce(&mut Self, &mut bool) -> R) -> Option<R> {
        self.dialog_with(title, open, DialogOpts::new(), f)
    }
    pub fn dialog_with<R>(
        &mut self,
        title: &str,
        open: &mut bool,
        opts: DialogOpts,
        f: impl FnOnce(&mut Self, &mut bool) -> R,
    ) -> Option<R> {
        let t = self.s(title);
        let o = opts.raw(self.arena);
        // CUI only touches `open` inside cui_dialog_begin.
        if !unsafe { sys::cui_dialog_begin(self.ctx, t, open, &o) } {
            return None;
        }
        let r = f(self, open);
        unsafe { sys::cui_dialog_end(self.ctx) };
        Some(r)
    }
    /// A right-aligned row for dialog actions.
    pub fn dialog_footer<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        unsafe { sys::cui_dialog_footer_begin(self.ctx) };
        let r = f(self);
        unsafe { sys::cui_dialog_footer_end(self.ctx) };
        r
    }
    /// A panel that slides in from an edge, shown while `*open` is true.
    pub fn sheet<R>(&mut self, title: &str, open: &mut bool, f: impl FnOnce(&mut Self, &mut bool) -> R) -> Option<R> {
        self.sheet_with(title, open, SheetOpts::new(), f)
    }
    pub fn sheet_with<R>(
        &mut self,
        title: &str,
        open: &mut bool,
        opts: SheetOpts,
        f: impl FnOnce(&mut Self, &mut bool) -> R,
    ) -> Option<R> {
        let t = self.s(title);
        let o = opts.raw(self.arena);
        if !unsafe { sys::cui_sheet_begin(self.ctx, t, open, &o) } {
            return None;
        }
        let r = f(self, open);
        unsafe { sys::cui_sheet_end(self.ctx) };
        Some(r)
    }

    /// Queues a toast notification.
    pub fn toast(&mut self, title: &str) -> ToastHandle {
        self.toast_with(title, ToastOpts::new())
    }
    pub fn toast_with(&mut self, title: &str, opts: ToastOpts) -> ToastHandle {
        let t = self.s(title);
        let o = opts.raw(self.arena);
        ToastHandle(unsafe { sys::cui_toast_ex(self.ctx, t, &o) })
    }
    pub fn toast_action_clicked(&mut self, toast: ToastHandle) -> bool {
        unsafe { sys::cui_toast_action_clicked(self.ctx, toast.0) }
    }
    pub fn toast_dismiss(&mut self, toast: ToastHandle) {
        unsafe { sys::cui_toast_dismiss(self.ctx, toast.0) }
    }

    /// Inline alert. Returns false once dismissed (when dismissible).
    pub fn alert(&mut self, title: &str) -> bool {
        self.alert_with(title, AlertOpts::new())
    }
    pub fn alert_with(&mut self, title: &str, opts: AlertOpts) -> bool {
        let t = self.s(title);
        let o = opts.raw(self.arena);
        unsafe { sys::cui_alert_ex(self.ctx, t, &o) }
    }

    /// Spotlight style command palette. Returns the chosen command's index.
    /// Open it with a shortcut: `if ui.shortcut(Mods::PRIMARY, Key::K) { open = true }`.
    pub fn command_palette(&mut self, open: &mut bool, commands: &[Command]) -> Option<usize> {
        let raw: Vec<sys::cui_command> = commands
            .iter()
            .map(|c| sys::cui_command {
                title: self.arena.str(c.title),
                icon: self.arena.opt_str(c.icon),
                group: self.arena.opt_str(c.group),
                shortcut: self.arena.opt_str(c.shortcut),
            })
            .collect();
        let i = unsafe { sys::cui_command_palette(self.ctx, open, raw.as_ptr(), raw.len() as i32) };
        usize::try_from(i).ok()
    }
}

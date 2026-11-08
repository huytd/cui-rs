use crate::builder::{Container, Run, Widget};
use crate::opts::*;
use crate::sys;
use crate::widgets::bool_result;
use crate::Ui;
use core::ffi::c_char;

type CStr = *const c_char;

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

/// A popover anchored to the previous component (`cui_popover`).
pub struct Popover(CStr);

/// A button that opens a menu (`cui_menu`).
pub struct Menu(CStr);

/// A menu entry (`cui_menu_item`).
pub struct MenuItem(CStr);

/// A modal dialog (`cui_dialog`).
pub struct Dialog<'v> {
    title: CStr,
    open: &'v mut bool,
}

/// A panel that slides in from an edge (`cui_sheet`).
pub struct Sheet<'v> {
    title: CStr,
    open: &'v mut bool,
}

/// A toast notification (`cui_toast`).
pub struct Toast(CStr);

/// An inline alert (`cui_alert`).
pub struct Alert(CStr);

/// Popovers, menus, dialogs, sheets, toasts and alerts. Containers that may
/// be closed run their closure only while showing, and return `Some` with
/// its value.
impl<'f> Ui<'f> {
    /// A popover anchored to the previous component. Show it with
    /// [`Ui::popover_open`] or [`Ui::popover_toggle`].
    pub fn popover<'a>(&mut self, id: &str) -> Container<'_, 'f, PopoverOpts<'a>, Popover> {
        let id = self.s(id);
        Container::new(self, PopoverOpts::new(), Popover(id))
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
    pub fn menu<'a>(&mut self, label: &str) -> Container<'_, 'f, MenuOpts<'a>, Menu> {
        let l = self.s(label);
        Container::new(self, MenuOpts::new(), Menu(l))
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
    /// A menu entry. `.clicked()` tells whether it was chosen.
    pub fn menu_item<'a>(&mut self, label: &str) -> Widget<'_, 'f, MenuItemOpts<'a>, MenuItem> {
        let l = self.s(label);
        Widget::new(self, MenuItemOpts::new(), MenuItem(l))
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
    /// clicks outside set it to false; `.show` passes it to the closure too,
    /// so the dialog's own buttons can close it.
    pub fn dialog<'a, 'v>(&mut self, title: &str, open: &'v mut bool) -> Container<'_, 'f, DialogOpts<'a>, Dialog<'v>> {
        let t = self.s(title);
        Container::new(self, DialogOpts::new(), Dialog { title: t, open })
    }
    /// A right-aligned row for dialog actions.
    pub fn dialog_footer<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        unsafe { sys::cui_dialog_footer_begin(self.ctx) };
        let r = f(self);
        unsafe { sys::cui_dialog_footer_end(self.ctx) };
        r
    }
    /// A panel that slides in from an edge, shown while `*open` is true.
    pub fn sheet<'a, 'v>(&mut self, title: &str, open: &'v mut bool) -> Container<'_, 'f, SheetOpts<'a>, Sheet<'v>> {
        let t = self.s(title);
        Container::new(self, SheetOpts::new(), Sheet { title: t, open })
    }

    /// Queues a toast notification. `.show()` returns its handle.
    pub fn toast<'a>(&mut self, title: &str) -> Widget<'_, 'f, ToastOpts<'a>, Toast> {
        let t = self.s(title);
        Widget::new(self, ToastOpts::new(), Toast(t))
    }
    pub fn toast_action_clicked(&mut self, toast: ToastHandle) -> bool {
        unsafe { sys::cui_toast_action_clicked(self.ctx, toast.0) }
    }
    pub fn toast_dismiss(&mut self, toast: ToastHandle) {
        unsafe { sys::cui_toast_dismiss(self.ctx, toast.0) }
    }

    /// Inline alert. `.show()` returns false once it is dismissed.
    pub fn alert<'a>(&mut self, title: &str) -> Widget<'_, 'f, AlertOpts<'a>, Alert> {
        let t = self.s(title);
        Widget::new(self, AlertOpts::new(), Alert(t))
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

impl<'f> Container<'_, 'f, PopoverOpts<'_>, Popover> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> Option<R> {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        if !unsafe { sys::cui_popover_begin(ctx, c.0, &o) } {
            return None;
        }
        let r = f(ui);
        unsafe { sys::cui_popover_end(ctx) };
        Some(r)
    }
}

impl<'f> Container<'_, 'f, MenuOpts<'_>, Menu> {
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>) -> R) -> Option<R> {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        if !unsafe { sys::cui_menu_begin(ctx, c.0, &o) } {
            return None;
        }
        let r = f(ui);
        unsafe { sys::cui_menu_end(ctx) };
        Some(r)
    }
}

impl<'f> Container<'_, 'f, DialogOpts<'_>, Dialog<'_>> {
    /// Builds the dialog while it is open. The closure gets `open` back so
    /// its buttons can close the dialog.
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>, &mut bool) -> R) -> Option<R> {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        // CUI only touches `open` inside cui_dialog_begin.
        if !unsafe { sys::cui_dialog_begin(ctx, c.title, c.open, &o) } {
            return None;
        }
        let r = f(ui, c.open);
        unsafe { sys::cui_dialog_end(ctx) };
        Some(r)
    }
}

impl<'f> Container<'_, 'f, SheetOpts<'_>, Sheet<'_>> {
    /// Builds the sheet while it is open. The closure gets `open` back.
    pub fn show<R>(self, f: impl FnOnce(&mut Ui<'f>, &mut bool) -> R) -> Option<R> {
        let Container { ui, opts, c } = self;
        let (ctx, o) = (ui.ctx, opts.raw(ui.arena));
        if !unsafe { sys::cui_sheet_begin(ctx, c.title, c.open, &o) } {
            return None;
        }
        let r = f(ui, c.open);
        unsafe { sys::cui_sheet_end(ctx) };
        Some(r)
    }
}

impl Run<MenuItemOpts<'_>> for MenuItem {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &MenuItemOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_menu_item_ex(ui.ctx, self.0, &o) }
    }
}

impl Run<ToastOpts<'_>> for Toast {
    type Output = ToastHandle;
    fn run(self, ui: &mut Ui<'_>, opts: &ToastOpts<'_>) -> ToastHandle {
        let o = opts.raw(ui.arena);
        ToastHandle(unsafe { sys::cui_toast_ex(ui.ctx, self.0, &o) })
    }
}

impl Run<AlertOpts<'_>> for Alert {
    type Output = bool;
    fn run(self, ui: &mut Ui<'_>, opts: &AlertOpts<'_>) -> bool {
        let o = opts.raw(ui.arena);
        unsafe { sys::cui_alert_ex(ui.ctx, self.0, &o) }
    }
}

bool_result!(clicked, "Draws it; true once, when chosen.": MenuItem => MenuItemOpts);

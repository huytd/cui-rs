//! Safe Rust bindings to [CUI](https://github.com/yappologistic/CUI):
//! modern native UI components, immediate mode.
//!
//! ```no_run
//! # #[cfg(feature = "app")] {
//! use cui::{App, AppDesc, BoxOpts, ButtonOpts, ToastOpts, Tone};
//!
//! let mut notifications = false;
//! App::run(&AppDesc::new("Hello").size(900, 600), |ui| {
//!     ui.column_with(BoxOpts::new().pad(24.0).gap(12.0), |ui| {
//!         ui.heading("Hello, CUI", 2);
//!         ui.switch("Notifications", &mut notifications);
//!         if ui.button_with("Save", ButtonOpts::new().primary()) {
//!             ui.toast_with("Saved", ToastOpts::new().tone(Tone::Success));
//!         }
//!     });
//! })
//! .unwrap();
//! # }
//! ```
//!
//! # Mapping from C
//!
//! | C | Rust |
//! |---|---|
//! | `cui_button(ui, "Save")` | `ui.button("Save")` |
//! | `cui_button(ui, "Save", .variant = CUI_BUTTON_PRIMARY)` | `ui.button_with("Save", ButtonOpts::new().variant(ButtonVariant::Primary))` |
//! | `cui_row(ui, .gap = 8) { ... }` | `ui.row_with(BoxOpts::new().gap(8.0), \|ui\| { ... })` |
//! | `if (cui_dialog(ui, "T", &open)) { ... }` | `ui.dialog("T", &mut open, \|ui, open\| { ... })` |
//! | `char buf[64]` | [`TextBuffer::new(64)`](TextBuffer) |
//! | `CUI_ICON_PLUS` | [`icons::PLUS`] |
//! | `CUI_FILL`, `CUI_PX(200)` | [`Size::FILL`], `200.0` (or `Size::Px(200.0)`) |
//! | `CUI_COLOR_ACCENT` | [`Color::ACCENT`] or `Token::Accent.into()` |
//!
//! Everything in CUI is also available raw through [`sys`]; use
//! [`Ui::raw`] to get the `cui_ctx*` for those calls.

mod arena;
mod display;
mod headless;
mod layout;
mod opts;
mod overlay;
mod text_buffer;
mod types;
mod ui;
mod widgets;

#[cfg(feature = "app")]
mod app;

pub mod icons;

pub use cui_sys as sys;
pub use display::{Column, MarkdownResponse, Series, TableSort};
pub use headless::Headless;
pub use opts::*;
pub use overlay::{Command, ToastHandle};
pub use text_buffer::TextBuffer;
pub use types::{
    Accent, Align, ButtonVariant, Color, ControlSize, Cursor, Date, Dir, Elevation, Font, Id, InputEvents,
    Justify, Key, Mods, MouseButton, Placement, Rect, Side, Size, TabsVariant, TextAlign, Texture, ThemeMode,
    Tone, Token, Vec2,
};
pub use ui::Ui;

#[cfg(feature = "app")]
pub use app::{open_url, App, AppDesc, Error, FileDialog, FileDialogKind, RendererKind};

/// Version of the CUI C library this crate is built on, e.g. "1.1.0".
pub fn cui_version() -> &'static str {
    let v = sys::CUI_VERSION_STRING;
    core::str::from_utf8(&v[..v.len() - 1]).unwrap_or("")
}
/// The upstream CUI commit.
pub use cui_sys::CUI_COMMIT;

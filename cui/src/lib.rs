//! Safe Rust bindings to [CUI](https://github.com/yappologistic/CUI):
//! modern native UI components, immediate mode.
//!
//! ```no_run
//! # #[cfg(feature = "app")] {
//! use cui::prelude::*;
//!
//! let mut notifications = false;
//! let mut clicks = 0;
//! App::run(&AppDesc::new("Hello").size(900, 600), |ui| {
//!     ui.column().pad(24).gap(12).show(|ui| {
//!         ui.heading("Hello, CUI", 2);
//!         ui.switch("Notifications", &mut notifications);
//!         if ui.button("Save").primary().clicked() {
//!             clicks += 1;
//!             ui.toast("Saved").tone(Tone::Success);
//!         }
//!         text!(ui, "Saved {clicks} times");
//!     });
//! })
//! .unwrap();
//! # }
//! ```
//!
//! # How components work
//!
//! Components with options return a builder. Chain options with CUI's field
//! names; the component draws when the statement ends, or when you ask for
//! its result with `.clicked()`, `.changed()` or `.show()`. Containers end
//! with `.show(|ui| ...)`. Components without options take their arguments
//! and draw right away. See [`Widget`] and [`Container`].
//!
//! # Mapping from C
//!
//! | C | Rust |
//! |---|---|
//! | `cui_button(ui, "Save")` | `ui.button("Save");` |
//! | `if (cui_button(ui, "Save", .variant = CUI_BUTTON_PRIMARY))` | `if ui.button("Save").primary().clicked()` |
//! | `cui_row(ui, .gap = 8) { ... }` | `ui.row().gap(8).show(\|ui\| { ... })` |
//! | `if (cui_dialog(ui, "T", &open)) { ... }` | `ui.dialog("T", &mut open).show(\|ui, open\| { ... })` |
//! | `cui_textf(ui, "%d items", n)` | `text!(ui, "{n} items")` |
//! | `char buf[64]; cui_input(ui, "id", buf, sizeof buf)` | `ui.input("id", &mut string)` (a `String` or [`TextBuffer`]) |
//! | `cui_push_id_int(ui, i); ... cui_pop_id(ui);` | `ui.id_scope(i, \|ui\| ...)` |
//! | `CUI_ICON_PLUS` | [`icons::PLUS`] |
//! | `CUI_FILL`, `CUI_PX(200)` | [`FILL`], `200` |
//! | `CUI_COLOR_ACCENT` | [`Color::ACCENT`] or `Token::Accent` |
//!
//! Everything in CUI is also available raw through [`sys`]; use
//! [`Ui::raw`] to get the `cui_ctx*` for those calls.

mod arena;
mod builder;
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
pub use builder::{Container, IdKey, IntoOpt, TextTarget, Widget};
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

/// The builder parts of each component, as they appear in builder types
/// (`Widget<'_, '_, ButtonOpts, component::Button>`). You rarely name them.
pub mod component {
    pub use crate::display::{
        Avatar, Badge, Chart, CodeBlock, Donut, EmptyState, Image, ListItem, Markdown, Metric, Progress, Table, Tree,
        TreeLeaf,
    };
    pub use crate::layout::{Card, Flex, Scroll, Split};
    pub use crate::overlay::{Alert, Dialog, Menu, MenuItem, Popover, Sheet, Toast};
    pub use crate::widgets::{
        Button, Check, Chip, DatePicker, IconButton, NavItem, Number, Radio, Segmented, Select, Slider, TagInput, Tabs,
        Text, TextField,
    };
}

/// Take all the free space (`CUI_FILL`): `.width(FILL)`.
pub const FILL: Size = Size::FILL;

/// Formatted body text, like `format!`: `text!(ui, "{n} items")`. Returns
/// the text builder, so options chain on: `text!(ui, "{n}").tabular(true)`.
/// Formats straight into frame memory, without a `String`.
#[macro_export]
macro_rules! text {
    ($ui:expr, $($arg:tt)*) => {
        $ui.text_fmt(::core::format_args!($($arg)*))
    };
}

/// Everything an app usually needs: `use cui::prelude::*;`.
pub mod prelude {
    #[cfg(feature = "app")]
    pub use crate::{App, AppDesc};
    pub use crate::{
        icons, text, Accent, Align, ButtonVariant, Color, Column, Command, ControlSize, Date, Dir, Elevation, Font,
        Justify, Key, Mods, MouseButton, Placement, Series, Side, Size, TableSort, TabsVariant, TextAlign, TextBuffer,
        ThemeMode, Tone, Token, Ui, FILL,
    };
}

#[cfg(feature = "app")]
pub use app::{open_url, App, AppDesc, Error, FileDialog, FileDialogKind, RendererKind};

/// Version of the CUI C library this crate is built on, e.g. "1.1.0".
pub fn cui_version() -> &'static str {
    let v = sys::CUI_VERSION_STRING;
    core::str::from_utf8(&v[..v.len() - 1]).unwrap_or("")
}
/// The upstream CUI commit.
pub use cui_sys::CUI_COMMIT;

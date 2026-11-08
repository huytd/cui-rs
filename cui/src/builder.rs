//! The builders that components return.
//!
//! A component call such as `ui.button("Save")` returns a [`Widget`]. Chain
//! options onto it; it draws itself when the statement ends, or when you ask
//! for its result:
//!
//! ```
//! # let mut h = cui::Headless::new(400.0, 300.0, 1.0);
//! # h.frame(0.016, |ui| {
//! ui.button("Cancel");                                  // drawn at the `;`
//! if ui.button("Save").primary().clicked() { /* ... */ } // drawn by .clicked()
//! # });
//! ```
//!
//! Containers return a [`Container`]; chain options, then `.show(|ui| ...)`
//! builds the content.
//!
//! A builder borrows the `Ui` mutably, so nothing else can be built until it
//! has drawn: components always come out in the order you wrote them.
//!
//! One consequence: a closure must not return a builder. Write
//! `.show(|ui| { ui.text("Hi"); })`, with braces and a semicolon, not
//! `.show(|ui| ui.text("Hi"))`; the compiler rejects the second form with a
//! "lifetime may not live long enough" error.

use crate::sys;
use crate::types::*;
use crate::Ui;

/// Draws a component with its options. Implemented by each component.
#[doc(hidden)]
pub trait Run<O> {
    type Output;
    fn run(self, ui: &mut Ui<'_>, opts: &O) -> Self::Output;
}

/// A component with options, drawn when it is dropped (the end of the
/// statement) or by [`Widget::show`] and the result shortcuts like
/// `.clicked()`.
pub struct Widget<'u, 'f, O, C: Run<O>> {
    pub(crate) ui: &'u mut Ui<'f>,
    pub(crate) opts: O,
    pub(crate) c: Option<C>,
}

impl<'u, 'f, O, C: Run<O>> Widget<'u, 'f, O, C> {
    pub(crate) fn new(ui: &'u mut Ui<'f>, opts: O, c: C) -> Self {
        Widget { ui, opts, c: Some(c) }
    }

    /// Draws the component now and returns what it reported.
    pub fn show(mut self) -> C::Output {
        let c = self.c.take().expect("widget already drawn");
        c.run(self.ui, &self.opts)
    }

    /// Replaces all options at once, e.g. with a preset you keep around.
    pub fn opts(mut self, opts: O) -> Self {
        self.opts = opts;
        self
    }
}

impl<O, C: Run<O>> Drop for Widget<'_, '_, O, C> {
    fn drop(&mut self) {
        if let Some(c) = self.c.take() {
            // Do not draw half-built UI while unwinding from a panic.
            if !std::thread::panicking() {
                c.run(self.ui, &self.opts);
            }
        }
    }
}

/// A container with options. Finish it with `.show(|ui| ...)`, which builds
/// the content between CUI's begin and end calls.
#[must_use = "a container draws nothing until you call .show(|ui| ...)"]
pub struct Container<'u, 'f, O, C> {
    pub(crate) ui: &'u mut Ui<'f>,
    pub(crate) opts: O,
    pub(crate) c: C,
}

impl<'u, 'f, O, C> Container<'u, 'f, O, C> {
    pub(crate) fn new(ui: &'u mut Ui<'f>, opts: O, c: C) -> Self {
        Container { ui, opts, c }
    }

    /// Replaces all options at once, e.g. with a preset you keep around.
    pub fn opts(mut self, opts: O) -> Self {
        self.opts = opts;
        self
    }
}

/// Values an option setter accepts for a field of type `T`. Besides `T`
/// itself: plain numbers for sizes (`.width(200)`), integers for floats
/// (`.pad(16)`), a `&str` for an optional string (`.icon(icons::PLUS)`), a
/// [`Token`] for a color, and arrays or vectors for slices.
pub trait IntoOpt<T> {
    fn into_opt(self) -> T;
}

impl<T> IntoOpt<T> for T {
    fn into_opt(self) -> T {
        self
    }
}

macro_rules! into_opt_as {
    ($target:ty: $($from:ty),*) => {$(
        impl IntoOpt<$target> for $from {
            fn into_opt(self) -> $target { self as $target }
        }
    )*};
}
into_opt_as!(f32: f64, i32, u32, usize);
into_opt_as!(f64: f32, i32, u32, usize);
into_opt_as!(i32: u32, usize, i64, u64);

macro_rules! into_opt_px {
    ($($from:ty),*) => {$(
        impl IntoOpt<Size> for $from {
            fn into_opt(self) -> Size { Size::Px(self as f32) }
        }
    )*};
}
into_opt_px!(f32, f64, i32, u32, usize);

impl IntoOpt<Color> for Token {
    fn into_opt(self) -> Color {
        Color::token(self)
    }
}

impl<'a> IntoOpt<Option<&'a str>> for &'a str {
    fn into_opt(self) -> Option<&'a str> {
        Some(self)
    }
}
impl<'a> IntoOpt<Option<&'a str>> for &'a String {
    fn into_opt(self) -> Option<&'a str> {
        Some(self.as_str())
    }
}

impl<'a, T> IntoOpt<Option<&'a [T]>> for &'a [T] {
    fn into_opt(self) -> Option<&'a [T]> {
        Some(self)
    }
}
impl<'a, T, const N: usize> IntoOpt<Option<&'a [T]>> for &'a [T; N] {
    fn into_opt(self) -> Option<&'a [T]> {
        Some(self.as_slice())
    }
}
impl<'a, T> IntoOpt<Option<&'a [T]>> for &'a Vec<T> {
    fn into_opt(self) -> Option<&'a [T]> {
        Some(self.as_slice())
    }
}

/// Text a field edits in place: a [`String`] or a fixed-size
/// [`crate::TextBuffer`].
pub trait TextTarget {
    #[doc(hidden)]
    fn edit<R>(&mut self, scratch: &mut Vec<u8>, f: impl FnOnce(&mut [u8]) -> R) -> R;
}

impl TextTarget for crate::TextBuffer {
    fn edit<R>(&mut self, _: &mut Vec<u8>, f: impl FnOnce(&mut [u8]) -> R) -> R {
        f(self.as_mut_bytes())
    }
}

impl TextTarget for String {
    fn edit<R>(&mut self, scratch: &mut Vec<u8>, f: impl FnOnce(&mut [u8]) -> R) -> R {
        // CUI edits a fixed buffer, so give it the text plus room to grow;
        // one frame of typing or pasting fits easily.
        let text = self.split('\0').next().unwrap_or("");
        scratch.clear();
        scratch.extend_from_slice(text.as_bytes());
        scratch.resize(text.len() + text.len().max(4096) + 1, 0);
        let r = f(scratch);
        let n = scratch.iter().position(|&b| b == 0).unwrap_or(scratch.len());
        if scratch[..n] != *self.as_bytes() {
            let edited = String::from_utf8_lossy(&scratch[..n]);
            self.clear();
            self.push_str(&edited);
        }
        r
    }
}

/// Keys for [`Ui::id_scope`]: strings or integers.
pub trait IdKey {
    #[doc(hidden)]
    fn push(self, ui: &mut Ui<'_>);
}

impl IdKey for &str {
    fn push(self, ui: &mut Ui<'_>) {
        let p = ui.s(self);
        unsafe { sys::cui_push_id(ui.ctx, p) }
    }
}
impl IdKey for &String {
    fn push(self, ui: &mut Ui<'_>) {
        self.as_str().push(ui)
    }
}

macro_rules! id_key_int {
    ($($t:ty),*) => {$(
        impl IdKey for $t {
            #[allow(clippy::unnecessary_cast)]
            fn push(self, ui: &mut Ui<'_>) {
                // Fold wide values so large ids stay distinct in a C int.
                let v = self as u64;
                unsafe { sys::cui_push_id_int(ui.ctx, (v ^ (v >> 32)) as i32) }
            }
        }
    )*};
}
id_key_int!(i32, u32, i64, u64, usize);

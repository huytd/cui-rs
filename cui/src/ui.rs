use crate::arena::Arena;
use crate::builder::IdKey;
use crate::sys;
use crate::types::*;
use core::ffi::{c_char, CStr};
use core::ptr;

/// The UI for one frame. Every component is a method; call them every frame
/// and they return what happened (clicked, changed).
///
/// Containers take a closure that builds their content, which keeps CUI's
/// begin/end calls paired.
pub struct Ui<'f> {
    pub(crate) ctx: *mut sys::cui_ctx,
    pub(crate) arena: &'f mut Arena,
    #[cfg(feature = "app")]
    pub(crate) app: *mut sys::cui_app,
}

impl<'f> Ui<'f> {
    /// The raw context, for calling anything in [`crate::sys`] that is not
    /// wrapped yet.
    pub fn raw(&mut self) -> *mut sys::cui_ctx {
        self.ctx
    }

    /// Copies a string into frame memory as a C string (valid until the next
    /// frame). Handy with [`Ui::raw`].
    pub fn c_str(&mut self, s: &str) -> *const c_char {
        self.arena.str(s)
    }

    pub(crate) fn s(&mut self, s: &str) -> *const c_char {
        self.arena.str(s)
    }

    pub(crate) fn strs(&mut self, items: &[&str]) -> *const *const c_char {
        self.arena.strs(items)
    }

    // ---- frame ----

    /// Size of the drawable area in logical points.
    pub fn viewport_size(&mut self) -> Vec2 {
        unsafe { sys::cui_viewport_size(self.ctx) }.into()
    }
    /// Points to pixels (2.0 on a typical retina display).
    pub fn dpi_scale(&mut self) -> f32 {
        unsafe { sys::cui_dpi_scale(self.ctx) }
    }
    /// Seconds since the context was created.
    pub fn time(&mut self) -> f64 {
        unsafe { sys::cui_time(self.ctx) }
    }
    /// Seconds since the last frame.
    pub fn delta_time(&mut self) -> f32 {
        unsafe { sys::cui_delta_time(self.ctx) }
    }
    pub fn frame_count(&mut self) -> u64 {
        unsafe { sys::cui_frame_count(self.ctx) }
    }
    /// Ask for another frame even if nothing changes (for your own animation).
    pub fn request_redraw(&mut self) {
        unsafe { sys::cui_request_redraw(self.ctx) }
    }

    // ---- input queries ----

    /// Pressed this frame (with key repeat).
    pub fn key_pressed(&mut self, key: Key) -> bool {
        unsafe { sys::cui_key_pressed(self.ctx, key.to_raw()) }
    }
    pub fn key_down(&mut self, key: Key) -> bool {
        unsafe { sys::cui_key_down(self.ctx, key.to_raw()) }
    }
    /// E.g. `ui.shortcut(Mods::PRIMARY, Key::K)`.
    pub fn shortcut(&mut self, mods: Mods, key: Key) -> bool {
        unsafe { sys::cui_shortcut(self.ctx, mods.0, key.to_raw()) }
    }
    pub fn key_mods(&mut self) -> Mods {
        Mods(unsafe { sys::cui_key_mods(self.ctx) })
    }
    pub fn mouse_pos(&mut self) -> Vec2 {
        unsafe { sys::cui_mouse_pos(self.ctx) }.into()
    }
    pub fn mouse_down(&mut self, b: MouseButton) -> bool {
        unsafe { sys::cui_mouse_down(self.ctx, b.to_raw()) }
    }
    pub fn mouse_clicked(&mut self, b: MouseButton) -> bool {
        unsafe { sys::cui_mouse_clicked(self.ctx, b.to_raw()) }
    }
    /// A text field has keyboard focus, so typing is not a shortcut.
    pub fn wants_keyboard(&mut self) -> bool {
        unsafe { sys::cui_wants_keyboard(self.ctx) }
    }
    pub fn wants_mouse(&mut self) -> bool {
        unsafe { sys::cui_wants_mouse(self.ctx) }
    }
    pub fn cursor(&mut self) -> Option<Cursor> {
        Cursor::from_raw(unsafe { sys::cui_get_cursor(self.ctx) })
    }
    /// Files dropped on the window this frame (dropzones claim theirs first).
    pub fn dropped_files(&mut self) -> Vec<String> {
        let n = unsafe { sys::cui_dropped_file_count(self.ctx) };
        (0..n)
            .filter_map(|i| unsafe { cstr(sys::cui_dropped_file(self.ctx, i)) })
            .collect()
    }
    /// Files are being dragged over the window.
    pub fn file_drag_active(&mut self) -> bool {
        unsafe { sys::cui_file_drag_active(self.ctx) }
    }

    // ---- theme ----

    pub fn set_theme(&mut self, mode: ThemeMode) {
        unsafe { sys::cui_set_theme(self.ctx, mode.to_raw()) }
    }
    pub fn theme_mode(&mut self) -> ThemeMode {
        ThemeMode::from_raw(unsafe { sys::cui_get_theme_mode(self.ctx) }).unwrap_or_default()
    }
    pub fn set_accent(&mut self, accent: Accent) {
        unsafe { sys::cui_set_accent(self.ctx, accent.to_raw()) }
    }
    pub fn accent(&mut self) -> Accent {
        Accent::from_raw(unsafe { sys::cui_get_accent(self.ctx) }).unwrap_or_default()
    }
    /// The active theme. Edit it to customize; changes apply next frame.
    pub fn theme_mut(&mut self) -> &mut sys::cui_theme {
        unsafe { &mut *sys::cui_theme_get(self.ctx) }
    }
    pub fn set_reduced_motion(&mut self, reduced: bool) {
        unsafe { sys::cui_set_reduced_motion(self.ctx, reduced) }
    }
    /// Resolves a token color to the real color in the current theme.
    pub fn color_of(&mut self, c: Color) -> Color {
        unsafe { sys::cui_color_of(self.ctx, c.into()) }.into()
    }

    // ---- fonts ----

    /// Loads a TrueType/OpenType font (the data is copied).
    pub fn font_load(&mut self, data: &[u8]) -> Option<Font> {
        let f = unsafe { sys::cui_font_load_memory(self.ctx, data.as_ptr().cast(), data.len(), true) };
        (f >= 0).then_some(Font(f))
    }
    pub fn font_load_file(&mut self, path: &str) -> Option<Font> {
        let p = self.s(path);
        let f = unsafe { sys::cui_font_load_file(self.ctx, p) };
        (f >= 0).then_some(Font(f))
    }
    /// Replace a built-in role (e.g. your brand font as [`Font::BODY`]).
    pub fn font_set_role(&mut self, role: Font, font: Font) {
        unsafe { sys::cui_font_set_role(self.ctx, role.0, font.0) }
    }
    /// Glyphs missing from `font` are looked up in `fallback` (CJK, emoji).
    pub fn font_add_fallback(&mut self, font: Font, fallback: Font) {
        unsafe { sys::cui_font_add_fallback(self.ctx, font.0, fallback.0) }
    }
    /// Returns (width, height) of `text` in points.
    pub fn measure_text(&mut self, font: Font, size: f32, text: &str) -> Vec2 {
        let m = unsafe { sys::cui_measure_text(self.ctx, font.0, size, text.as_ptr().cast(), text.len() as i32) };
        Vec2 { x: m.width, y: m.height }
    }

    // ---- identity ----

    pub fn make_id(&mut self, s: &str) -> Id {
        let p = self.s(s);
        Id(unsafe { sys::cui_make_id(self.ctx, p) })
    }
    /// Runs `f` inside an id scope, so repeated items get distinct
    /// identities. The key is a string or an integer: `ui.id_scope(i, |ui| ...)`.
    pub fn id_scope<R>(&mut self, key: impl IdKey, f: impl FnOnce(&mut Self) -> R) -> R {
        key.push(self);
        let r = f(self);
        unsafe { sys::cui_pop_id(self.ctx) };
        r
    }
    /// Id of the most recent component.
    pub fn last_id(&mut self) -> Id {
        Id(unsafe { sys::cui_last_id(self.ctx) })
    }
    /// Rectangle of the most recent component.
    pub fn last_rect(&mut self) -> Rect {
        unsafe { sys::cui_last_rect(self.ctx) }.into()
    }
    pub fn last_hovered(&mut self) -> bool {
        unsafe { sys::cui_last_hovered(self.ctx) }
    }
    /// Rectangle of any component or box from the latest layout.
    pub fn rect_of(&mut self, id: Id) -> Rect {
        unsafe { sys::cui_get_rect(self.ctx, id.0) }.into()
    }
    pub fn is_disabled(&mut self) -> bool {
        unsafe { sys::cui_is_disabled(self.ctx) }
    }
    /// Everything built in `f` is disabled when `disabled` is true.
    pub fn disabled<R>(&mut self, disabled: bool, f: impl FnOnce(&mut Self) -> R) -> R {
        unsafe { sys::cui_push_disabled(self.ctx, disabled) };
        let r = f(self);
        unsafe { sys::cui_pop_disabled(self.ctx) };
        r
    }

    // ---- images ----

    /// Uploads RGBA pixels (`w * h * 4` bytes) as a texture.
    pub fn image_create(&mut self, w: i32, h: i32, rgba: &[u8]) -> Texture {
        assert!(rgba.len() >= (w * h * 4) as usize, "image_create: buffer smaller than w * h * 4");
        Texture(unsafe { sys::cui_image_create(self.ctx, w, h, rgba.as_ptr().cast()) })
    }
    /// Loads a PNG or JPEG file. Returns the texture and its size.
    pub fn image_load(&mut self, path: &str) -> Option<(Texture, i32, i32)> {
        let p = self.s(path);
        let (mut w, mut h) = (0, 0);
        let t = unsafe { sys::cui_image_load(self.ctx, p, &mut w, &mut h) };
        (t != 0).then_some((Texture(t), w, h))
    }
    pub fn image_destroy(&mut self, tex: Texture) {
        unsafe { sys::cui_image_destroy(self.ctx, tex.0) }
    }

    // ---- app ----

    /// Closes the window at the end of this frame (no-op without the app layer).
    pub fn quit(&mut self) {
        #[cfg(feature = "app")]
        if !self.app.is_null() {
            unsafe { sys::cui_app_quit(self.app) }
        }
    }
    #[cfg(feature = "app")]
    pub fn set_window_title(&mut self, title: &str) {
        if !self.app.is_null() {
            let p = self.s(title);
            unsafe { sys::cui_app_set_title(self.app, p) }
        }
    }
}

/// Copies a C string that CUI owns.
pub(crate) unsafe fn cstr(p: *const c_char) -> Option<String> {
    if p.is_null() {
        None
    } else {
        Some(CStr::from_ptr(p).to_string_lossy().into_owned())
    }
}

#[allow(dead_code)]
pub(crate) const NULL: *const c_char = ptr::null();

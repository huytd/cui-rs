use crate::arena::Arena;
use crate::sys;
use crate::types::*;
use crate::Ui;
use core::ptr::NonNull;
use std::ffi::CString;

/// CUI without a window: the software renderer draws into memory. Use it
/// for tests, screenshots and servers, or to drive CUI from your own loop
/// and feed it input yourself.
///
/// ```
/// let mut h = cui::Headless::new(400.0, 300.0, 1.0);
/// h.frame(1.0 / 60.0, |ui| { ui.button("OK"); });
/// let (w, _, px) = h.render_rgba();
/// assert_eq!(px.len(), (w * 300 * 4) as usize);
/// ```
pub struct Headless {
    ctx: NonNull<sys::cui_ctx>,
    sw: NonNull<sys::cui_sw_renderer>,
    arena: Arena,
}

impl Headless {
    /// A context with a viewport of `width` x `height` points at `scale`
    /// pixels per point.
    pub fn new(width: f32, height: f32, scale: f32) -> Headless {
        Self::with_theme(width, height, scale, ThemeMode::Light)
    }

    pub fn with_theme(width: f32, height: f32, scale: f32, theme: ThemeMode) -> Headless {
        let sw = NonNull::new(unsafe { sys::cui_sw_create() }).expect("cui_sw_create failed");
        let desc = sys::cui_ctx_desc {
            renderer: unsafe { sys::cui_sw_iface(sw.as_ptr()) },
            theme: theme.to_raw(),
            ..Default::default()
        };
        let ctx = NonNull::new(unsafe { sys::cui_create(&desc) }).expect("cui_create failed");
        unsafe { sys::cui_set_viewport(ctx.as_ptr(), width, height, scale) };
        Headless { ctx, sw, arena: Arena::default() }
    }

    pub fn raw(&self) -> *mut sys::cui_ctx {
        self.ctx.as_ptr()
    }

    pub fn set_viewport(&mut self, width: f32, height: f32, scale: f32) {
        unsafe { sys::cui_set_viewport(self.ctx.as_ptr(), width, height, scale) }
    }

    /// Builds one frame; `dt` is the time since the previous one in seconds.
    pub fn frame<R>(&mut self, dt: f32, f: impl FnOnce(&mut Ui) -> R) -> R {
        let ctx = self.ctx.as_ptr();
        self.arena.reset();
        unsafe { sys::cui_begin_frame(ctx, dt) };
        let mut ui = Ui {
            ctx,
            arena: &mut self.arena,
            #[cfg(feature = "app")]
            app: core::ptr::null_mut(),
        };
        let r = f(&mut ui);
        unsafe { sys::cui_end_frame(ctx) };
        r
    }

    /// Something animates or input is settling.
    pub fn needs_redraw(&self) -> bool {
        unsafe { sys::cui_needs_redraw(self.ctx.as_ptr()) }
    }

    /// Renders the last frame: (width, height, RGBA pixels).
    pub fn render_rgba(&mut self) -> (i32, i32, Vec<u8>) {
        let dd = unsafe { sys::cui_get_draw_data(self.ctx.as_ptr()) };
        let (w, h) = unsafe { ((*dd).width_px, (*dd).height_px) };
        let mut px = vec![0u8; (w * h * 4) as usize];
        unsafe { sys::cui_sw_render(self.sw.as_ptr(), dd, px.as_mut_ptr().cast(), w, h, w * 4, sys::CUI_SW_RGBA) };
        (w, h, px)
    }

    /// Renders the last frame to a PNG file.
    pub fn save_png(&mut self, path: &str) -> bool {
        let (w, h, px) = self.render_rgba();
        let p = CString::new(path).unwrap();
        unsafe { sys::cui_write_png(p.as_ptr(), px.as_ptr().cast(), w, h, w * 4) }
    }

    // ---- input, applied to the next frame ----

    pub fn mouse_move(&mut self, x: f32, y: f32) {
        unsafe { sys::cui_input_mouse_move(self.ctx.as_ptr(), x, y) }
    }
    pub fn mouse_button(&mut self, button: MouseButton, down: bool) {
        unsafe { sys::cui_input_mouse_button(self.ctx.as_ptr(), button.to_raw(), down) }
    }
    /// In lines; positive `dy` scrolls up.
    pub fn scroll(&mut self, dx: f32, dy: f32) {
        unsafe { sys::cui_input_scroll(self.ctx.as_ptr(), dx, dy) }
    }
    pub fn key(&mut self, key: Key, down: bool, mods: Mods) {
        unsafe { sys::cui_input_key(self.ctx.as_ptr(), key.to_raw(), down, mods.0) }
    }
    pub fn text(&mut self, text: &str) {
        let t = CString::new(text.replace('\0', "")).unwrap();
        unsafe { sys::cui_input_text(self.ctx.as_ptr(), t.as_ptr()) }
    }
    pub fn focus(&mut self, window_focused: bool) {
        unsafe { sys::cui_input_focus(self.ctx.as_ptr(), window_focused) }
    }
}

impl Drop for Headless {
    fn drop(&mut self) {
        unsafe {
            sys::cui_destroy(self.ctx.as_ptr());
            sys::cui_sw_destroy(self.sw.as_ptr());
        }
    }
}

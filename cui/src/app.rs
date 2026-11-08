use crate::arena::Arena;
use crate::sys;
use crate::types::*;
use crate::ui::cstr;
use crate::Ui;
use core::ptr::NonNull;
use std::ffi::CString;

c_enum! {
    pub enum RendererKind: cui_renderer_kind {
        /// OpenGL, falling back to software.
        Auto = CUI_RENDERER_AUTO,
        OpenGl = CUI_RENDERER_OPENGL,
        Software = CUI_RENDERER_SOFTWARE,
    }
}

c_enum! {
    pub enum FileDialogKind: cui_file_dialog_kind { Open = CUI_FILE_OPEN, Save = CUI_FILE_SAVE, Folder = CUI_FILE_FOLDER }
}

/// How to create the window.
#[derive(Clone, Debug, Default)]
pub struct AppDesc {
    /// Default "CUI".
    pub title: Option<String>,
    /// Logical size, default 1080 x 720.
    pub width: i32,
    pub height: i32,
    /// Default 360 x 280.
    pub min_width: i32,
    pub min_height: i32,
    /// Not resizable.
    pub fixed_size: bool,
    /// Render every frame even when idle (games, video).
    pub continuous: bool,
    pub no_vsync: bool,
    /// Create the window hidden (tests).
    pub hidden: bool,
    pub renderer: RendererKind,
    pub theme: ThemeMode,
    pub accent: Accent,
    /// Use the OS light/dark setting where available.
    pub follow_system_theme: bool,
}

impl AppDesc {
    pub fn new(title: &str) -> Self {
        AppDesc { title: Some(title.to_owned()), ..Default::default() }
    }
    pub fn size(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    pub fn min_size(mut self, width: i32, height: i32) -> Self {
        self.min_width = width;
        self.min_height = height;
        self
    }
    pub fn theme(mut self, theme: ThemeMode) -> Self {
        self.theme = theme;
        self
    }
    pub fn accent(mut self, accent: Accent) -> Self {
        self.accent = accent;
        self
    }
    pub fn follow_system_theme(mut self, on: bool) -> Self {
        self.follow_system_theme = on;
        self
    }
    pub fn renderer(mut self, renderer: RendererKind) -> Self {
        self.renderer = renderer;
        self
    }
    pub fn continuous(mut self, on: bool) -> Self {
        self.continuous = on;
        self
    }
    pub fn fixed_size(mut self, on: bool) -> Self {
        self.fixed_size = on;
        self
    }
    pub fn hidden(mut self, on: bool) -> Self {
        self.hidden = on;
        self
    }
}

/// A native file dialog request.
#[derive(Clone, Debug, Default)]
pub struct FileDialog {
    pub kind: FileDialogKind,
    pub title: Option<String>,
    /// E.g. "Images".
    pub filter_name: Option<String>,
    /// Extensions separated by ';', e.g. "png;jpg".
    pub filter: Option<String>,
    /// For save dialogs.
    pub default_name: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    /// The window or its renderer could not be created.
    CreateFailed,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("could not create the CUI window")
    }
}

impl std::error::Error for Error {}

/// A window with an event loop and a renderer.
///
/// ```no_run
/// let mut app = cui::App::new(&cui::AppDesc::new("Hello")).unwrap();
/// while let Some(mut ui) = app.frame() {
///     ui.heading("Hello, CUI", 2);
/// }
/// ```
pub struct App {
    raw: NonNull<sys::cui_app>,
    arena: Arena,
}

impl App {
    pub fn new(desc: &AppDesc) -> Result<App, Error> {
        let title = desc.title.as_deref().map(|t| CString::new(t.replace('\0', "")).unwrap());
        let d = sys::cui_app_desc {
            title: title.as_ref().map_or(core::ptr::null(), |t| t.as_ptr()),
            width: desc.width,
            height: desc.height,
            min_width: desc.min_width,
            min_height: desc.min_height,
            fixed_size: desc.fixed_size,
            continuous: desc.continuous,
            no_vsync: desc.no_vsync,
            hidden: desc.hidden,
            renderer: desc.renderer.to_raw(),
            theme: desc.theme.to_raw(),
            accent: desc.accent.to_raw(),
            follow_system_theme: desc.follow_system_theme,
        };
        let raw = NonNull::new(unsafe { sys::cui_app_create(&d) }).ok_or(Error::CreateFailed)?;
        Ok(App { raw, arena: Arena::default() })
    }

    /// Creates the window and calls `frame` until it closes. Call
    /// [`Ui::quit`] to close it from your code.
    pub fn run(desc: &AppDesc, mut frame: impl FnMut(&mut Ui)) -> Result<(), Error> {
        let mut app = App::new(desc)?;
        while let Some(mut ui) = app.frame() {
            frame(&mut ui);
        }
        Ok(())
    }

    /// Presents the previous frame, waits for input or animation, and begins
    /// the next one. Returns `None` once the window is closed.
    pub fn frame(&mut self) -> Option<Ui<'_>> {
        if !unsafe { sys::cui_app_frame(self.raw.as_ptr()) } {
            return None;
        }
        // The previous frame is finished now, so its strings can go.
        self.arena.reset();
        Some(Ui {
            ctx: unsafe { sys::cui_app_ui(self.raw.as_ptr()) },
            arena: &mut self.arena,
            app: self.raw.as_ptr(),
        })
    }

    pub fn raw(&self) -> *mut sys::cui_app {
        self.raw.as_ptr()
    }
    pub fn quit(&mut self) {
        unsafe { sys::cui_app_quit(self.raw.as_ptr()) }
    }
    pub fn set_title(&mut self, title: &str) {
        let t = CString::new(title.replace('\0', "")).unwrap();
        unsafe { sys::cui_app_set_title(self.raw.as_ptr(), t.as_ptr()) }
    }
    pub fn set_size(&mut self, width: i32, height: i32) {
        unsafe { sys::cui_app_set_size(self.raw.as_ptr(), width, height) }
    }
    /// Window size in logical points.
    pub fn size(&self) -> Vec2 {
        unsafe { sys::cui_app_size(self.raw.as_ptr()) }.into()
    }
    pub fn is_software(&self) -> bool {
        unsafe { sys::cui_app_is_software(self.raw.as_ptr()) }
    }
    /// HWND, NSWindow*, X11 Window, SDL_Window* or GLFWwindow*.
    pub fn native_window(&self) -> *mut core::ffi::c_void {
        unsafe { sys::cui_app_native_window(self.raw.as_ptr()) }
    }
    /// "win32", "cocoa", "x11", "sdl2" or "glfw".
    pub fn backend(&self) -> String {
        unsafe { cstr(sys::cui_app_backend(self.raw.as_ptr())) }.unwrap_or_default()
    }

    /// Renders the current UI into RGBA pixels: (width, height, pixels).
    pub fn screenshot(&mut self) -> Option<(i32, i32, Vec<u8>)> {
        let ctx = unsafe { sys::cui_app_ui(self.raw.as_ptr()) };
        let dd = unsafe { &*sys::cui_get_draw_data(ctx) };
        let (w, h) = (dd.width_px, dd.height_px);
        let mut px = vec![0u8; (w.max(0) * h.max(0) * 4) as usize];
        unsafe { sys::cui_app_screenshot(self.raw.as_ptr(), px.as_mut_ptr().cast(), w, h) }.then_some((w, h, px))
    }

    /// Shows a blocking native file dialog and returns the chosen path. On
    /// Linux this uses zenity or kdialog when installed.
    pub fn file_dialog(&mut self, dialog: &FileDialog) -> Option<String> {
        file_dialog(self.raw.as_ptr(), dialog)
    }
}

impl Drop for App {
    fn drop(&mut self) {
        unsafe { sys::cui_app_destroy(self.raw.as_ptr()) }
    }
}

impl<'f> Ui<'f> {
    /// [`App::file_dialog`] from inside a frame.
    pub fn file_dialog(&mut self, dialog: &FileDialog) -> Option<String> {
        file_dialog(self.app, dialog)
    }
}

fn file_dialog(app: *mut sys::cui_app, dialog: &FileDialog) -> Option<String> {
    let c = |s: &Option<String>| s.as_deref().map(|s| CString::new(s.replace('\0', "")).unwrap());
    let (title, filter_name, filter, default_name) =
        (c(&dialog.title), c(&dialog.filter_name), c(&dialog.filter), c(&dialog.default_name));
    let p = |s: &Option<CString>| s.as_ref().map_or(core::ptr::null(), |s| s.as_ptr());
    let o = sys::cui_file_dialog_opts {
        kind: dialog.kind.to_raw(),
        title: p(&title),
        filter_name: p(&filter_name),
        filter: p(&filter),
        default_name: p(&default_name),
    };
    let mut out = vec![0u8; 4096];
    if !unsafe { sys::cui_app_file_dialog(app, &o, out.as_mut_ptr().cast(), out.len()) } {
        return None;
    }
    let n = out.iter().position(|&b| b == 0).unwrap_or(out.len());
    Some(String::from_utf8_lossy(&out[..n]).into_owned())
}

/// Opens a URL or file with the system's default handler.
pub fn open_url(url: &str) -> bool {
    let u = CString::new(url.replace('\0', "")).unwrap();
    unsafe { sys::cui_app_open_url(u.as_ptr()) }
}

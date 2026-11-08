use crate::arena::Arena;
use crate::sys;
use core::ffi::{c_char, c_int};

/// Converts a Rust option value into the field type of a C options struct.
pub(crate) trait ToC<T> {
    fn to_c(&self, a: &mut Arena) -> T;
}

macro_rules! to_c_identity {
    ($($t:ty),*) => {$(
        impl ToC<$t> for $t {
            fn to_c(&self, _: &mut Arena) -> $t { *self }
        }
    )*};
}
to_c_identity!(bool, f32, f64, i32);

impl ToC<*const c_char> for Option<&str> {
    fn to_c(&self, a: &mut Arena) -> *const c_char {
        a.opt_str(*self)
    }
}

impl ToC<*const *const c_char> for Option<&[&str]> {
    fn to_c(&self, a: &mut Arena) -> *const *const c_char {
        self.map_or(core::ptr::null(), |s| a.strs(s))
    }
}

/// Defines a Rust enum over a C enum, keeping the C values (so a renumbered
/// constant upstream is picked up, and a removed one fails to compile).
macro_rules! c_enum {
    ($(#[$m:meta])* pub enum $name:ident : $cty:ident { $( $(#[$vm:meta])* $v:ident = $c:ident ),* $(,)? }) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $( $(#[$vm])* $v ),* }

        impl $name {
            /// The C value.
            pub const fn to_raw(self) -> sys::$cty {
                match self { $( Self::$v => sys::$c as sys::$cty ),* }
            }
            /// The variant for a C value, if it is known.
            #[allow(clippy::unnecessary_cast)]
            pub fn from_raw(v: sys::$cty) -> Option<Self> {
                $( if v == sys::$c as sys::$cty { return Some(Self::$v); } )*
                None
            }
        }

        impl Default for $name {
            fn default() -> Self { [$(Self::$v),*][0] }
        }

        impl ToC<sys::$cty> for $name {
            fn to_c(&self, _: &mut Arena) -> sys::$cty { self.to_raw() }
        }
    };
}
pub(crate) use c_enum;

c_enum! {
    /// Light or dark.
    pub enum ThemeMode: cui_theme_mode { Light = CUI_THEME_LIGHT, Dark = CUI_THEME_DARK }
}

c_enum! {
    /// The accent hue that controls, focus rings and selections use.
    pub enum Accent: cui_accent {
        Neutral = CUI_ACCENT_NEUTRAL, Violet = CUI_ACCENT_VIOLET, Blue = CUI_ACCENT_BLUE,
        Green = CUI_ACCENT_GREEN, Amber = CUI_ACCENT_AMBER, Orange = CUI_ACCENT_ORANGE,
        Coral = CUI_ACCENT_CORAL, Rose = CUI_ACCENT_ROSE,
    }
}

c_enum! {
    pub enum MouseButton: cui_mouse_button { Left = CUI_MOUSE_LEFT, Right = CUI_MOUSE_RIGHT, Middle = CUI_MOUSE_MIDDLE }
}

c_enum! {
    pub enum Key: cui_key {
        None = CUI_KEY_NONE, Tab = CUI_KEY_TAB, Enter = CUI_KEY_ENTER, Escape = CUI_KEY_ESCAPE,
        Backspace = CUI_KEY_BACKSPACE, Delete = CUI_KEY_DELETE, Space = CUI_KEY_SPACE,
        Left = CUI_KEY_LEFT, Right = CUI_KEY_RIGHT, Up = CUI_KEY_UP, Down = CUI_KEY_DOWN,
        Home = CUI_KEY_HOME, End = CUI_KEY_END, PageUp = CUI_KEY_PAGE_UP, PageDown = CUI_KEY_PAGE_DOWN,
        A = CUI_KEY_A, B = CUI_KEY_B, C = CUI_KEY_C, D = CUI_KEY_D, E = CUI_KEY_E, F = CUI_KEY_F,
        G = CUI_KEY_G, H = CUI_KEY_H, I = CUI_KEY_I, J = CUI_KEY_J, K = CUI_KEY_K, L = CUI_KEY_L,
        M = CUI_KEY_M, N = CUI_KEY_N, O = CUI_KEY_O, P = CUI_KEY_P, Q = CUI_KEY_Q, R = CUI_KEY_R,
        S = CUI_KEY_S, T = CUI_KEY_T, U = CUI_KEY_U, V = CUI_KEY_V, W = CUI_KEY_W, X = CUI_KEY_X,
        Y = CUI_KEY_Y, Z = CUI_KEY_Z,
        Num0 = CUI_KEY_0, Num1 = CUI_KEY_1, Num2 = CUI_KEY_2, Num3 = CUI_KEY_3, Num4 = CUI_KEY_4,
        Num5 = CUI_KEY_5, Num6 = CUI_KEY_6, Num7 = CUI_KEY_7, Num8 = CUI_KEY_8, Num9 = CUI_KEY_9,
        Minus = CUI_KEY_MINUS, Equal = CUI_KEY_EQUAL, Comma = CUI_KEY_COMMA, Period = CUI_KEY_PERIOD,
        Slash = CUI_KEY_SLASH,
        F1 = CUI_KEY_F1, F2 = CUI_KEY_F2, F3 = CUI_KEY_F3, F4 = CUI_KEY_F4, F5 = CUI_KEY_F5,
        F6 = CUI_KEY_F6, F7 = CUI_KEY_F7, F8 = CUI_KEY_F8, F9 = CUI_KEY_F9, F10 = CUI_KEY_F10,
        F11 = CUI_KEY_F11, F12 = CUI_KEY_F12,
    }
}

c_enum! {
    pub enum Cursor: cui_cursor {
        Arrow = CUI_CURSOR_ARROW, Hand = CUI_CURSOR_HAND, Text = CUI_CURSOR_TEXT,
        ResizeEw = CUI_CURSOR_RESIZE_EW, ResizeNs = CUI_CURSOR_RESIZE_NS, Move = CUI_CURSOR_MOVE,
        NotAllowed = CUI_CURSOR_NOT_ALLOWED,
    }
}

c_enum! {
    /// How a box lays out its children.
    pub enum Dir: cui_dir { Column = CUI_COLUMN, Row = CUI_ROW, Stack = CUI_STACK, Grid = CUI_GRID }
}

c_enum! {
    /// Cross axis alignment. `Default`: columns stretch their children, rows center them.
    pub enum Align: cui_align {
        Default = CUI_ALIGN_DEFAULT, Start = CUI_ALIGN_START, Center = CUI_ALIGN_CENTER,
        End = CUI_ALIGN_END, Stretch = CUI_ALIGN_STRETCH,
    }
}

c_enum! {
    /// Main axis distribution.
    pub enum Justify: cui_justify {
        Start = CUI_JUSTIFY_START, Center = CUI_JUSTIFY_CENTER, End = CUI_JUSTIFY_END,
        Between = CUI_JUSTIFY_BETWEEN, Around = CUI_JUSTIFY_AROUND,
    }
}

c_enum! {
    pub enum Elevation: cui_elevation {
        None = CUI_ELEVATION_NONE, Resting = CUI_ELEVATION_RESTING,
        Raised = CUI_ELEVATION_RAISED, Floating = CUI_ELEVATION_FLOATING,
    }
}

c_enum! {
    pub enum TextAlign: cui_text_align { Left = CUI_TEXT_LEFT, Center = CUI_TEXT_CENTER, Right = CUI_TEXT_RIGHT }
}

c_enum! {
    pub enum ButtonVariant: cui_button_variant {
        /// Secondary for buttons, ghost for icon buttons.
        Default = CUI_BUTTON_DEFAULT,
        /// Neutral filled.
        Secondary = CUI_BUTTON_SECONDARY,
        /// The one main action.
        Primary = CUI_BUTTON_PRIMARY,
        Outline = CUI_BUTTON_OUTLINE,
        Ghost = CUI_BUTTON_GHOST,
        Danger = CUI_BUTTON_DANGER,
        Link = CUI_BUTTON_LINK,
    }
}

c_enum! {
    pub enum ControlSize: cui_control_size { Md = CUI_SIZE_MD, Sm = CUI_SIZE_SM, Lg = CUI_SIZE_LG }
}

c_enum! {
    pub enum TabsVariant: cui_tabs_variant { Underline = CUI_TABS_UNDERLINE, Pill = CUI_TABS_PILL }
}

c_enum! {
    pub enum Placement: cui_placement {
        BottomStart = CUI_PLACE_BOTTOM_START, Bottom = CUI_PLACE_BOTTOM, BottomEnd = CUI_PLACE_BOTTOM_END,
        TopStart = CUI_PLACE_TOP_START, Top = CUI_PLACE_TOP, TopEnd = CUI_PLACE_TOP_END,
        RightStart = CUI_PLACE_RIGHT_START, LeftStart = CUI_PLACE_LEFT_START,
        Right = CUI_PLACE_RIGHT, Left = CUI_PLACE_LEFT,
    }
}

c_enum! {
    pub enum Side: cui_side { Right = CUI_SIDE_RIGHT, Left = CUI_SIDE_LEFT, Bottom = CUI_SIDE_BOTTOM, Top = CUI_SIDE_TOP }
}

c_enum! {
    pub enum Tone: cui_tone {
        Neutral = CUI_TONE_NEUTRAL, Info = CUI_TONE_INFO, Success = CUI_TONE_SUCCESS,
        Warning = CUI_TONE_WARNING, Danger = CUI_TONE_DANGER, Violet = CUI_TONE_VIOLET,
        Teal = CUI_TONE_TEAL, Rose = CUI_TONE_ROSE, Accent = CUI_TONE_ACCENT,
    }
}

c_enum! {
    /// Semantic colors. Use them (through [`Color::token`]) instead of raw
    /// colors so the UI follows the theme and the accent.
    pub enum Token: cui_token {
        None = CUI_TOKEN_NONE, Background = CUI_TOKEN_BACKGROUND, Surface = CUI_TOKEN_SURFACE,
        SurfaceRaised = CUI_TOKEN_SURFACE_RAISED, SurfaceMuted = CUI_TOKEN_SURFACE_MUTED,
        Foreground = CUI_TOKEN_FOREGROUND, TextSecondary = CUI_TOKEN_TEXT_SECONDARY,
        TextMuted = CUI_TOKEN_TEXT_MUTED, Border = CUI_TOKEN_BORDER, BorderSubtle = CUI_TOKEN_BORDER_SUBTLE,
        BorderStrong = CUI_TOKEN_BORDER_STRONG, Accent = CUI_TOKEN_ACCENT, AccentStrong = CUI_TOKEN_ACCENT_STRONG,
        AccentSubtle = CUI_TOKEN_ACCENT_SUBTLE, AccentForeground = CUI_TOKEN_ACCENT_FOREGROUND,
        Success = CUI_TOKEN_SUCCESS, Warning = CUI_TOKEN_WARNING, Danger = CUI_TOKEN_DANGER, Info = CUI_TOKEN_INFO,
        ControlOn = CUI_TOKEN_CONTROL_ON, ControlGlyph = CUI_TOKEN_CONTROL_GLYPH,
        ControlTrack = CUI_TOKEN_CONTROL_TRACK, ControlTrackHover = CUI_TOKEN_CONTROL_TRACK_HOVER,
        ControlThumb = CUI_TOKEN_CONTROL_THUMB, ControlThumbOn = CUI_TOKEN_CONTROL_THUMB_ON,
        Hover = CUI_TOKEN_HOVER, Active = CUI_TOKEN_ACTIVE, Scrim = CUI_TOKEN_SCRIM, FocusRing = CUI_TOKEN_FOCUS_RING,
        Series1 = CUI_TOKEN_SERIES_1, Series2 = CUI_TOKEN_SERIES_2, Series3 = CUI_TOKEN_SERIES_3,
        Series4 = CUI_TOKEN_SERIES_4, Series5 = CUI_TOKEN_SERIES_5, Series6 = CUI_TOKEN_SERIES_6,
    }
}

/// An sRGB color with straight alpha. All zeros (the default) means "not
/// set" in options, so CUI picks its own color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };
    pub const BACKGROUND: Color = Color::token(Token::Background);
    pub const SURFACE: Color = Color::token(Token::Surface);
    pub const SURFACE_RAISED: Color = Color::token(Token::SurfaceRaised);
    pub const SURFACE_MUTED: Color = Color::token(Token::SurfaceMuted);
    pub const FOREGROUND: Color = Color::token(Token::Foreground);
    pub const TEXT_SECONDARY: Color = Color::token(Token::TextSecondary);
    pub const TEXT_MUTED: Color = Color::token(Token::TextMuted);
    pub const BORDER: Color = Color::token(Token::Border);
    pub const ACCENT: Color = Color::token(Token::Accent);
    pub const SUCCESS: Color = Color::token(Token::Success);
    pub const WARNING: Color = Color::token(Token::Warning);
    pub const DANGER: Color = Color::token(Token::Danger);
    pub const INFO: Color = Color::token(Token::Info);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
    /// `Color::rgb(0x3b82f6)`
    pub const fn rgb(hex: u32) -> Self {
        Color { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8, a: 255 }
    }
    /// `Color::rgba(0x3b82f680)`
    pub const fn rgba(hex: u32) -> Self {
        Color { r: (hex >> 24) as u8, g: (hex >> 16) as u8, b: (hex >> 8) as u8, a: hex as u8 }
    }
    /// A theme color that CUI resolves when drawing (`CUI_TOKEN_COLOR`).
    pub const fn token(t: Token) -> Self {
        Color { r: t.to_raw() as u8, g: 0xC5, b: 0x1D, a: 0 }
    }
    /// One of the six chart series colors.
    pub const fn series(n: usize) -> Self {
        let t = [Token::Series1, Token::Series2, Token::Series3, Token::Series4, Token::Series5, Token::Series6];
        Color::token(t[n % 6])
    }
    /// A color from OKLCH (lightness 0..1, chroma, hue in degrees).
    pub fn oklch(l: f32, c: f32, h_degrees: f32, alpha: f32) -> Self {
        unsafe { sys::cui_oklch(l, c, h_degrees, alpha) }.into()
    }
    /// Mixes in OKLab, like CSS `color-mix(in oklab)`. Resolve tokens first.
    pub fn mix(self, other: Color, t: f32) -> Self {
        unsafe { sys::cui_mix(self.into(), other.into(), t) }.into()
    }
    pub fn with_alpha(self, alpha_multiplier: f32) -> Self {
        unsafe { sys::cui_alpha(self.into(), alpha_multiplier) }.into()
    }
}

impl From<Token> for Color {
    fn from(t: Token) -> Self {
        Color::token(t)
    }
}
impl From<sys::cui_color> for Color {
    fn from(c: sys::cui_color) -> Self {
        Color { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}
impl From<Color> for sys::cui_color {
    fn from(c: Color) -> Self {
        sys::cui_color { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}
impl ToC<sys::cui_color> for Color {
    fn to_c(&self, _: &mut Arena) -> sys::cui_color {
        (*self).into()
    }
}

/// A length in layout: fit the content, grow into free space, a fixed size
/// in points, or a percentage of the parent. `200.0.into()` is `Px(200.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Size {
    #[default]
    Fit,
    /// Share of the free space, by weight.
    Grow(f32),
    Px(f32),
    /// 0..100 of the parent.
    Percent(f32),
}

impl Size {
    /// Take all the free space (`CUI_FILL`).
    pub const FILL: Size = Size::Grow(1.0);
}

impl From<f32> for Size {
    fn from(v: f32) -> Self {
        Size::Px(v)
    }
}
impl From<Size> for sys::cui_size {
    fn from(s: Size) -> Self {
        let (kind, value) = match s {
            Size::Fit => (sys::CUI_SIZE_FIT, 0.0),
            Size::Grow(w) => (sys::CUI_SIZE_GROW, w),
            Size::Px(v) => (sys::CUI_SIZE_FIXED, v),
            Size::Percent(p) => (sys::CUI_SIZE_PERCENT, p),
        };
        sys::cui_size { kind, value }
    }
}
impl ToC<sys::cui_size> for Size {
    fn to_c(&self, _: &mut Arena) -> sys::cui_size {
        (*self).into()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
impl From<sys::cui_vec2> for Vec2 {
    fn from(v: sys::cui_vec2) -> Self {
        Vec2 { x: v.x, y: v.y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
impl Rect {
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.w && p.y < self.y + self.h
    }
}
impl From<sys::cui_rect> for Rect {
    fn from(r: sys::cui_rect) -> Self {
        Rect { x: r.x, y: r.y, w: r.w, h: r.h }
    }
}
impl From<Rect> for sys::cui_rect {
    fn from(r: Rect) -> Self {
        sys::cui_rect { x: r.x, y: r.y, w: r.w, h: r.h }
    }
}

/// A calendar date; `month` is 1..=12.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}
impl Date {
    pub fn new(year: i32, month: i32, day: i32) -> Self {
        Date { year, month, day }
    }
    pub fn today() -> Self {
        unsafe { sys::cui_date_today() }.into()
    }
}
impl From<sys::cui_date> for Date {
    fn from(d: sys::cui_date) -> Self {
        Date { year: d.year, month: d.month, day: d.day }
    }
}
impl From<Date> for sys::cui_date {
    fn from(d: Date) -> Self {
        sys::cui_date { year: d.year, month: d.month, day: d.day }
    }
}

/// Identity of a component or box, from [`crate::Ui::last_id`] and friends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Id(pub sys::cui_id);

impl ToC<sys::cui_id> for Id {
    fn to_c(&self, _: &mut Arena) -> sys::cui_id {
        self.0
    }
}

/// A font id: one of the built-in roles or a font you loaded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Font(pub sys::cui_font);

impl Font {
    /// Inter Regular.
    pub const BODY: Font = Font(sys::CUI_FONT_BODY as c_int);
    /// Inter Medium.
    pub const MEDIUM: Font = Font(sys::CUI_FONT_MEDIUM as c_int);
    /// Geist Medium, for headings 28pt and up.
    pub const DISPLAY: Font = Font(sys::CUI_FONT_DISPLAY as c_int);
    /// Geist Mono.
    pub const MONO: Font = Font(sys::CUI_FONT_MONO as c_int);
    /// Lucide icons.
    pub const ICONS: Font = Font(sys::CUI_FONT_ICONS as c_int);
}

impl ToC<sys::cui_font> for Font {
    fn to_c(&self, _: &mut Arena) -> sys::cui_font {
        self.0
    }
}

/// A texture (an image uploaded with [`crate::Ui::image_create`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Texture(pub sys::cui_texture);

impl ToC<sys::cui_texture> for Texture {
    fn to_c(&self, _: &mut Arena) -> sys::cui_texture {
        self.0
    }
}

/// Keyboard modifier set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mods(pub i32);

impl Mods {
    pub const NONE: Mods = Mods(0);
    pub const SHIFT: Mods = Mods(sys::CUI_MOD_SHIFT as i32);
    pub const CTRL: Mods = Mods(sys::CUI_MOD_CTRL as i32);
    pub const ALT: Mods = Mods(sys::CUI_MOD_ALT as i32);
    /// Command on macOS, the Windows key elsewhere.
    pub const SUPER: Mods = Mods(sys::CUI_MOD_SUPER as i32);
    /// The shortcut modifier: Command on macOS, Ctrl elsewhere.
    #[cfg(target_os = "macos")]
    pub const PRIMARY: Mods = Mods::SUPER;
    /// The shortcut modifier: Command on macOS, Ctrl elsewhere.
    #[cfg(not(target_os = "macos"))]
    pub const PRIMARY: Mods = Mods::CTRL;

    pub fn contains(self, other: Mods) -> bool {
        self.0 & other.0 == other.0
    }
}

impl core::ops::BitOr for Mods {
    type Output = Mods;
    fn bitor(self, rhs: Mods) -> Mods {
        Mods(self.0 | rhs.0)
    }
}

/// What a text field reported this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputEvents(pub i32);

impl InputEvents {
    pub fn changed(self) -> bool {
        self.0 & sys::CUI_CHANGED as i32 != 0
    }
    /// Enter was pressed.
    pub fn submitted(self) -> bool {
        self.0 & sys::CUI_SUBMITTED as i32 != 0
    }
    pub fn focused(self) -> bool {
        self.0 & sys::CUI_FOCUSED as i32 != 0
    }
    pub fn blurred(self) -> bool {
        self.0 & sys::CUI_BLURRED as i32 != 0
    }
}
